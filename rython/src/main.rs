//! Rython Compiler Main Entry Point
//! A complete compiler from Rython to NASM assembly

use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

const VERSION: &str = "0.1.0";
const AUTHOR: &str = "Rython Project";

fn print_help() {
    println!("Rython Compiler v{}", VERSION);
    println!("A compiler from Rython to NASM assembly\n");
    println!("Usage: rython [OPTIONS] <input_file> [output_file]");
    println!("\nOptions:");
    println!("  -c, --compile <file>     Compile Rython file to assembly");
    println!("  -o, --output <file>      Specify output file name");
    println!("  -s, --stdin              Read from stdin instead of file");
    println!("  -a, --assembly           Output assembly code (default: NASM)");
    println!("  -b, --binary             Output binary/executable");
    println!("  -d, --debug              Enable debug output");
    println!("  -v, --verbose            Verbose output");
    println!("  -V, --version            Show version information");
    println!("  -h, --help               Show this help message");
    println!("\nBIOS Bootloader Options:");
    println!("  --bootloader             Generate bare bootloader (512-byte binary)");
    println!("  --bios-mode <file>       Compile to BIOS bootloader with graphics kernel");
    println!("  --opcode-mode <file>     Use opcode emitter instead of NASM");
    println!("\nExamples:");
    println!("  rython -c program.ry -o program.asm");
    println!("  rython --binary program.ry -o program.exe");
    println!("  cat program.ry | rython --stdin --assembly");
    println!("  rython --bios-mode program.ry -o boot.img");
}

fn print_version() {
    println!("Rython Compiler v{}", VERSION);
    println!("Copyright (c) 2024 {}", AUTHOR);
    println!("License: MIT");
    println!("Website: https://github.com/rython/rython");
    println!("\nCompiler features:");
    println!("  • Rython to NASM assembly compilation");
    println!("  • Rython to machine code via OpcodeEmitter");
    println!("  • Cross-platform (Windows x64, Linux x86_64)");
    println!("  • Reference counting for dynamic objects");
    println!("  • BIOS bootloader generation");
    println!("  • Real mode → Protected mode → Long mode transition");
}

fn compile_from_stdin() -> Result<(), String> {
    println!("Enter Rython code (press Ctrl+D or Ctrl+Z to finish):");
    
    let mut source_code = String::new();
    io::stdin().read_to_string(&mut source_code)
        .map_err(|e| format!("Failed to read from stdin: {}", e))?;
    
    if source_code.trim().is_empty() {
        return Err("No input provided".to_string());
    }
    
    // Generate a temporary output file name
    let output_file = String::from("output.bin");
    
    println!("Compiling...");
    let result = rython::compiler::compile_code(&source_code, &output_file);
    
    match result {
        Ok(_) => {
            println!("Compilation successful! Output: {}", output_file);
            
            // Display binary info
            if let Ok(metadata) = fs::metadata(&output_file) {
                println!("Binary size: {} bytes", metadata.len());
                
                // Show first few bytes
                if let Ok(content) = fs::read(&output_file) {
                    println!("\nFirst 32 bytes of binary:");
                    for (i, byte) in content.iter().take(32).enumerate() {
                        if i % 8 == 0 {
                            print!("\n{:04X}: ", i);
                        }
                        print!("{:02X} ", byte);
                    }
                    println!();
                }
            }
            Ok(())
        }
        Err(e) => {
            println!("Compilation failed: {}", e);
            Err(e)
        }
    }
}

fn compile_from_file(input_file: &str, output_file: Option<&str>, use_opcode_emitter: bool) -> Result<(), String> {
    let output = output_file.map(String::from).unwrap_or_else(|| {
        let base = Path::new(input_file)
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("output");
        if use_opcode_emitter {
            format!("{}.bin", base)
        } else {
            format!("{}.asm", base)
        }
    });
    
    println!("Compiling {} to {}...", input_file, output);
    
    if use_opcode_emitter {
        // Use opcode emitter
        let source_code = fs::read_to_string(input_file)
            .map_err(|e| format!("Failed to read input file: {}", e))?;
        
        let result = rython::compiler::compile_code(&source_code, &output);
        if result.is_ok() {
            println!("Opcode compilation successful!");
            
            // Show binary info
            if let Ok(metadata) = fs::metadata(&output) {
                println!("Binary size: {} bytes", metadata.len());
            }
        }
        result
    } else {
        // Use NASM emitter (legacy mode)
        let source_code = fs::read_to_string(input_file)
            .map_err(|e| format!("Failed to read input file: {}", e))?;
        
        // Parse the program
        let program = rython::parser::parse_program(&source_code)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        // Generate NASM assembly
        let asm_code = rython::emitter::compile_to_nasm(&program);
        fs::write(&output, asm_code)
            .map_err(|e| format!("Failed to write assembly: {}", e))?;
        
        println!("Assembly generation successful!");
        
        // Show first few lines
        if let Ok(content) = fs::read_to_string(&output) {
            println!("\nFirst 20 lines of assembly:");
            for (i, line) in content.lines().enumerate().take(20) {
                println!("{}", line);
                if i >= 19 {
                    println!("... (truncated)");
                    break;
                }
            }
        }
        
        Ok(())
    }
}

fn generate_bootloader(output_file: &str) -> Result<(), String> {
    println!("Generating bootloader to {}...", output_file);
    
    let bootloader_code = rython::bios::transition::create_hello_bootloader();
    fs::write(output_file, &bootloader_code)
        .map_err(|e| format!("Failed to write bootloader: {}", e))?;
    
    println!("Bootloader generated successfully!");
    println!("Size: {} bytes", bootloader_code.len());
    
    if bootloader_code.len() == 512 {
        println!("✓ Perfect 512-byte bootloader!");
    } else {
        println!("⚠ Warning: Bootloader is {} bytes (should be 512)", bootloader_code.len());
    }
    
    Ok(())
}

fn generate_bios_bootloader(input_file: &str, output_file: &str) -> Result<(), String> {
    println!("Generating BIOS bootloader from {} to {}...", input_file, output_file);
    
    let source_code = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read input file: {}", e))?;
    
    // This now uses the new emitter through compile_to_bios
    rython::compiler::compile_to_bios(&source_code, output_file)
}

fn assemble_and_link(input_file: &str, output_file: &str) -> Result<(), String> {
    println!("Assembling and linking {} to {}...", input_file, output_file);
    
    // First compile to assembly (legacy mode)
    let asm_file = format!("{}.asm", output_file);
    compile_from_file(input_file, Some(&asm_file), false)?;
    
    // Determine platform
    #[cfg(target_os = "windows")]
    let format = "win64";
    #[cfg(target_os = "linux")]
    let format = "elf64";
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    let format = "elf64";
    
    // Check for local NASM first
    let nasm = if cfg!(target_os = "windows") {
        if std::path::Path::new("bin/nasm.exe").exists() {
            "bin/nasm.exe"
        } else {
            "nasm.exe"
        }
    } else {
        if std::path::Path::new("bin/nasm").exists() {
            "bin/nasm"
        } else {
            "nasm"
        }
    };
    
    println!("Assembling with NASM (format: {}, binary: {})...", format, nasm);
    let output = Command::new(nasm)
        .args(&["-f", format, "-o", output_file, &asm_file])
        .output()
        .map_err(|e| format!("Failed to run NASM (tried '{}'): {}", nasm, e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("NASM assembly failed: {}", stderr));
    }
    
    // Clean up assembly file
    let _ = fs::remove_file(&asm_file);
    
    println!("Assembly and linking successful!");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return;
    }
    
    match args[1].as_str() {
        "-h" | "--help" => {
            print_help();
        }
        "-V" | "--version" => {
            print_version();
        }
        "-s" | "--stdin" => {
            if let Err(e) = compile_from_stdin() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "-c" | "--compile" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                print_help();
                std::process::exit(1);
            }
            
            let input_file = &args[2];
            let output_file = args.get(3).map(|s| s.as_str());
            let use_opcode_emitter = args.contains(&"--opcode".to_string());
            
            if let Err(e) = compile_from_file(input_file, output_file, use_opcode_emitter) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "-b" | "--binary" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                print_help();
                std::process::exit(1);
            }
            
            let input_file = &args[2];
            let output_file = args.get(3).cloned().unwrap_or_else(|| "output.exe".to_string());
            
            if let Err(e) = assemble_and_link(input_file, &output_file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "--bootloader" => {
            let output_file = args.get(2).cloned().unwrap_or_else(|| "bootloader.bin".to_string());
            
            if let Err(e) = generate_bootloader(&output_file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "--bios-mode" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                print_help();
                std::process::exit(1);
            }
            
            let input_file = &args[2];
            let output_file = args.get(3).cloned().unwrap_or_else(|| "bios_boot.img".to_string());
            
            if let Err(e) = generate_bios_bootloader(input_file, &output_file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "--opcode-mode" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                print_help();
                std::process::exit(1);
            }
            
            let input_file = &args[2];
            let output_file = args.get(3).map(|s| s.as_str());
            
            if let Err(e) = compile_from_file(input_file, output_file, true) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            // Try to treat as input file with opcode emitter by default
            let input_file = &args[1];
            let output_file = args.get(2).map(|s| s.as_str());
            
            if let Err(e) = compile_from_file(input_file, output_file, true) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}