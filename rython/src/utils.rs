
use std::path::Path;
use std::process::Command;
use std::fs;

/// Tries to locate NASM in the system. 
/// We need this because different OS/Env setups have it in weird places.
pub fn find_nasm() -> String {
    // Just check if it's already in the PATH first. 
    // Most users will have it globally installed.
    if Command::new("nasm").arg("-v").output().is_ok() {
        return "nasm".to_string();
    }
    
    // Check common locations if 'which' or direct call fails
    let paths = [
        "/usr/bin/nasm",
        "/usr/local/bin/nasm",
        "bin/nasm",
        "bin/nasm.exe",
    ];
    
    for p in paths {
        if Path::new(p).exists() {
            return p.to_string();
        }
    }
    
    // Fallback to "nasm" and hope for the best, 
    // the caller will handle the error anyway.
    "nasm".to_string()
}

/// Verification tool to make sure the environment isn't broken
pub fn verify_nasm_installation() {
    println!("Checking NASM...");
    
    let bin = find_nasm();
    let check = Command::new(&bin).arg("-v").output();
    
    match check {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout);
            println!("✓ Found: {}", version.trim());
            
            // Quick smoke test: can it actually assemble a trivial file?
            // This catches cases where the binary exists but is for the wrong architecture.
            run_smoke_test(&bin);
        }
        _ => {
            eprintln!("✗ NASM not found or not working.");
            eprintln!("Install it via 'sudo apt install nasm' (Linux) or brew (macOS).");
        }
    }
}

fn run_smoke_test(nasm_path: &str) {
    let test_asm = "section .text\nglobal _start\n_start: mov eax, 1\nint 0x80";
    let tmp_src = "test_nasm.asm";
    let tmp_obj = "test_nasm.o";
    
    if fs::write(tmp_src, test_asm).is_err() {
        return; // Can't even write to disk, skip test
    }
    
    let status = Command::new(nasm_path)
        .args(&["-f", "elf64", "-o", tmp_obj, tmp_src])
        .status();
        
    let _ = fs::remove_file(tmp_src);
    if let Ok(s) = status {
        if s.success() {
            let _ = fs::remove_file(tmp_obj);
            println!("✓ NASM smoke test passed.");
        } else {
            println!("! NASM exists but failed to assemble a simple stub.");
        }
    }
}