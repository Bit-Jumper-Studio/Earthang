// src/cli/mod.rs
pub mod commands;
pub mod parser;

use colored::*;
use commands::CommandExecutor;

pub fn run() -> Result<(), String> {
    let cli = parser::Cli::parse();
    
    match cli.command {
        Some(command) => command.execute(),
        None => {
            print_help();
            Ok(())
        }
    }
}

fn print_help() {
    // Fixed width for the box
    let box_width = 60;
    let separator = "─".repeat(box_width - 2); // -2 for border characters
    
    println!();
    println!("┌{}┐", separator.cyan());
    
    // Center the title
    let title = "RYTHON";
    let title_padding = (box_width - 2 - title.len()) / 2;
    println!("│{}{}{}│", 
        " ".repeat(title_padding),
        title.cyan().bold(),
        " ".repeat(box_width - 2 - title.len() - title_padding)
    );
    
    println!("├{}┤", separator.cyan());
    
    // Description lines
    print_box_line("A compiler from Rython to NASM assembly with mode", box_width, Color::White, false);
    print_box_line("transitions and bootloader generation.", box_width, Color::White, false);
    print_box_line("", box_width, Color::White, false);
    
    // USAGE section
    print_box_line("USAGE:", box_width, Color::Yellow, true);
    print_box_line("  rython <COMMAND> [OPTIONS]", box_width, Color::White, false);
    print_box_line("", box_width, Color::White, false);
    
    // COMMANDS section
    print_box_line("COMMANDS:", box_width, Color::Yellow, true);
    print_box_line("  compile     Compile Rython code to binary", box_width, Color::White, false);
    print_box_line("  generate    Generate standalone bootloaders", box_width, Color::White, false);
    print_box_line("  test        Test mode transitions and features", box_width, Color::White, false);
    print_box_line("  version     Show version information", box_width, Color::White, false);
    print_box_line("", box_width, Color::White, false);
    
    // EXAMPLES section
    print_box_line("EXAMPLES:", box_width, Color::Yellow, true);
    print_box_line("  rython compile program.ry -o output.bin", box_width, Color::Green, false);
    print_box_line("  rython generate compact -o boot.bin", box_width, Color::Green, false);
    print_box_line("  rython test transitions", box_width, Color::Green, false);
    print_box_line("", box_width, Color::White, false);
    
    // Footer
    print_box_line("Use '--help' with any command for detailed information.", box_width, Color::Blue, false);
    
    println!("└{}┘", separator.cyan());
    println!();
}

fn print_box_line(text: &str, box_width: usize, color: Color, bold: bool) {
    let padded_text = if text.is_empty() {
        " ".repeat(box_width - 2)
    } else {
        format!(" {:<width$}", text, width = box_width - 3)
    };
    
    let mut colored_text = padded_text.color(color);
    if bold {
        colored_text = colored_text.bold();
    }
    
    println!("│{}│", colored_text);
}

pub fn print_version() {
    let version = env!("CARGO_PKG_VERSION");
    let box_width = 60;
    let separator = "─".repeat(box_width - 2);
    
    println!();
    println!("┌{}┐", separator.cyan());
    
    // Center the title
    let title = format!("RYTHON BIT JUMPER SYSTEM v{}", version);
    let title_padding = (box_width - 2 - title.len()) / 2;
    println!("│{}{}{}│", 
        " ".repeat(title_padding),
        title.cyan().bold(),
        " ".repeat(box_width - 2 - title.len() - title_padding)
    );
    
    println!("├{}┤", separator.cyan());
    
    // Description
    print_box_line("A complete compiler with seamless mode transitions", box_width, Color::White, false);
    print_box_line("from 16-bit real mode to 64-bit long mode.", box_width, Color::White, false);
    print_box_line("", box_width, Color::White, false);
    
    // FEATURES section
    print_box_line("FEATURES:", box_width, Color::Yellow, true);
    
    let features = [
        "• 16-bit Real Mode bootloader (512 bytes)",
        "• 32-bit Protected Mode transition",
        "• 64-bit Long Mode transition",
        "• SSE/AVX/AVX-512 enabling",
        "• Ultra-compact bootloaders (256-512 bytes)",
        "• Complete mode transition ladder",
        "• Graphics kernel support",
        "• Multi-platform output",
    ];
    
    for feature in features.iter() {
        print_box_line(&format!("  {}", feature), box_width, Color::Green, false);
    }
    print_box_line("", box_width, Color::White, false);
    
    // TRANSITION PATH section
    print_box_line("TRANSITION PATH:", box_width, Color::Yellow, true);
    print_box_line("  16-bit Real Mode → A20 Gate → 32-bit Protected Mode", box_width, Color::White, false);
    print_box_line("  PAE Paging → 64-bit Long Mode → SSE/AVX/AVX-512", box_width, Color::White, false);
    print_box_line("", box_width, Color::White, false);
    
    // Footer
    print_box_line("AUTHORS: Rython Project - Bit Jumper Division", box_width, Color::Blue, false);
    print_box_line("LICENSE: MIT", box_width, Color::Blue, false);
    
    println!("└{}┘", separator.cyan());
    println!();
}