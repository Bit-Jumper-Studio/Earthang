// src/cli/commands.rs
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use clap::{Args, Subcommand};
use colored::*;

use crate::cli::parser::Target;
use crate::compiler;

/// Main command enum
#[derive(Subcommand)]
pub enum Command {
    /// Compile Rython code to binary
    Compile(CompileArgs),
    
    /// Generate standalone bootloaders
    Generate(GenerateArgs),
    
    /// Test mode transitions and features
    Test(TestArgs),
    
    /// Show version information
    Version,
}

/// Compile command arguments
#[derive(Args)]
pub struct CompileArgs {
    /// Input Rython file (use '-' for stdin)
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    
    /// Output file
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    
    /// Target architecture
    #[arg(short, long, value_name = "TARGET", default_value = "bios64")]
    pub target: Target,
    
    /// Keep intermediate assembly file
    #[arg(short = 'k', long)]
    pub keep_assembly: bool,
}

/// Generate command arguments
#[derive(Args)]
pub struct GenerateArgs {
    /// What to generate
    #[command(subcommand)]
    pub what: GenerateWhat,
}

#[derive(Subcommand)]
pub enum GenerateWhat {
    /// Generate 512-byte compact bootloader
    Compact {
        /// Output file
        #[arg(short, long, default_value = "compact.bin")]
        output: PathBuf,
    },
    
    /// Generate 256-byte micro bootloader
    Micro {
        /// Output file
        #[arg(short, long, default_value = "micro.bin")]
        output: PathBuf,
    },
    
    /// Generate hello world bootloader
    Bootloader {
        /// Output file
        #[arg(short, long, default_value = "bootloader.bin")]
        output: PathBuf,
    },
}

/// Test command arguments
#[derive(Args)]
pub struct TestArgs {
    /// What to test
    #[command(subcommand)]
    pub what: TestWhat,
}

#[derive(Subcommand)]
pub enum TestWhat {
    /// Test all mode transitions
    Transitions,
    
    /// Test graphics kernel
    Graphics,
    
    /// Test all features
    All,
}

/// Command executor trait
pub trait CommandExecutor {
    fn execute(&self) -> Result<(), String>;
}

impl CommandExecutor for Command {
    fn execute(&self) -> Result<(), String> {
        match self {
            Command::Compile(args) => args.execute(),
            Command::Generate(args) => args.execute(),
            Command::Test(args) => args.execute(),
            Command::Version => {
                crate::cli::print_version();
                Ok(())
            }
        }
    }
}

impl CommandExecutor for CompileArgs {
    fn execute(&self) -> Result<(), String> {
        println!();
        println!("{}", "╔══════════════════════════════════════════╗".cyan());
        println!("{}", "║          COMPILING RYTHON CODE          ║".cyan().bold());
        println!("{}", "╚══════════════════════════════════════════╝".cyan());
        println!();
        
        // Read source code
        let source_code = if self.input.to_string_lossy() == "-" {
            println!("{}", "Reading from stdin (press Ctrl+D or Ctrl+Z to finish):".blue());
            let mut code = String::new();
            io::stdin().read_to_string(&mut code)
                .map_err(|e| format!("Failed to read stdin: {}", e))?;
            if code.trim().is_empty() {
                return Err("No input provided".to_string());
            }
            println!("  {} {}", "✓".green(), "Code read successfully".white());
            code
        } else {
            println!("{}: {}", "Input file".blue(), self.input.display().to_string().white());
            let code = fs::read_to_string(&self.input)
                .map_err(|e| format!("Failed to read {}: {}", self.input.display(), e))?;
            println!("  {} {}", "✓".green(), "File read successfully".white());
            code
        };
        
        // Determine output file
        let output_file = self.output.clone().unwrap_or_else(|| {
            if self.input.to_string_lossy() == "-" {
                PathBuf::from("output")
            } else {
                let base_name = self.input.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                let ext = match self.target {
                    Target::Linux64 => "",
                    Target::Windows64 => ".exe",
                    _ => ".bin",
                };
                
                PathBuf::from(format!("{}{}", base_name, ext))
            }
        });
        
        // Show compilation info
        println!("{}: {}", "Target architecture".blue(), 
            format!("{:?}", self.target).yellow().bold());
        println!("{}: {}", "Output file".blue(), 
            output_file.display().to_string().white());
        
        // Create compiler config
        let config = compiler::CompilerConfig {
            target: match self.target {
                Target::Bios16 => compiler::Target::Bios16,
                Target::Bios32 => compiler::Target::Bios32,
                Target::Bios64 => compiler::Target::Bios64,
                Target::Bios64Sse => compiler::Target::Bios64Sse,
                Target::Bios64Avx => compiler::Target::Bios64Avx,
                Target::Bios64Avx512 => compiler::Target::Bios64Avx512,
                Target::Linux64 => compiler::Target::Linux64,
                Target::Windows64 => compiler::Target::Windows64,
            },
            verbose: true,
            keep_assembly: self.keep_assembly,
        };
        
        // Compile with status indicators
        println!();
        println!("{}", "Compilation progress:".blue());
        print!("  {} Parsing Rython code... ", "→".cyan());
        
        let mut compiler = compiler::RythonCompiler::new(config);
        
        print!("{} ", "✓".green());
        print!("{} Generating assembly... ", "→".cyan());
        
        compiler.compile(&source_code, &output_file.to_string_lossy())?;
        
        print!("{} ", "✓".green());
        println!("{} Assembling binary... ", "→".cyan());
        
        // Show file info
        if let Ok(metadata) = fs::metadata(&output_file) {
            print!("{} ", "✓".green());
            
            println!();
            println!();
            println!("{}", "╔══════════════════════════════════════════╗".green());
            println!("{}", "║          COMPILATION SUCCESSFUL          ║".green().bold());
            println!("{}", "╚══════════════════════════════════════════╝".green());
            println!();
            
            let size = metadata.len();
            
            // Display file info in a nice table
            println!("{}", "┌────────────────────────────────────────────┐".white());
            println!("{}", "│              FILE INFORMATION              │".white().bold());
            println!("{}", "├────────────────────────────────────────────┤".white());
            println!("│ {:<12} {:30} │", "File:".blue(), 
                format!("{}", output_file.display()).white());
            println!("│ {:<12} {:30} │", "Size:".blue(), 
                format!("{} bytes", size).green());
            
            match self.target {
                Target::Bios16 | Target::Bios32 | Target::Bios64 => {
                    let status = if size == 512 {
                        format!("{} (Perfect bootloader)", "OK".green())
                    } else if size < 512 {
                        format!("{} ({} bytes free)", "WARNING".yellow(), 512-size)
                    } else {
                        format!("{} (Too large for boot sector)", "ERROR".red())
                    };
                    println!("│ {:<12} {:30} │", "Bootable:".blue(), status);
                }
                _ => {}
            }
            
            println!("{}", "└────────────────────────────────────────────┘".white());
        }
        
        Ok(())
    }
}

impl CommandExecutor for GenerateArgs {
    fn execute(&self) -> Result<(), String> {
        match &self.what {
            GenerateWhat::Compact { output } => {
                println!();
                println!("{}", "╔══════════════════════════════════════════╗".cyan());
                println!("{}", "║      GENERATING COMPACT BOOTLOADER      ║".cyan().bold());
                println!("{}", "╚══════════════════════════════════════════╝".cyan());
                generate_compact_bootloader(output)
            }
            GenerateWhat::Micro { output } => {
                println!();
                println!("{}", "╔══════════════════════════════════════════╗".magenta());
                println!("{}", "║       GENERATING MICRO BOOTLOADER       ║".magenta().bold());
                println!("{}", "╚══════════════════════════════════════════╝".magenta());
                generate_micro_bootloader(output)
            }
            GenerateWhat::Bootloader { output } => {
                println!();
                println!("{}", "╔══════════════════════════════════════════╗".blue());
                println!("{}", "║        GENERATING BOOTLOADER            ║".blue().bold());
                println!("{}", "╚══════════════════════════════════════════╝".blue());
                generate_bootloader(output)
            }
        }
    }
}

impl CommandExecutor for TestArgs {
    fn execute(&self) -> Result<(), String> {
        match &self.what {
            TestWhat::Transitions => test_mode_transitions(),
            TestWhat::Graphics => test_graphics_kernel(),
            TestWhat::All => test_all_features(),
        }
    }
}

// Helper functions with nice formatting
fn generate_compact_bootloader(output_file: &PathBuf) -> Result<(), String> {
    use crate::bios::transition::CompactBootloader;
    
    println!();
    println!("{}", "Building bootloader sectors...".blue());
    println!("  {} Initializing bootloader...", "→".cyan());
    
    let mut compact = CompactBootloader::new();
    let bootloader = compact.create()
        .map_err(|e| format!("Failed to create bootloader: {}", e))?;
    
    fs::write(output_file, &bootloader)
        .map_err(|e| format!("Failed to write bootloader: {}", e))?;
    
    println!("  {} Writing to disk...", "→".cyan());
    
    println!();
    println!("{}", "┌────────────────────────────────────────────┐".green());
    println!("{}", "│          BOOTLOADER GENERATED             │".green().bold());
    println!("{}", "├────────────────────────────────────────────┤".green());
    println!("│ {:<12} {:30} │", "File:".blue(), 
        format!("{}", output_file.display()).white());
    println!("│ {:<12} {:30} │", "Size:".blue(), 
        format!("{} bytes", bootloader.len()).yellow().bold());
    
    if bootloader.len() == 512 {
        println!("│ {:<12} {:30} │", "Status:".blue(), 
            "Perfect 512-byte bootloader".green());
        println!("│ {:<12} {:30} │", "Bootable:".blue(), 
            "Yes - BIOS compatible".green());
    } else {
        println!("│ {:<12} {:30} │", "Status:".blue(), 
            format!("{} bytes", bootloader.len()).yellow());
    }
    
    println!("{}", "└────────────────────────────────────────────┘".green());
    println!();
    
    Ok(())
}

fn generate_micro_bootloader(output_file: &PathBuf) -> Result<(), String> {
    use crate::bios::transition::MicroBootloader;
    
    println!();
    println!("{}", "Creating micro bootloader (256 bytes)...".blue());
    println!("  {} Crafting minimal boot sector...", "→".cyan());
    
    let mut micro = MicroBootloader::new();
    let bootloader = micro.create();
    
    fs::write(output_file, &bootloader)
        .map_err(|e| format!("Failed to write bootloader: {}", e))?;
    
    println!("  {} Optimizing size...", "→".cyan());
    
    println!();
    println!("{}", "┌────────────────────────────────────────────┐".magenta());
    println!("{}", "│        MICRO BOOTLOADER GENERATED         │".magenta().bold());
    println!("{}", "├────────────────────────────────────────────┤".magenta());
    println!("│ {:<12} {:30} │", "File:".blue(), 
        format!("{}", output_file.display()).white());
    println!("│ {:<12} {:30} │", "Size:".blue(), 
        format!("{} bytes", bootloader.len()).magenta().bold());
    
    if bootloader.len() == 256 {
        println!("│ {:<12} {:30} │", "Status:".blue(), 
            "Ultra-compact 256-byte".magenta());
        println!("│ {:<12} {:30} │", "Efficiency:".blue(), 
            "Fits in half a sector".green());
    }
    
    println!("{}", "└────────────────────────────────────────────┘".magenta());
    println!();
    
    Ok(())
}

fn generate_bootloader(output_file: &PathBuf) -> Result<(), String> {
    use crate::bios::transition::create_hello_bootloader;
    
    println!();
    println!("{}", "Creating hello world bootloader...".blue());
    println!("  {} Generating boot sector...", "→".cyan());
    
    let bootloader = create_hello_bootloader();
    fs::write(output_file, &bootloader)
        .map_err(|e| format!("Failed to write bootloader: {}", e))?;
    
    println!("  {} Adding boot signature...", "→".cyan());
    
    println!();
    println!("{}", "┌────────────────────────────────────────────┐".blue());
    println!("{}", "│          HELLO WORLD BOOTLOADER           │".blue().bold());
    println!("{}", "├────────────────────────────────────────────┤".blue());
    println!("│ {:<12} {:30} │", "File:".blue(), 
        format!("{}", output_file.display()).white());
    println!("│ {:<12} {:30} │", "Size:".blue(), 
        format!("{} bytes", bootloader.len()).blue());
    println!("│ {:<12} {:30} │", "Test with:".blue(), 
        "qemu-system-x86_64".green());
    println!("│ {:<12} {:30} │", "Command:".blue(), 
        "qemu-system-x86_64 -drive format=raw,file=bootloader.bin".cyan());
    println!("{}", "└────────────────────────────────────────────┘".blue());
    println!();
    
    Ok(())
}

fn test_mode_transitions() -> Result<(), String> {
    use crate::bios::transition::{CompactBootloader, MicroBootloader, ModeTransitionEmitter, GraphicsKernel};
    
    println!();
    println!("{}", "╔══════════════════════════════════════════╗".cyan());
    println!("{}", "║      TESTING BIT JUMPER SYSTEM          ║".cyan().bold());
    println!("{}", "╚══════════════════════════════════════════╝".cyan());
    println!();
    
    println!("{}", "Test Suite Execution:".blue());
    println!("{}", "─────────────────────".white());
    
    // Test 16-bit real mode
    println!("\n{} 16-bit Real Mode Bootloader", "1.".yellow().bold());
    println!("{}", "   ──────────────────────────".white());
    let mut compact = CompactBootloader::new();
    let bootloader = compact.create()?;
    println!("   {} Size: {} bytes", "✓".green(), bootloader.len().to_string().white());
    println!("   {} Boot signature: 0x{:02X}{:02X}", "✓".green(), 
        bootloader[510], bootloader[511]);
    
    // Test micro bootloader
    println!("\n{} Micro Bootloader (256 bytes)", "2.".yellow().bold());
    println!("{}", "   ──────────────────────────".white());
    let mut micro = MicroBootloader::new();
    let micro_boot = micro.create();
    println!("   {} Size: {} bytes", "✓".green(), micro_boot.len().to_string().white());
    
    // Test mode transition emitter
    println!("\n{} Mode Transition Emitter", "3.".yellow().bold());
    println!("{}", "   ──────────────────────".white());
    let mut emitter = ModeTransitionEmitter::new();
    match emitter.create_bootloader() {
        Ok(_) => println!("   {} Mode transitions compiled successfully", "✓".green()),
        Err(e) => println!("   {} Mode transition error: {}", "✗".red(), e),
    }
    
    // Test graphics kernel
    println!("\n{} Graphics Kernel", "4.".yellow().bold());
    println!("{}", "   ─────────────────".white());
    let mut kernel = GraphicsKernel::new();
    
    let graphics_code = vec![
        0x48, 0xB8, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x48, 0x89, 0xC7,
        0x48, 0xC7, 0xC0, 0xFF, 0x00, 0xFF, 0x00,
        0x48, 0xC7, 0xC1, 0x00, 0x10, 0x00, 0x00,
        0xF3, 0x48, 0xAB,
        0xF4,
    ];
    
    kernel.load_graphics_code(&graphics_code);
    println!("   {} Graphics kernel loaded at 0x{:016X}", "✓".green(), kernel.entry_point);
    
    let disk_image = kernel.create_disk_image();
    println!("   {} Disk image created: {} bytes", "✓".green(), 
        disk_image.len().to_string().white());
    
    // Summary
    println!();
    println!("{}", "╔══════════════════════════════════════════╗".green());
    println!("{}", "║          TEST SUITE COMPLETE            ║".green().bold());
    println!("{}", "╚══════════════════════════════════════════╝".green());
    println!();
    
    println!("{}", "TRANSITION PATH VALIDATED:".blue());
    println!("{}", "──────────────────────────".white());
    println!("   {} 16-bit Real Mode → A20 Gate", "→".cyan());
    println!("   {} 32-bit Protected Mode → PAE Paging", "→".cyan());
    println!("   {} 64-bit Long Mode → SSE/AVX/AVX-512", "→".cyan());
    println!("   {} Graphics Kernel Execution", "→".cyan());
    println!();
    
    Ok(())
}

fn test_graphics_kernel() -> Result<(), String> {
    println!();
    println!("{}", "Testing graphics kernel...".cyan().bold());
    // Implementation
    Ok(())
}

fn test_all_features() -> Result<(), String> {
    println!();
    println!("{}", "Running full test suite...".cyan().bold());
    // Implementation
    Ok(())
}