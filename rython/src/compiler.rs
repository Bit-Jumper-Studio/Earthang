// src/compiler.rs - UPDATED to use the new NasmEmitter
//! Rython Compiler - Now using NASM with proper emitter

use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    BIOS,      // 512-byte bootloader
    Linux64,   // Linux ELF64
    Windows64, // Windows PE64
}

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub target: Target,
    pub verbose: bool,
    pub keep_assembly: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            target: Target::Linux64,
            verbose: false,
            keep_assembly: false,
        }
    }
}

pub struct RythonCompiler {
    config: CompilerConfig,
    nasm_emitter: crate::emitter::NasmEmitter,
}

impl RythonCompiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self {
            config,
            nasm_emitter: crate::emitter::NasmEmitter::new(),
        }
    }
    
    /// Main compilation entry point - USES THE NEW EMITTER
    pub fn compile(&mut self, source: &str, output_path: &str) -> Result<(), String> {
        // Parse source code
        let program = crate::parser::parse_program(source)
            .map_err(|e| format!("Parse error: {}", e))?;
        
        // Set target on emitter
        match self.config.target {
            Target::BIOS => self.nasm_emitter.set_target_bios(),
            Target::Linux64 => self.nasm_emitter.set_target_linux(),
            Target::Windows64 => self.nasm_emitter.set_target_windows(),
        }
        
        // Generate assembly using the new emitter
        let asm = self.nasm_emitter.compile_program(&program);
        
        if self.config.verbose {
            println!("[Rython] Generated assembly (first 50 lines):");
            for (i, line) in asm.lines().take(50).enumerate() {
                println!("{:3}: {}", i + 1, line);
            }
        }
        
        let asm_file = format!("{}.asm", output_path);
        
        // Write assembly
        fs::write(&asm_file, &asm)
            .map_err(|e| format!("Failed to write assembly: {}", e))?;
        
        // Assemble based on target
        match self.config.target {
            Target::BIOS => self.assemble_bios(&asm_file, output_path),
            Target::Linux64 => self.assemble_linux(&asm_file, output_path),
            Target::Windows64 => self.assemble_windows(&asm_file, output_path),
        }?;
        
        if !self.config.keep_assembly {
            fs::remove_file(&asm_file)
                .map_err(|e| format!("Failed to remove assembly file: {}", e))?;
        }
        
        Ok(())
    }
    
    fn assemble_bios(&self, asm_file: &str, output_path: &str) -> Result<(), String> {
        let nasm = crate::utils::find_nasm();
        
        Command::new(&nasm)
            .arg("-f")
            .arg("bin")
            .arg("-o")
            .arg(output_path)
            .arg(asm_file)
            .status()
            .map_err(|e| format!("Failed to run NASM: {}", e))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err("NASM assembly failed".to_string())
                }
            })
    }
    
    fn assemble_linux(&self, asm_file: &str, output_path: &str) -> Result<(), String> {
        let nasm = crate::utils::find_nasm();
        let obj_file = format!("{}.o", output_path);
        
        // Assemble with NASM
        Command::new(&nasm)
            .arg("-f")
            .arg("elf64")
            .arg("-o")
            .arg(&obj_file)
            .arg(asm_file)
            .status()
            .map_err(|e| format!("Failed to run NASM: {}", e))
            .and_then(|status| {
                if !status.success() {
                    return Err("NASM assembly failed".to_string());
                }
                Ok(())
            })?;
        
        // Link
        let result = Command::new("ld")
            .arg("-o")
            .arg(output_path)
            .arg(&obj_file)
            .status();
        
        match result {
            Ok(status) if status.success() => {
                fs::remove_file(&obj_file).ok();
                Ok(())
            }
            _ => {
                // Try manual linking
                crate::linker::manual_link(&obj_file, output_path)?;
                fs::remove_file(&obj_file).ok();
                Ok(())
            }
        }
    }
    
    fn assemble_windows(&self, asm_file: &str, output_path: &str) -> Result<(), String> {
        let nasm = crate::utils::find_nasm();
        let obj_file = format!("{}.obj", output_path);
        
        Command::new(&nasm)
            .arg("-f")
            .arg("win64")
            .arg("-o")
            .arg(&obj_file)
            .arg(asm_file)
            .status()
            .map_err(|e| format!("Failed to run NASM: {}", e))
            .and_then(|status| {
                if !status.success() {
                    return Err("NASM assembly failed".to_string());
                }
                Ok(())
            })?;
        
        // Try manual linking first
        match crate::linker::manual_link(&obj_file, output_path) {
            Ok(_) => {
                fs::remove_file(&obj_file).ok();
                Ok(())
            }
            Err(e) => {
                println!("Manual linking failed: {}", e);
                // Fallback
                let result = Command::new("link")
                    .arg("/subsystem:console")
                    .arg("/entry:main")
                    .arg(&obj_file)
                    .arg(format!("/out:{}", output_path))
                    .status();
                
                result.map_err(|e| format!("Linking failed: {}", e))
                    .and_then(|status| {
                        if status.success() {
                            fs::remove_file(&obj_file).ok();
                            Ok(())
                        } else {
                            Err("All linking attempts failed".to_string())
                        }
                    })
            }
        }
    }
}

// ========== PUBLIC API FUNCTIONS ==========

pub fn compile_to_bios(source: &str, output_path: &str) -> Result<(), String> {
    let config = CompilerConfig {
        target: Target::BIOS,
        verbose: true,
        keep_assembly: true,  // Keep for debugging
    };
    
    let mut compiler = RythonCompiler::new(config);
    compiler.compile(source, output_path)
}

pub fn compile_to_linux(source: &str, output_path: &str) -> Result<(), String> {
    let config = CompilerConfig {
        target: Target::Linux64,
        verbose: true,
        ..Default::default()
    };
    
    let mut compiler = RythonCompiler::new(config);
    compiler.compile(source, output_path)
}

pub fn compile_to_windows(source: &str, output_path: &str) -> Result<(), String> {
    let config = CompilerConfig {
        target: Target::Windows64,
        verbose: true,
        ..Default::default()
    };
    
    let mut compiler = RythonCompiler::new(config);
    compiler.compile(source, output_path)
}

pub fn compile_to_bootloader(source_code: &str) -> Result<Vec<u8>, String> {
    let config = CompilerConfig {
        target: Target::BIOS,
        verbose: true,
        ..Default::default()
    };
    
    let mut compiler = RythonCompiler::new(config);
    
    // Parse the source code
    let program = crate::parser::parse_program(source_code)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    // Create temporary file
    let temp_file = "temp_boot.bin";
    compiler.nasm_emitter.set_target_bios();
    let asm = compiler.nasm_emitter.compile_program(&program);
    
    let asm_file = format!("{}.asm", temp_file);
    fs::write(&asm_file, &asm)
        .map_err(|e| format!("Failed to write assembly: {}", e))?;
    
    compiler.assemble_bios(&asm_file, temp_file)?;
    
    // Read binary back
    let binary = fs::read(temp_file)
        .map_err(|e| format!("Failed to read binary: {}", e))?;
    
    // Clean up
    fs::remove_file(temp_file).ok();
    fs::remove_file(&asm_file).ok();
    
    Ok(binary)
}

// Keep the original compile_code function for backward compatibility
pub fn compile_code<S: AsRef<str>>(source: S, output_path: S) -> Result<(), String> {
    let source_code = source.as_ref();
    let output_str = output_path.as_ref();
    
    let mut compiler = RythonCompiler::new(CompilerConfig::default());
    compiler.compile(source_code, output_str)
}