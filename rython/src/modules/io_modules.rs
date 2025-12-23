use super::Module;
use crate::builtins::BuiltinFunction;

pub struct IOModule;

impl Module for IOModule {
    fn name(&self) -> &'static str {
        "io"
    }

    fn functions(&self) -> Vec<BuiltinFunction> {
        vec![
            // Input functions
            BuiltinFunction::new("input", 0, true, "rython_input", vec![]),
            BuiltinFunction::new("input", 1, true, "rython_input_prompt", vec!["str"]),
            
            // File operations
            BuiltinFunction::new("open", 1, true, "rython_open", vec!["str"]),
            BuiltinFunction::new("open", 2, true, "rython_open_mode", vec!["str", "str"]),
            
            // System
            BuiltinFunction::new("exit", 0, false, "rython_exit", vec![]),
            BuiltinFunction::new("exit", 1, false, "rython_exit_code", vec!["int"]),
        ]
    }

    fn constants(&self) -> Vec<(&'static str, String)> {
        vec![
            ("stdin", "0".to_string()),
            ("stdout", "1".to_string()),
            ("stderr", "2".to_string()),
        ]
    }
}