use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsdHeader {
    pub syntax_extensions: Vec<SsdSyntaxExtension>,
    pub capabilities: Vec<String>,
    pub metadata: SsdMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsdSyntaxExtension {
    pub pattern: String,
    pub replacement: String,
    pub assembly_label: Option<String>,
    pub register_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsdMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsdAssemblyBlock {
    pub label: String,
    pub code: String,
    pub target: String,
    pub dependencies: Vec<String>,
}

pub struct SsdInjector {
    pub syntax_table: HashMap<String, String>,
    pub assembly_blocks: HashMap<String, Vec<String>>,
    pub loaded_headers: Vec<SsdHeader>,
    pub loaded_assembly: Vec<SsdAssemblyBlock>,
}

impl SsdInjector {
    pub fn new() -> Self {
        Self {
            syntax_table: HashMap::new(),
            assembly_blocks: HashMap::new(),
            loaded_headers: Vec::new(),
            loaded_assembly: Vec::new(),
        }
    }
    
    pub fn load_header(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read SSD header: {}", e))?;
        
        let header: SsdHeader = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse SSD header: {}", e))?;
        
        self.loaded_headers.push(header.clone());
        
        // Add syntax extensions to table
        for ext in &header.syntax_extensions {
            self.syntax_table.insert(ext.pattern.clone(), ext.replacement.clone());
        }
        
        Ok(())
    }
    
    pub fn load_assembly_block(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read SSD assembly: {}", e))?;
        
        let block: SsdAssemblyBlock = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse SSD assembly: {}", e))?;
        
        self.loaded_assembly.push(block.clone());
        
        // Add to assembly blocks
        let entry = self.assembly_blocks.entry(block.target.clone()).or_insert(Vec::new());
        entry.push(block.code.clone());
        
        Ok(())
    }
    
    pub fn apply_syntax_extensions(&self, source: &str) -> String {
        let mut result = source.to_string();
        
        for (pattern, replacement) in &self.syntax_table {
            // Simple string replacement (in real implementation, would use proper parsing)
            result = result.replace(pattern, replacement);
        }
        
        result
    }
    
    pub fn get_assembly_for_target(&self, target: &str) -> Vec<String> {
        self.assembly_blocks.get(target).cloned().unwrap_or_default()
    }
    
    pub fn create_test_syntax_mutation() -> SsdHeader {
        SsdHeader {
            syntax_extensions: vec![
                SsdSyntaxExtension {
                    pattern: ">".to_string(),
                    replacement: "print".to_string(),
                    assembly_label: Some("fast_print".to_string()),
                    register_args: vec!["rcx".to_string(), "rdx".to_string()],
                },
                SsdSyntaxExtension {
                    pattern: "!!".to_string(),
                    replacement: "panic".to_string(),
                    assembly_label: Some("panic".to_string()),
                    register_args: vec!["rcx".to_string()],
                },
            ],
            capabilities: vec!["syntax_mutation".to_string(), "fast_print".to_string()],
            metadata: SsdMetadata {
                name: "Test SSD Header".to_string(),
                version: "1.0.0".to_string(),
                author: Some("Rython Team".to_string()),
                description: Some("Test syntax mutation header".to_string()),
            },
        }
    }
    
    pub fn create_test_negative_number_handling() -> SsdAssemblyBlock {
        SsdAssemblyBlock {
            label: "print_negative".to_string(),
            code: r#"
; SSD Assembly: Negative number handling
print_negative:
    test rax, rax
    jns .positive
    push rax
    mov rcx, '-'
    call putchar
    pop rax
    neg rax
.positive:
    call print_decimal
    ret
"#.to_string(),
            target: "windows64".to_string(),
            dependencies: vec!["putchar".to_string(), "print_decimal".to_string()],
        }
    }
}