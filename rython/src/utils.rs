use std::path::Path;
use std::process::Command;
use std::fs;

/// Find the NASM executable with proper OS detection
pub fn find_nasm() -> String {
    // First, try to find system NASM using multiple methods
    
    // Method 1: Check if 'nasm' command works directly
    if let Ok(output) = Command::new("nasm").arg("--version").output() {
        if output.status.success() {
            return "nasm".to_string();
        }
    }
    
    // Method 2: Use 'which' command to find NASM in PATH
    if let Ok(output) = Command::new("which").arg("nasm").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return path;
            }
        }
    }
    
    // Method 3: Check common Linux paths
    let common_paths = vec![
        "/usr/bin/nasm",
        "/usr/local/bin/nasm",
        "/bin/nasm",
    ];
    
    for path in common_paths {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }
    
    // Method 4: Check bin/ directory - but ONLY if it's a valid binary
    let bin_dir = Path::new("bin");
    if bin_dir.exists() {
        let candidates = if cfg!(target_os = "windows") {
            vec!["nasm.exe"]
        } else if cfg!(target_os = "macos") {
            vec!["nasm-macos", "nasm"]
        } else {
            vec!["nasm-linux", "nasm"]
        };
        
        for candidate in candidates {
            let path = bin_dir.join(candidate);
            if path.exists() {
                // Check if it's a valid executable, not a text file
                if is_valid_executable(&path) {
                    return path.to_string_lossy().to_string();
                }
            }
        }
    }
    
    // Final fallback
    if cfg!(target_os = "windows") {
        "nasm.exe".to_string()
    } else {
        "nasm".to_string()
    }
}

/// Check if a file is a valid executable
fn is_valid_executable(path: &Path) -> bool {
    // Check if file exists
    if !path.exists() {
        return false;
    }
    
    // Check if it's a file (not a directory)
    if let Ok(metadata) = fs::metadata(path) {
        if !metadata.is_file() {
            return false;
        }
    } else {
        return false;
    }
    
    // On Unix-like systems, check execute permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return false; // No execute permission
            }
        }
    }
    
    // Check if it's not a text file (at least not empty and not a Git LFS pointer)
    if let Ok(content) = fs::read_to_string(path) {
        // Check for Git LFS pointer
        if content.contains("version https://git-lfs.github.com/spec/v1") {
            return false;
        }
        // Check if it's a very small file (< 10 bytes) - unlikely to be NASM
        if content.len() < 10 && content.chars().all(|c| c.is_ascii()) {
            return false;
        }
    }
    
    // Try to run it with --version to see if it works
    if let Ok(output) = Command::new(path).arg("--version").output() {
        return output.status.success();
    }
    
    false
}

/// Get NASM version with detailed error reporting
pub fn get_nasm_version() -> Result<String, String> {
    let nasm_path = find_nasm();
    
    // Try multiple version flags
    let version_flags = ["--version", "-v", "-V"];
    
    for flag in version_flags.iter() {
        match Command::new(&nasm_path).arg(flag).output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if !stdout.trim().is_empty() {
                    return Ok(stdout.trim().to_string());
                } else if !stderr.trim().is_empty() {
                    return Ok(stderr.trim().to_string());
                }
            }
            Ok(output) => {
                // Command ran but returned non-zero
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    return Err(format!("NASM error: {}", stderr));
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(format!("NASM not found: {}", nasm_path));
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return Err(format!("Permission denied for NASM: {}", nasm_path));
                } else if e.kind() == std::io::ErrorKind::InvalidInput {
                    return Err(format!("Invalid NASM executable: {} (might be a text file)", nasm_path));
                }
                return Err(format!("Failed to run NASM: {} (error: {})", nasm_path, e));
            }
        }
    }
    
    Err(format!("Could not get version from NASM: {}", nasm_path))
}

/// Check if NASM can run
pub fn can_run_nasm() -> bool {
    get_nasm_version().is_ok()
}

/// Simple command to find linker
pub fn find_linker() -> String {
    // Try common linkers
    let linkers = if cfg!(target_os = "windows") {
        vec!["link.exe", "ld.exe"]
    } else {
        vec!["ld", "ld.lld"]
    };
    
    for linker in linkers {
        if Command::new(linker).arg("--version").output().is_ok() {
            return linker.to_string();
        }
    }
    
    // Fallback
    if cfg!(target_os = "windows") {
        "link.exe".to_string()
    } else {
        "ld".to_string()
    }
}

/// Quick test for NASM
pub fn test_nasm_installation() {
    println!("Testing NASM installation...");
    
    match get_nasm_version() {
        Ok(version) => {
            println!("✓ NASM found: {}", version);
            
            // Try to assemble a simple test
            let test_asm = r#"BITS 64
section .text
global _start
_start:
    mov eax, 60
    xor edi, edi
    syscall"#;
            
            let test_file = "/tmp/test_nasm.asm";
            let output_file = "/tmp/test_nasm.o";
            
            if fs::write(test_file, test_asm).is_ok() {
                let nasm_path = find_nasm();
                let output = Command::new(&nasm_path)
                    .arg("-f")
                    .arg("elf64")
                    .arg("-o")
                    .arg(output_file)
                    .arg(test_file)
                    .output();
                
                // Clean up
                let _ = fs::remove_file(test_file);
                let _ = fs::remove_file(output_file);
                
                match output {
                    Ok(output) if output.status.success() => {
                        println!("✓ NASM can assemble successfully");
                    }
                    Ok(output) => {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("✗ NASM assembly failed: {}", error);
                    }
                    Err(e) => {
                        println!("✗ Failed to run NASM: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ NASM error: {}", e);
            println!("  Install NASM with: sudo apt install nasm");
            println!("  Or remove broken bin/nasm file: rm -f bin/nasm");
        }
    }
}