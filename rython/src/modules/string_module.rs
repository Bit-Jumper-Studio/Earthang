use super::Module;
use crate::builtins::BuiltinFunction;

pub struct StringModule;

impl Module for StringModule {
    fn name(&self) -> &'static str {
        "string"
    }

    fn functions(&self) -> Vec<BuiltinFunction> {
        vec![
            // String manipulation
            BuiltinFunction::new("upper", 1, true, "rython_upper", vec!["str"]),
            BuiltinFunction::new("lower", 1, true, "rython_lower", vec!["str"]),
            BuiltinFunction::new("strip", 1, true, "rython_strip", vec!["str"]),
            BuiltinFunction::new("lstrip", 1, true, "rython_lstrip", vec!["str"]),
            BuiltinFunction::new("rstrip", 1, true, "rython_rstrip", vec!["str"]),
            
            // String searching
            BuiltinFunction::new("find", 2, true, "rython_find", vec!["str", "str"]),
            BuiltinFunction::new("replace", 3, true, "rython_replace", vec!["str", "str", "str"]),
            BuiltinFunction::new("split", 2, true, "rython_split", vec!["str", "str"]),
            BuiltinFunction::new("join", 2, true, "rython_join", vec!["str", "list"]),
            
            // String testing
            BuiltinFunction::new("startswith", 2, true, "rython_startswith", vec!["str", "str"]),
            BuiltinFunction::new("endswith", 2, true, "rython_endswith", vec!["str", "str"]),
            BuiltinFunction::new("isdigit", 1, true, "rython_isdigit", vec!["str"]),
            BuiltinFunction::new("isalpha", 1, true, "rython_isalpha", vec!["str"]),
            
            // Formatting
            BuiltinFunction::new("format", -1, true, "rython_format", vec!["str", "any..."]),
        ]
    }

    fn constants(&self) -> Vec<(&'static str, String)> {
        vec![]
    }
}