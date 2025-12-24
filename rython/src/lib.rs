// src/lib.rs
pub mod parser;
pub mod emitter;
pub mod compiler;
pub mod linker;
pub mod utils;
pub mod bios;
pub mod cli;

// Re-export commonly used items
pub use compiler::*;
pub use emitter::*;
pub use parser::*;
pub use cli::*;