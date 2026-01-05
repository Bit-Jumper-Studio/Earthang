use std::collections::HashMap;
use crate::parser::Statement;
use crate::rcl_compiler::{RclLibrary, RclEntry, RclFunction, RclVariable, RclMetadata, RclConstant};

/// Represents a module that can be converted to RCL
pub struct Module {
    pub name: String,
    pub functions: HashMap<String, (Vec<String>, Vec<Statement>, String)>,
    pub variables: HashMap<String, (String, String)>, // (value, type)
    pub constants: HashMap<String, (String, String)>, // (value, type)
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            constants: HashMap::new(),
        }
    }
    
    pub fn add_function(&mut self, name: &str, args: Vec<String>, body: Vec<Statement>, return_type: &str) {
        self.functions.insert(name.to_string(), (args, body, return_type.to_string()));
    }
    
    pub fn add_variable(&mut self, name: &str, value: &str, type_name: &str) {
        self.variables.insert(name.to_string(), (value.to_string(), type_name.to_string()));
    }
    
    pub fn add_constant(&mut self, name: &str, value: &str, type_name: &str) {
        self.constants.insert(name.to_string(), (value.to_string(), type_name.to_string()));
    }
    
    pub fn to_rcl_library(&self, target: &str) -> RclLibrary {
        let metadata = RclMetadata {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            author: Some("Rython Module".to_string()),
            description: Some(format!("Module: {}", self.name)),
            dependencies: Vec::new(),
            exports: self.get_all_exports(),
            target: target.to_string(),
            rcl_version: "1.0.0".to_string(),
            capabilities: Vec::new(),
        };
        
        let mut entries = Vec::new();
        
        // Convert functions - FIXED: removed ast and assembly fields
        for (name, (args, _, return_type)) in &self.functions {
            let params: Vec<(String, String)> = args.iter()
                .map(|arg| (arg.clone(), "auto".to_string()))
                .collect();
            
            entries.push(RclEntry::Function(RclFunction {
                name: name.clone(),
                signature: format!("{}({})", name, args.join(", ")),
                parameters: params,
                return_type: return_type.clone(),
                inlineable: false,
                pure: false,
                capabilities: Vec::new(),
            }));
        }
        
        // Convert variables
        for (name, (value, type_name)) in &self.variables {
            entries.push(RclEntry::Variable(RclVariable {
                name: name.clone(),
                value: value.clone(),
                type_name: type_name.clone(),
                constant: false,
                export: true,
            }));
        }
        
        // Convert constants
        for (name, (value, value_type)) in &self.constants {
            entries.push(RclEntry::Constant(RclConstant {
                name: name.clone(),
                value_type: value_type.clone(),
                value: value.clone(),
                export: true,
            }));
        }
        
        let mut symbol_table = HashMap::new();
        for entry in &entries {
            match entry {
                RclEntry::Function(f) => {
                    symbol_table.insert(f.name.clone(), "function".to_string());
                }
                RclEntry::Variable(v) => {
                    symbol_table.insert(v.name.clone(), "variable".to_string());
                }
                RclEntry::Constant(c) => {
                    symbol_table.insert(c.name.clone(), "constant".to_string());
                }
                _ => {}
            }
        }
        
        RclLibrary {
            metadata,
            entries,
            imports: Vec::new(),
            symbol_table,
        }
    }
    
    fn get_all_exports(&self) -> Vec<String> {
        let mut exports = Vec::new();
        
        exports.extend(self.functions.keys().cloned());
        exports.extend(self.variables.keys().cloned());
        exports.extend(self.constants.keys().cloned());
        
        exports
    }
}

/// Module registry
pub struct ModuleRegistry {
    modules: HashMap<String, Module>,
}

impl ModuleRegistry {
    pub fn default_registry() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        
        // Create some default modules
        let mut math = Module::new("math");
        math.add_function("sqrt", vec!["x".to_string()], vec![], "float");
        math.add_function("pow", vec!["base".to_string(), "exponent".to_string()], vec![], "float");
        math.add_constant("PI", "3.141592653589793", "float");
        math.add_constant("E", "2.718281828459045", "float");
        
        registry.register_module(math);
        registry
    }
    
    pub fn register_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }
    
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }
    
    pub fn extract_required_modules(&self, program: &crate::parser::Program) -> Vec<String> {
        let mut required = Vec::new();
        
        for stmt in &program.body {
            if let Statement::Expr(crate::parser::Expr::Call { func, args: _, kwargs: _, span: _ }) = stmt {
                if func == "import" || func == "from" {
                    // Simple module detection
                    for module_name in self.modules.keys() {
                        if func.contains(module_name) {
                            required.push(module_name.clone());
                        }
                    }
                }
            }
        }
        
        required
    }
}