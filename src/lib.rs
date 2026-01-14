pub mod backend;
pub mod compiler;
pub mod disk_cache;
pub mod dsl;
pub mod emitter;
pub mod extension;
pub mod lua_frontend;
pub mod lua_pool;
pub mod cli;

pub use backend::{Backend, BackendRegistry, Target, Capability};
pub use compiler::{EarthangCompiler, CompilerConfig, compile, compile_with_hardware};
pub use lua_frontend::{parse_program, LuaFrontend};
pub use extension::{EarthngModule, AssemblyEmitter, BasicAssemblyEmitter, ExtensionRegistry, MathModule, StringModule, SystemModule};  // NEW

pub mod parser {
    pub use crate::lua_frontend::{
        Program, Statement, Expr, Position, Span, Op,
        parse_program, ParseError,
        CompareOp, BoolOp, UnaryOp
    };
}