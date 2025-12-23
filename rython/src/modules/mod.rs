pub mod builtin_module;
pub mod math_module;
pub mod string_module;
pub mod io_module;
pub mod types_module;

use crate::builtins::BuiltinFunction;

pub trait Module {
    fn name(&self) -> &'static str;
    fn functions(&self) -> Vec<BuiltinFunction>;
    fn constants(&self) -> Vec<(&'static str, String)>;
}

pub fn all_modules() -> Vec<Box<dyn Module>> {
    vec![
        Box::new(builtin_module::BuiltinModule),
        Box::new(math_module::MathModule),
        Box::new(string_module::StringModule),
        Box::new(io_module::IOModule),
        Box::new(types_module::TypesModule),
    ]
}