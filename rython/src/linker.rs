use std::process::Command;
use std::path::{Path, PathBuf};
use std::env::consts::{OS};
use std::fs;

pub fn manual_link(object_file: &str, output_name: &str) -> Result<(), String> {
    println!("Attempting manual linking...");
    
    // Normalize paths
    let object_path = Path::new(object_file).to_path_buf();
    let output_path = Path::new(output_name).to_path_buf();
    
    // Handle the common case where object file and output have same name but different extensions
    // If they're the same path, automatically add .exe to output for Windows
    if object_path == output_path {
        println!("Note: Object file and output have same name, adjusting output name...");
        
        if OS == "linux" {
            // For cross-compilation to Windows, use .exe
            let new_output = object_path.with_extension("exe");
            return link_windows_cross_compile(&object_path, &new_output);
        } else {
            // For native Windows, also use .exe
            let new_output = object_path.with_extension("exe");
            return link_windows_native(&object_path, &new_output);
        }
    }
    
    // Ensure the object file exists
    if !object_path.exists() {
        return Err(format!("Object file '{}' does not exist", object_file));
    }
    
    // Detect if we're on Linux and trying to build for Windows
    if OS == "linux" {
        return link_windows_cross_compile(&object_path, &output_path);
    }
    
    // Original Windows linking code (for when running on Windows)
    link_windows_native(&object_path, &output_path)
}

fn link_windows_cross_compile(object_path: &PathBuf, output_path: &PathBuf) -> Result<(), String> {
    println!("Cross-compiling for Windows from Linux...");
    
    // Ensure output has .exe extension for Windows
    let output_exe = if let Some(ext) = output_path.extension() {
        if ext == "exe" {
            output_path.clone()
        } else {
            output_path.with_extension("exe")
        }
    } else {
        output_path.with_extension("exe")
    };
    
    // Convert to strings for command
    let object_file = object_path.to_str().ok_or("Invalid object file path")?;
    let output_file = output_exe.to_str().ok_or("Invalid output file path")?;
    
    println!("Linking {} -> {}", object_file, output_file);
    
    // Try using MinGW cross-compiler (check which tools are available)
    let mingw_tools = [
        "x86_64-w64-mingw32-gcc",
        "x86_64-w64-mingw32-g++",
        "x86_64-w64-mingw32-ld",
    ];
    
    let mut found_tool = None;
    for tool in &mingw_tools {
        if Command::new("which").arg(tool).output().is_ok_and(|output| output.status.success()) {
            found_tool = Some(*tool);
            break;
        }
    }
    
    match found_tool {
        Some("x86_64-w64-mingw32-gcc") | Some("x86_64-w64-mingw32-g++") => {
            // Use gcc/g++ for linking (handles libraries and startup code automatically)
            println!("Using {} for linking...", found_tool.unwrap());
            
            // First try: Standard linking with libraries
            let output = Command::new(found_tool.unwrap())
                .args(&[
                    "-o", output_file,
                    object_file,
                    "-L./lib",
                    "-lrython_runtime",
                    "-lmsvcrt",
                    "-mconsole",
                    "-Wl,--entry=main",
                    "-static",
                ])
                .output()
                .map_err(|e| format!("Failed to run {}: {}", found_tool.unwrap(), e))?;
            
            if output.status.success() {
                println!("✓ Standard linking successful!");
            } else {
                println!("Standard linking failed, trying simpler approach...");
                let error_msg = String::from_utf8_lossy(&output.stderr);
                println!("Error: {}", error_msg);
                
                // Second try: Simpler linking without external libraries
                let output = Command::new(found_tool.unwrap())
                    .args(&[
                        "-o", output_file,
                        object_file,
                        "-mconsole",
                        "-static",
                        "-nostdlib",
                        "-Wl,--entry=main",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to run {} (simple): {}", found_tool.unwrap(), e))?;
                
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Cross-compilation linking failed: {}", error_msg));
                }
            }
            
            if Path::new(output_file).exists() {
                let file_size = fs::metadata(output_file).map(|m| m.len()).unwrap_or(0);
                println!("✓ Cross-compilation successful! Output: {} ({} bytes)", output_file, file_size);
                Ok(())
            } else {
                Err("Linking appeared to succeed but output file was not created".to_string())
            }
        }
        Some("x86_64-w64-mingw32-ld") => {
            // Use ld directly
            println!("Using x86_64-w64-mingw32-ld for linking...");
            
            let output = Command::new("x86_64-w64-mingw32-ld")
                .args(&[
                    "-o", output_file,
                    object_file,
                    "-L./lib",
                    "-lrython_runtime",
                    "-lmsvcrt",
                    "--entry", "main",
                    "--subsystem", "console",
                    "--static",
                ])
                .output()
                .map_err(|e| format!("Failed to run x86_64-w64-mingw32-ld: {}", e))?;
            
            if output.status.success() {
                println!("✓ LD linking successful!");
            } else {
                println!("LD linking failed, trying without libraries...");
                
                // Try without libraries
                let output = Command::new("x86_64-w64-mingw32-ld")
                    .args(&[
                        "-o", output_file,
                        object_file,
                        "--entry", "main",
                        "--subsystem", "console",
                        "--static",
                    ])
                    .output()
                    .map_err(|e| format!("Failed to run x86_64-w64-mingw32-ld (simple): {}", e))?;
                    
                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("LD linking failed: {}", error_msg));
                }
            }
            
            if Path::new(output_file).exists() {
                println!("✓ LD linking successful! Output: {}", output_file);
                Ok(())
            } else {
                Err("LD linking appeared to succeed but output file was not created".to_string())
            }
        }
        _ => {
            // No MinGW tools found, try using Wine as a fallback
            link_with_wine(object_file, output_file)
        }
    }
}

fn link_with_wine(object_file: &str, output_file: &str) -> Result<(), String> {
    println!("Attempting to use Wine for linking...");
    
    // Check if Wine is installed
    if Command::new("wine").arg("--version").output().is_err() {
        return Err(
            "Cannot link Windows executable on Linux. Please install one of:\n\
            1. MinGW cross-compiler: sudo apt-get install mingw-w64\n\
            2. Wine: sudo apt-get install wine\n\
            3. Or compile on Windows directly.".to_string()
        );
    }
    
    // Check if Windows linker tools exist
    let ld_exe = "bin/ld.exe";
    if !Path::new(ld_exe).exists() {
        return Err(format!("{} not found. Windows linker tools required.", ld_exe));
    }
    
    println!("Using Wine to run Windows linker...");
    
    // Run ld.exe through Wine
    let output = Command::new("wine")
        .args(&[
            ld_exe,
            "-o", output_file,
            object_file,
            "-L./lib",
            "-lrython_runtime",
            "-lmsvcrt",
            "--entry", "main",
            "--subsystem", "console",
        ])
        .output()
        .map_err(|e| format!("Failed to run Wine: {}", e))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        println!("Wine linking error: {}", error_msg);
        
        // Try alternative entry point
        let output = Command::new("wine")
            .args(&[
                ld_exe,
                "-o", output_file,
                object_file,
                "-L./lib",
                "-lrython_runtime",
                "--entry", "_main",
                "--subsystem", "console",
            ])
            .output()
            .map_err(|e| format!("Failed to run Wine (alternative): {}", e))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Wine linking failed: {}", error_msg));
        }
    }
    
    if Path::new(output_file).exists() {
        println!("✓ Wine linking successful! Output: {}", output_file);
        Ok(())
    } else {
        Err("Wine linking appeared to succeed but output file was not created".to_string())
    }
}

fn link_windows_native(object_path: &PathBuf, output_path: &PathBuf) -> Result<(), String> {
    println!("Linking on Windows host...");
    
    let object_file = object_path.to_str().ok_or("Invalid object file path")?;
    
    // Ensure output has .exe extension
    let output_exe = if let Some(ext) = output_path.extension() {
        if ext == "exe" {
            output_path.clone()
        } else {
            output_path.with_extension("exe")
        }
    } else {
        output_path.with_extension("exe")
    };
    
    let output_file_exe = output_exe.to_str().ok_or("Invalid output file path")?;
    
    let ld_binary = "bin/ld.exe";
    if !Path::new(ld_binary).exists() {
        return Err("ld.exe not found in bin/".to_string());
    }
    
    println!("Linking {} -> {}", object_file, output_file_exe);
    
    let output = Command::new(ld_binary)
        .args(&[
            "-o", output_file_exe,
            object_file,
            "-L./lib",
            "-lrython_runtime",
            "-lmsvcrt",
            "--entry", "main",
            "--subsystem", "console",
        ])
        .output()
        .map_err(|e| format!("Failed to run ld: {}", e))?;
        
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        println!("LD linking failed: {}", error_msg);
        
        // Try with different entry point
        let output = Command::new(ld_binary)
            .args(&[
                "-o", output_file_exe,
                object_file,
                "-L./lib",
                "-lrython_runtime",
                "--entry", "_main",
                "--subsystem", "console",
            ])
            .output()
            .map_err(|e| format!("Failed to run ld (alternative): {}", e))?;
            
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("LD linking failed: {}", error_msg));
        }
    }
    
    if Path::new(output_file_exe).exists() {
        println!("✓ Native Windows linking successful! Output: {}", output_file_exe);
        Ok(())
    } else {
        Err("Linking appeared to succeed but output file was not created".to_string())
    }
}