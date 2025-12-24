// src/cli/parser.rs
use clap::{Parser, ValueEnum};

use crate::cli::commands::Command;

/// Rython Compiler with Bit Jumper System
#[derive(Parser)]
#[command(
    name = "rython",
    version = env!("CARGO_PKG_VERSION"),
    author = "Rython Project - Bit Jumper System",
    about = "A compiler from Rython to NASM assembly with mode transitions",
    long_about = r#"
Rython Compiler - Bit Jumper System
====================================

A complete compiler from Rython to NASM assembly with mode transitions
and ultra-compact bootloader generation.

Features:
• 16-bit Real Mode bootloader (512 bytes)
• 32-bit Protected Mode transition  
• 64-bit Long Mode transition
• SSE/AVX/AVX-512 enabling
• Ultra-compact bootloaders (256-512 bytes)
• Complete mode transition ladder
"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
    
    /// Enable debug output
    #[arg(short, long, global = true)]
    pub debug: bool,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

impl Cli {
    pub fn parse() -> Self {
        Parser::parse()
    }
    
    pub fn print_help() {
        let mut cmd = <Cli as clap::CommandFactory>::command();
        cmd.print_help().unwrap();
    }
}

/// Compilation target architectures
#[derive(ValueEnum, Clone, Debug)]
pub enum Target {
    /// 16-bit real mode bootloader (512 bytes)
    Bios16,
    /// 32-bit protected mode bootloader
    Bios32,
    /// 64-bit long mode bootloader
    Bios64,
    /// 64-bit with SSE enabled
    Bios64Sse,
    /// 64-bit with AVX enabled
    Bios64Avx,
    /// 64-bit with AVX-512 enabled
    Bios64Avx512,
    /// Linux ELF64 executable
    Linux64,
    /// Windows PE64 executable
    Windows64,
}