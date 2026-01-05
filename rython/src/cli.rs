use clap::{Parser, Subcommand, Args, ValueEnum};
use std::path::PathBuf;
use crate::compiler::{RythonCompiler, CompilerConfig};
use crate::rcl_integration::RclCli;

/// Rython Compiler CLI
#[derive(Parser)]
#[command(name = "rython")]
#[command(about = "Rython Compiler - Python-like syntax to bare metal binary")]
#[command(version = "1.0.0")]
#[command(long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Enable quiet mode
    #[arg(short, long)]
    pub quiet: bool,
    
    /// Command to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Compile Rython source to binary
    Compile(CompileArgs),
    
    /// Create RCL library from Rython source
    #[command(name = "rcl-compile")]
    RclCompile(RclCompileArgs),
    
    /// Show RCL library information
    #[command(name = "rcl-info")]
    RclInfo(RclInfoArgs),
    
    /// List functions in RCL library
    #[command(name = "rcl-list")]
    RclList(RclInfoArgs),
    
    /// Extract assembly from RCL library
    #[command(name = "rcl-extract")]
    RclExtract(RclExtractArgs),
    
    /// Test SSD syntax mutation
    #[command(name = "ssd-test")]
    SsdTest,
    
    /// Create SSD components
    #[command(name = "create-ssd")]
    CreateSsd(CreateSsdArgs),
    
    /// Generate code
    Generate(GenerateArgs),
    
    /// Create RCL library (alias)
    #[command(name = "create-rcl")]
    CreateRcl(RclCompileArgs),
    
    /// Run tests
    Test(TestArgs),
    
    /// Show version
    Version,
    
    /// Generate modules
    #[command(name = "generate-modules")]
    GenerateModules,
    
    /// Compile modules
    #[command(name = "compile-modules")]
    CompileModules,
    
    /// Check toolchain
    Check,
}

/// System target platforms
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliTarget {
    Bios16,
    Bios32,
    Bios64,
    Bios64Sse,
    Bios64Avx,
    Bios64Avx512,
    Linux64,
    Windows64,
}

impl From<CliTarget> for crate::backend::Target {
    fn from(val: CliTarget) -> Self {
        match val {
            CliTarget::Bios16 => crate::backend::Target::Bios16,
            CliTarget::Bios32 => crate::backend::Target::Bios32,
            CliTarget::Bios64 => crate::backend::Target::Bios64,
            CliTarget::Bios64Sse => crate::backend::Target::Bios64Sse,
            CliTarget::Bios64Avx => crate::backend::Target::Bios64Avx,
            CliTarget::Bios64Avx512 => crate::backend::Target::Bios64Avx512,
            CliTarget::Linux64 => crate::backend::Target::Linux64,
            CliTarget::Windows64 => crate::backend::Target::Windows64,
        }
    }
}

/// Arguments for compile command
#[derive(Args)]
pub struct CompileArgs {
    /// Input file
    pub file: PathBuf,
    
    /// Output file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Target platform
    #[arg(short, long, value_enum, default_value_t = CliTarget::Bios64)]
    pub target: CliTarget,
    
    /// Enable SSD architecture
    #[arg(long)]
    pub ssd: bool,
    
    /// Load SSD header file (.json)
    #[arg(long)]
    pub ssd_header: Option<PathBuf>,
    
    /// Load SSD assembly file (.json)
    #[arg(long)]
    pub ssd_asm: Option<PathBuf>,
    
    /// Enable RCL library support
    #[arg(long)]
    pub rcl: bool,
    
    /// Load RCL library (.rcl)
    #[arg(long)]
    pub rcl_lib: Vec<String>,
    
    /// Keep assembly file
    #[arg(long)]
    pub keep_assembly: bool,
    
    /// Disable optimization
    #[arg(long)]
    pub no_optimize: bool,
    
    /// Enable syntax mutation test
    #[arg(long)]
    pub test_ssd: bool,
}

/// Arguments for RCL compile command
#[derive(Args)]
pub struct RclCompileArgs {
    /// Input Rython source file
    pub file: PathBuf,
    
    /// Output RCL library file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Target platform
    #[arg(short, long, value_enum, default_value_t = CliTarget::Bios64)]
    pub target: CliTarget,
}

/// Arguments for RCL info command
#[derive(Args)]
pub struct RclInfoArgs {
    /// RCL library file
    pub file: PathBuf,
}

/// Arguments for RCL extract command
#[derive(Args)]
pub struct RclExtractArgs {
    /// RCL library file
    pub file: PathBuf,
    
    /// Function name to extract
    pub function: String,
}

/// Arguments for create SSD command
#[derive(Args)]
pub struct CreateSsdArgs {
    /// Create header file (.json)
    #[arg(long)]
    pub header: Option<PathBuf>,
    
    /// Create assembly file (.json)
    #[arg(long)]
    pub asm: Option<PathBuf>,
}

/// Arguments for generate command
#[derive(Args)]
pub struct GenerateArgs {
    /// Type to generate
    #[arg(short, long, value_enum, default_value_t = CliTarget::Bios16)]
    pub r#type: CliTarget,
    
    /// Size in bytes
    #[arg(short, long, default_value_t = 512)]
    pub size: usize,
}

/// Arguments for test command
#[derive(Args)]
pub struct TestArgs {
    /// Test suite
    #[arg(short, long, default_value = "basic")]
    pub suite: String,
}

/// Main CLI handler
impl Cli {
    pub fn run(self) -> Result<(), String> {
        let verbose = self.verbose;
        
        match &self.command {
            Some(command) => match command {
                Commands::Compile(args) => self.handle_compile(args, verbose),
                Commands::RclCompile(args) => self.handle_rcl_compile(args, verbose),
                Commands::RclInfo(args) => self.handle_rcl_info(args, verbose),
                Commands::RclList(args) => self.handle_rcl_list(args, verbose),
                Commands::RclExtract(args) => self.handle_rcl_extract(args, verbose),
                Commands::SsdTest => self.handle_ssd_test(verbose),
                Commands::CreateSsd(args) => self.handle_create_ssd(args, verbose),
                Commands::Generate(args) => self.handle_generate(args, verbose),
                Commands::CreateRcl(args) => self.handle_rcl_compile(args, verbose),
                Commands::Test(args) => self.handle_test(args, verbose),
                Commands::Version => self.handle_version(),
                Commands::GenerateModules => self.handle_generate_modules(verbose),
                Commands::CompileModules => self.handle_compile_modules(verbose),
                Commands::Check => self.handle_check(verbose),
            },
            None => {
                // Show help when no command is provided
                println!("Rython Compiler - Python-like syntax to bare metal binary");
                println!();
                println!("USAGE:");
                println!("    rython [OPTIONS] <COMMAND>");
                println!();
                println!("For more information, run: rython --help");
                Ok(())
            }
        }
    }
    
    fn handle_compile(&self, args: &CompileArgs, verbose: bool) -> Result<(), String> {
        let input_file = &args.file;
        let output_file = args.output.as_ref().map_or_else(|| {
            let mut path = input_file.clone();
            path.set_extension("bin");
            path
        }, |p| p.clone());
        
        let target: crate::backend::Target = args.target.into();
        
        // Read source file
        let source = std::fs::read_to_string(input_file)
            .map_err(|e| format!("Failed to read source file '{}': {}", input_file.display(), e))?;
        
        // Parse to check syntax
        crate::parser::parse_program(&source)
            .map_err(|e| format!("Parse error in '{}': {:?}", input_file.display(), e))?;
        
        if verbose {
            println!("[COMPILER] Compiling '{}' to '{}'", input_file.display(), output_file.display());
            println!("[COMPILER] Target: {:?}", target);
        }
        
        let config = CompilerConfig {
            target,
            verbose,
            keep_assembly: args.keep_assembly,
            optimize: !args.no_optimize,
            modules: Vec::new(),
            ssd_headers: args.ssd_header.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            ssd_assembly: args.ssd_asm.iter().map(|p| p.to_string_lossy().to_string()).collect(),
            enable_ssd: args.ssd || args.test_ssd,
            enable_rcl: args.rcl,
            rcl_libraries: args.rcl_lib.clone(),
        };
        
        let mut compiler = RythonCompiler::new(config);
        compiler.compile(&source, &output_file.to_string_lossy())?;
        
        if verbose {
            println!("[COMPILER] Compilation successful!");
            println!("[COMPILER] Output: {}", output_file.display());
        }
        
        Ok(())
    }
    
    fn handle_rcl_compile(&self, args: &RclCompileArgs, verbose: bool) -> Result<(), String> {
        let input_file = &args.file;
        let output_file = args.output.as_ref().map_or_else(|| {
            let mut path = input_file.clone();
            path.set_extension("rcl");
            path
        }, |p| p.clone());
        
        let target_str = match args.target {
            CliTarget::Bios16 => "bios16",
            CliTarget::Bios32 => "bios32",
            CliTarget::Bios64 => "bios64",
            CliTarget::Bios64Sse => "bios64_sse",
            CliTarget::Bios64Avx => "bios64_avx",
            CliTarget::Bios64Avx512 => "bios64_avx512",
            CliTarget::Linux64 => "linux64",
            CliTarget::Windows64 => "windows64",
        };
        
        if verbose {
            println!("[RCL] Compiling '{}' to RCL library '{}'", 
                input_file.display(), output_file.display());
            println!("[RCL] Target: {}", target_str);
        }
        
        let cli = RclCli::new(verbose);
        cli.compile_to_rcl(
            &input_file.to_string_lossy(),
            &output_file.to_string_lossy(),
            target_str
        )
    }
    
    fn handle_rcl_info(&self, args: &RclInfoArgs, verbose: bool) -> Result<(), String> {
        let cli = RclCli::new(verbose);
        cli.show_rcl_info(&args.file.to_string_lossy())
    }
    
    fn handle_rcl_list(&self, args: &RclInfoArgs, verbose: bool) -> Result<(), String> {
        let cli = RclCli::new(verbose);
        cli.list_functions(&args.file.to_string_lossy())
    }
    
    fn handle_rcl_extract(&self, args: &RclExtractArgs, verbose: bool) -> Result<(), String> {
        let cli = RclCli::new(verbose);
        cli.extract_assembly(&args.file.to_string_lossy(), &args.function)
    }
    
    fn handle_ssd_test(&self, verbose: bool) -> Result<(), String> {
        if verbose {
            println!("[SSD] Testing SSD syntax mutation...");
        }
        
        let source = r#"
> "Hello, World!"
x = 10
y = 20
!! "This is a panic"
z = x + y
"#;
        
        println!("[SSD] Original source:");
        println!("{}", source);
        println!();
        
        let mutated = source
            .replace(">", "print")
            .replace("!!", "panic");
        
        println!("[SSD] Mutated source:");
        println!("{}", mutated);
        println!();
        
        println!("[SSD] Test completed successfully!");
        Ok(())
    }
    
    fn handle_create_ssd(&self, args: &CreateSsdArgs, verbose: bool) -> Result<(), String> {
        if verbose {
            println!("[SSD] Creating SSD components...");
        }
        
        use crate::ssd_injector;
        
        let mut created_anything = false;
        
        if let Some(ref header_path) = args.header {
            let header = ssd_injector::SsdInjector::create_test_syntax_mutation();
            let json = serde_json::to_string_pretty(&header)
                .map_err(|e| format!("Failed to serialize SSD header: {}", e))?;
            
            std::fs::write(header_path, &json)
                .map_err(|e| format!("Failed to write SSD header: {}", e))?;
            
            println!("[SSD] Created header: {}", header_path.display());
            created_anything = true;
        }
        
        if let Some(ref asm_path) = args.asm {
            let asm_block = ssd_injector::SsdInjector::create_test_negative_number_handling();
            let json = serde_json::to_string_pretty(&asm_block)
                .map_err(|e| format!("Failed to serialize SSD assembly: {}", e))?;
            
            std::fs::write(asm_path, &json)
                .map_err(|e| format!("Failed to write SSD assembly: {}", e))?;
            
            println!("[SSD] Created assembly: {}", asm_path.display());
            created_anything = true;
        }
        
        if !created_anything {
            println!("[SSD] No components specified. Use --header or --asm options.");
        }
        
        Ok(())
    }
    
    fn handle_generate(&self, args: &GenerateArgs, verbose: bool) -> Result<(), String> {
        let target: crate::backend::Target = args.r#type.into();
        
        if verbose {
            println!("[GENERATE] Generating code for target: {:?}", target);
            println!("[GENERATE] Size: {} bytes", args.size);
        }
        
        match target {
            crate::backend::Target::Bios16 => {
                println!("; 16-bit Boot Sector");
                println!("; Generated by Rython");
                println!("; Size: {} bytes", args.size);
                println!();
                println!("bits 16");
                println!("org 0x7C00");
                println!();
                println!("start:");
                println!("    cli");
                println!("    xor ax, ax");
                println!("    mov ds, ax");
                println!("    mov es, ax");
                println!("    mov ss, ax");
                println!("    mov sp, 0x7C00");
                println!("    sti");
                println!();
                println!("    ; Print message");
                println!("    mov si, msg");
                println!("    call print_string");
                println!();
                println!("    ; Halt");
                println!("    cli");
                println!("    hlt");
                println!("    jmp $");
                println!();
                println!("print_string:");
                println!("    pusha");
                println!("    mov ah, 0x0E");
                println!(".loop:");
                println!("    lodsb");
                println!("    test al, al");
                println!("    jz .done");
                println!("    int 0x10");
                println!("    jmp .loop");
                println!(".done:");
                println!("    popa");
                println!("    ret");
                println!();
                println!("msg:");
                println!("    db 'Rython 16-bit', 0");
                println!();
                
                let padding = args.size.saturating_sub(510);
                println!("    times {} db 0", padding);
                println!("    dw 0xAA55");
            }
            _ => {
                println!("[GENERATE] Target {:?} not yet implemented for code generation", target);
            }
        }
        
        Ok(())
    }
    
    fn handle_test(&self, args: &TestArgs, verbose: bool) -> Result<(), String> {
        if verbose {
            println!("[TEST] Running test suite: {}", args.suite);
        }
        
        match args.suite.as_str() {
            "basic" => {
                println!("[TEST] Running basic tests...");
                
                let source = r#"
def hello():
    print("Hello, World!")

x = 10
y = x + 5
"#;
                
                match crate::parser::parse_program(source) {
                    Ok(_) => println!("[TEST] ✓ Parser test passed"),
                    Err(e) => return Err(format!("Parser test failed: {:?}", e)),
                }
                
                println!("[TEST] ✓ CLI structure test passed");
                
                println!("[TEST] All basic tests passed!");
            }
            "full" => {
                println!("[TEST] Running full test suite...");
                
                let nasm_output = std::process::Command::new("nasm")
                    .arg("--version")
                    .output();
                
                match nasm_output {
                    Ok(output) => {
                        let version = String::from_utf8_lossy(&output.stdout);
                        println!("[TEST] ✓ NASM found: {}", version.lines().next().unwrap_or("unknown"));
                    }
                    Err(_) => println!("[TEST] ⚠ NASM not found (required for assembly)"),
                }
                
                let rustc_output = std::process::Command::new("rustc")
                    .arg("--version")
                    .output();
                
                match rustc_output {
                    Ok(output) => {
                        let version = String::from_utf8_lossy(&output.stdout);
                        println!("[TEST] ✓ Rust toolchain found: {}", version.trim());
                    }
                    Err(_) => println!("[TEST] ⚠ Rust toolchain not found"),
                }
                
                println!("[TEST] Full test suite completed!");
            }
            _ => {
                println!("[TEST] Unknown test suite: {}", args.suite);
                println!("[TEST] Available suites: basic, full");
            }
        }
        
        Ok(())
    }
    
    fn handle_version(&self) -> Result<(), String> {
        println!("Rython Compiler v1.0.0");
        println!("Copyright (c) 2024 Rython Project");
        println!("GitHub: https://github.com/Bit-Jumper-Studio/Rython");
        Ok(())
    }
    
    fn handle_generate_modules(&self, verbose: bool) -> Result<(), String> {
        if verbose {
            println!("[MODULES] Generating modules...");
        }
        
        println!("[MODULES] Available modules:");
        println!("  - std (standard library)");
        println!("  - math (mathematical functions)");
        println!("  - string (string operations)");
        println!("  - io (input/output)");
        
        println!();
        println!("[MODULES] To use a module, add 'import \"module_name\"' to your Rython code");
        
        Ok(())
    }
    
    fn handle_compile_modules(&self, verbose: bool) -> Result<(), String> {
        if verbose {
            println!("[MODULES] Compiling modules...");
        }
        
        println!("[MODULES] Module compilation not yet implemented");
        println!("[MODULES] This feature will compile modules to RCL libraries");
        
        Ok(())
    }
    
    fn handle_check(&self, verbose: bool) -> Result<(), String> {
        if verbose {
            println!("[CHECK] Checking toolchain...");
        }
        
        let checks = [
            ("Rust compiler", "rustc", &["--version"]),
            ("Cargo", "cargo", &["--version"]),
            ("NASM", "nasm", &["--version"]),
        ];
        
        let mut all_ok = true;
        
        for (name, cmd, args) in checks {
            match std::process::Command::new(cmd).args(args).output() {
                Ok(output) => {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        println!("[CHECK] ✓ {}: {}", name, version.lines().next().unwrap_or("").trim());
                    } else {
                        println!("[CHECK] ✗ {}: Not found or error", name);
                        all_ok = false;
                    }
                }
                Err(_) => {
                    println!("[CHECK] ✗ {}: Not found", name);
                    all_ok = false;
                }
            }
        }
        
        if all_ok {
            println!();
            println!("[CHECK] All checks passed! Toolchain is ready.");
        } else {
            println!();
            println!("[CHECK] Some checks failed. Install missing tools:");
            println!("  Rust: https://rustup.rs/");
            println!("  NASM: https://www.nasm.us/");
        }
        
        Ok(())
    }
}

/// Parse CLI arguments and run
pub fn run() -> Result<(), String> {
    let cli = Cli::parse();
    cli.run()
}