// src/builtins/modules.rs
pub trait Module {
    fn name(&self) -> &'static str;
    fn functions(&self) -> Vec<crate::builtins::BuiltinFunction>;
    fn constants(&self) -> Vec<(&'static str, String)>;
}

// For now, return empty implementations
pub fn all_modules() -> Vec<Box<dyn Module>> {
    vec![]
}