// src/builtins.rs - SIMPLIFIED VERSION
use std::collections::HashMap;
use lazy_static::lazy_static;

/// Built-in function information
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: &'static str,
    pub arg_count: i32,
    pub returns_value: bool,
    pub runtime_function: &'static str,
    pub arg_types: Vec<&'static str>,
}

impl BuiltinFunction {
    pub fn new(name: &'static str, arg_count: i32, returns_value: bool, 
               runtime_function: &'static str, arg_types: Vec<&'static str>) -> Self {
        BuiltinFunction {
            name,
            arg_count,
            returns_value,
            runtime_function,
            arg_types,
        }
    }
}

/// Registry of built-in functions
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunction>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut registry = BuiltinRegistry {
            functions: HashMap::new(),
        };
        
        // Math functions
        registry.functions.insert("add".to_string(), BuiltinFunction::new(
            "add", 2, true, "rython_add", vec!["int", "int"]
        ));
        
        registry.functions.insert("minus".to_string(), BuiltinFunction::new(
            "minus", 2, true, "rython_minus", vec!["int", "int"]
        ));
        
        registry.functions.insert("multiply".to_string(), BuiltinFunction::new(
            "multiply", 2, true, "rython_multiply", vec!["int", "int"]
        ));
        
        registry.functions.insert("divide".to_string(), BuiltinFunction::new(
            "divide", 2, true, "rython_divide", vec!["int", "int"]
        ));
        
        registry.functions.insert("fibonacci".to_string(), BuiltinFunction::new(
            "fibonacci", 1, true, "rython_fibonacci", vec!["int"]
        ));
        
        // Print functions
        registry.functions.insert("print_int".to_string(), BuiltinFunction::new(
            "print_int", 1, false, "rython_print_int", vec!["int"]
        ));
        
        registry.functions.insert("print_float".to_string(), BuiltinFunction::new(
            "print_float", 1, false, "rython_print_float", vec!["float"]
        ));
        
        registry.functions.insert("print_str".to_string(), BuiltinFunction::new(
            "print_str", 1, false, "rython_print_str", vec!["str"]
        ));
        
        registry.functions.insert("print_con".to_string(), BuiltinFunction::new(
            "print_con", -1, false, "rython_print_con", vec!["..."]
        ));
        
        registry
    }
    
    /// Check if a function is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// Get built-in function information
    pub fn get_builtin(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }
    
    /// Get all built-in function names
    pub fn get_all_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

// Global builtin registry
lazy_static! {
    pub static ref BUILTINS: BuiltinRegistry = BuiltinRegistry::new();
}