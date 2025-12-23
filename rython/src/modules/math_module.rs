use super::Module;
use crate::builtins::BuiltinFunction;

pub struct MathModule;

impl Module for MathModule {
    fn name(&self) -> &'static str {
        "math"
    }

    fn functions(&self) -> Vec<BuiltinFunction> {
        vec![
            // Basic arithmetic
            BuiltinFunction::new("abs", 1, true, "rython_abs", vec!["int"]),
            BuiltinFunction::new("pow", 2, true, "rython_pow", vec!["int", "int"]),
            BuiltinFunction::new("max", -1, true, "rython_max", vec!["int..."]),
            BuiltinFunction::new("min", -1, true, "rython_min", vec!["int..."]),
            BuiltinFunction::new("sum", 1, true, "rython_sum", vec!["list"]),
            
            // Rounding
            BuiltinFunction::new("round", 1, true, "rython_round", vec!["float"]),
            BuiltinFunction::new("floor", 1, true, "rython_floor", vec!["float"]),
            BuiltinFunction::new("ceil", 1, true, "rython_ceil", vec!["float"]),
            
            // Advanced math
            BuiltinFunction::new("sqrt", 1, true, "rython_sqrt", vec!["float"]),
            BuiltinFunction::new("sin", 1, true, "rython_sin", vec!["float"]),
            BuiltinFunction::new("cos", 1, true, "rython_cos", vec!["float"]),
            BuiltinFunction::new("tan", 1, true, "rython_tan", vec!["float"]),
            BuiltinFunction::new("log", 1, true, "rython_log", vec!["float"]),
            BuiltinFunction::new("exp", 1, true, "rython_exp", vec!["float"]),
            
            // Constants
            BuiltinFunction::new("pi", 0, true, "rython_pi", vec![]),
            BuiltinFunction::new("e", 0, true, "rython_e", vec![]),
        ]
    }

    fn constants(&self) -> Vec<(&'static str, String)> {
        vec![
            ("PI", "3.141592653589793".to_string()),
            ("E", "2.718281828459045".to_string()),
        ]
    }
}