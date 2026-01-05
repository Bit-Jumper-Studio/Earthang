use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use crate::parser::{Program, Statement, Expr};
use serde::{Serialize, Deserialize};

/// RCL File Format Version
const RCL_VERSION: &str = "1.0.0";

/// RCL Library Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub exports: Vec<String>,
    pub target: String,
    pub rcl_version: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Compiled Function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclFunction {
    pub name: String,
    pub signature: String,
    #[serde(default)]
    pub parameters: Vec<(String, String)>,
    pub return_type: String,
    pub inlineable: bool,
    pub pure: bool,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Compiled Variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclVariable {
    pub name: String,
    pub value: String,
    pub type_name: String,
    pub constant: bool,
    pub export: bool,
}

/// Compiled Type Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclType {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<(String, String)>,
    pub size: usize,
    pub alignment: usize,
}

/// Pre-compiled Assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclAssembly {
    pub label: String,
    pub code: String,
    pub target: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Compiled Constant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclConstant {
    pub name: String,
    pub value_type: String,
    pub value: String,
    pub export: bool,
}

/// RCL Library File
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RclLibrary {
    pub metadata: RclMetadata,
    #[serde(default)]
    pub entries: Vec<RclEntry>,
    #[serde(default)]
    pub imports: Vec<String>,
    #[serde(skip)]
    pub symbol_table: HashMap<String, String>,
}

/// Compiled Library Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RclEntry {
    Function(RclFunction),
    Variable(RclVariable),
    Type(RclType),
    Assembly(RclAssembly),
    Constant(RclConstant),
}

/// RCL Compiler
pub struct RclCompiler {
    pub library: RclLibrary,
    #[allow(dead_code)]
    current_target: String,
}

impl RclCompiler {
    pub fn new(name: &str, target: &str) -> Self {
        Self {
            library: RclLibrary {
                metadata: RclMetadata {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    author: None,
                    description: None,
                    dependencies: Vec::new(),
                    exports: Vec::new(),
                    target: target.to_string(),
                    rcl_version: RCL_VERSION.to_string(),
                    capabilities: Vec::new(),
                },
                entries: Vec::new(),
                imports: Vec::new(),
                symbol_table: HashMap::new(),
            },
            current_target: target.to_string(),
        }
    }
    
    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.body {
            self.compile_statement(stmt)?;
        }
        self.finalize()
    }
    
    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                self.compile_variable(name, value)?;
            }
            Statement::FunctionDef { name, args, body, .. } => {
                self.compile_function(name, args, body)?;
            }
            Statement::Expr(expr) => {
                if let Expr::Call { func, args, .. } = expr {
                    if func == "export" && !args.is_empty() {
                        if let Expr::String(s, _) = &args[0] {
                            self.library.metadata.exports.push(s.clone());
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn compile_variable(&mut self, name: &str, value: &Expr) -> Result<(), String> {
        let value_str = match value {
            Expr::Number(n, _) => n.to_string(),
            Expr::String(s, _) => s.clone(),
            Expr::Boolean(b, _) => b.to_string(),
            _ => return Err("Unsupported variable value".to_string()),
        };
        
        let export_flag = self.library.metadata.exports.contains(&name.to_string());
        
        let entry = RclEntry::Variable(RclVariable {
            name: name.to_string(),
            value: value_str,
            type_name: "auto".to_string(),
            constant: true,
            export: export_flag,
        });
        
        self.library.entries.push(entry);
        Ok(())
    }
    
    fn compile_function(&mut self, name: &str, args: &[String], _body: &[Statement]) -> Result<(), String> {
        let params: Vec<(String, String)> = args.iter()
            .map(|arg| (arg.clone(), "auto".to_string()))
            .collect();
        
        let _export_flag = self.library.metadata.exports.contains(&name.to_string());
        
        let entry = RclEntry::Function(RclFunction {
            name: name.to_string(),
            signature: format!("{}({})", name, args.join(", ")),
            parameters: params,
            return_type: "auto".to_string(),
            inlineable: true,
            pure: false,
            capabilities: Vec::new(),
        });
        
        self.library.entries.push(entry);
        Ok(())
    }
    
    fn finalize(&mut self) -> Result<(), String> {
        // If no exports specified, export everything
        if self.library.metadata.exports.is_empty() {
            for entry in &self.library.entries {
                match entry {
                    RclEntry::Function(f) => {
                        self.library.metadata.exports.push(f.name.clone());
                    }
                    RclEntry::Variable(v) if v.export => {
                        self.library.metadata.exports.push(v.name.clone());
                    }
                    _ => {}
                }
            }
        }
        
        // Build symbol table
        for entry in &self.library.entries {
            let (name, entry_type) = match entry {
                RclEntry::Function(f) => (f.name.clone(), "function".to_string()),
                RclEntry::Variable(v) => (v.name.clone(), "variable".to_string()),
                RclEntry::Constant(c) => (c.name.clone(), "constant".to_string()),
                RclEntry::Type(t) => (t.name.clone(), "type".to_string()),
                RclEntry::Assembly(a) => (a.label.clone(), "assembly".to_string()),
            };
            self.library.symbol_table.insert(name, entry_type);
        }
        
        Ok(())
    }
    
    /// Save RCL library to file WITH ALL REQUIRED FIELDS
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let file = File::create(path)
            .map_err(|e| format!("Failed to create RCL file: {}", e))?;
        
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.library)
            .map_err(|e| format!("Failed to serialize RCL: {}", e))?;
        
        Ok(())
    }
    
    /// Load RCL library from file - FIXED to handle missing fields
    pub fn load_from_file(path: &str) -> Result<RclLibrary, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read RCL file: {}", e))?;
        
        // Parse with serde
        let library: RclLibrary = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse RCL file: {}", e))?;
        
        Ok(library)
    }
}

/// Auto Import Resolver for RCL libraries
pub struct AutoImportResolver;

impl AutoImportResolver {
    pub fn new() -> Self {
        Self
    }
    
    pub fn resolve_imports(&self, source: &str) -> Vec<String> {
        let mut imports = Vec::new();
        
        // Simple import detection: look for "import" statements
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    // Remove quotes and trailing comments
                    let import = rest.split('#').next().unwrap_or(rest).trim();
                    if !import.is_empty() {
                        imports.push(import.trim_matches('"').to_string());
                    }
                }
            }
        }
        
        imports
    }
}

/// RCL Import Manager
pub struct RclImportManager {
    loaded_libraries: HashMap<String, RclLibrary>,
    import_paths: Vec<String>,
}

impl RclImportManager {
    pub fn new() -> Self {
        Self {
            loaded_libraries: HashMap::new(),
            import_paths: vec!["./".to_string(), "./lib/".to_string(), "./rcl/".to_string()],
        }
    }
    
    pub fn import_library(&mut self, lib_name: &str) -> Result<(), String> {
        if self.loaded_libraries.contains_key(lib_name) {
            return Ok(());
        }
        
        for path in &self.import_paths {
            let file_path = format!("{}/{}.rcl", path, lib_name);
            if Path::new(&file_path).exists() {
                let library = RclCompiler::load_from_file(&file_path)?;
                self.loaded_libraries.insert(lib_name.to_string(), library);
                return Ok(());
            }
        }
        
        Err(format!("Library '{}' not found", lib_name))
    }
    
    /// Expand imports in source code
    pub fn expand_imports(&mut self, source: &str) -> Result<(), String> {
        let resolver = AutoImportResolver::new();
        let imports = resolver.resolve_imports(source);
        
        for import in imports {
            self.import_library(&import)?;
        }
        
        Ok(())
    }
    
    /// Add a library manually
    pub fn add_library(&mut self, name: String, library: RclLibrary) {
        self.loaded_libraries.insert(name, library);
    }
    
    /// Get all loaded libraries
    pub fn get_loaded_libraries(&self) -> &HashMap<String, RclLibrary> {
        &self.loaded_libraries
    }
}

/// RCL Assembly Generator
pub struct RclAssemblyGenerator {
    #[allow(dead_code)]
    target: String,
}

impl RclAssemblyGenerator {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
        }
    }
    
    pub fn generate_library_assembly(&self, library: &RclLibrary) -> String {
        let mut asm = String::new();
        
        asm.push_str(&format!("; Library: {}\n", library.metadata.name));
        asm.push_str(&format!("; Target: {}\n\n", library.metadata.target));
        
        for entry in &library.entries {
            match entry {
                RclEntry::Function(func) => {
                    asm.push_str(&format!("; Function: {}\n", func.name));
                    asm.push_str(&format!("{}:\n", func.name));
                    asm.push_str("    ret\n\n");
                }
                RclEntry::Variable(var) if var.export => {
                    asm.push_str(&format!("; Variable: {}\n", var.name));
                    asm.push_str(&format!("{}:\n", var.name));
                    asm.push_str(&format!("    db '{}', 0\n\n", var.value));
                }
                _ => {}
            }
        }
        
        asm
    }
}

/// Create a test RCL library
pub fn create_test_library() -> RclLibrary {
    let metadata = RclMetadata {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        author: Some("Rython Team".to_string()),
        description: Some("Test library".to_string()),
        dependencies: Vec::new(),
        exports: vec!["test_func".to_string(), "test_var".to_string()],
        target: "bios64".to_string(),
        rcl_version: RCL_VERSION.to_string(),
        capabilities: Vec::new(),
    };
    
    let func = RclFunction {
        name: "test_func".to_string(),
        signature: "test_func()".to_string(),
        parameters: Vec::new(),
        return_type: "void".to_string(),
        inlineable: true,
        pure: false,
        capabilities: Vec::new(),
    };
    
    let var = RclVariable {
        name: "test_var".to_string(),
        value: "42".to_string(),
        type_name: "int".to_string(),
        constant: true,
        export: true,
    };
    
    let mut symbol_table = HashMap::new();
    symbol_table.insert("test_func".to_string(), "function".to_string());
    symbol_table.insert("test_var".to_string(), "variable".to_string());
    
    RclLibrary {
        metadata,
        entries: vec![RclEntry::Function(func), RclEntry::Variable(var)],
        imports: Vec::new(),
        symbol_table,
    }
}