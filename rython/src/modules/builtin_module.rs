use super::Module;
use crate::builtins::BuiltinFunction;

pub struct BuiltinModule;

impl Module for BuiltinModule {
    fn name(&self) -> &'static str {
        "builtins"
    }

    fn functions(&self) -> Vec<BuiltinFunction> {
        vec![
            // Unified print function (like Python)
            BuiltinFunction::new(
                "print", 
                -1,  // Variable arguments
                false, 
                "rython_print", 
                vec!["..."]
            ),
            
            // Type checking
            BuiltinFunction::new(
                "type", 
                1, 
                true, 
                "rython_type", 
                vec!["any"]
            ),
            
            // Length function
            BuiltinFunction::new(
                "len", 
                1, 
                true, 
                "rython_len", 
                vec!["str"]
            ),
            
            // Range function (like Python's range)
            BuiltinFunction::new(
                "range", 
                -1,  // 1-3 arguments
                true, 
                "rython_range", 
                vec!["int", "int?", "int?"]
            ),
            
            // Boolean functions
            BuiltinFunction::new(
                "bool", 
                1, 
                true, 
                "rython_bool", 
                vec!["any"]
            ),
            
            // Integer conversion
            BuiltinFunction::new(
                "int", 
                1, 
                true, 
                "rython_int", 
                vec!["any"]
            ),
            
            // String conversion
            BuiltinFunction::new(
                "str", 
                1, 
                true, 
                "rython_str", 
                vec!["any"]
            ),
            
            // Float conversion
            BuiltinFunction::new(
                "float", 
                1, 
                true, 
                "rython_float", 
                vec!["any"]
            ),
        ]
    }

    fn constants(&self) -> Vec<(&'static str, String)> {
        vec![
            ("True", "1".to_string()),
            ("False", "0".to_string()),
            ("None", "0".to_string()),
        ]
    }
}