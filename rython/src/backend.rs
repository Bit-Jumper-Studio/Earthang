use crate::parser::{Program, Statement, Expr, Position, Span};
use std::collections::HashMap;
use std::cell::RefCell;

// ========== CORE TYPE DEFINITIONS ==========

// Capabilities for backend selection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    // Architecture
    RealMode16,
    ProtectedMode32,
    LongMode64,
    
    // Extensions
    SSE,
    SSE2,
    SSE3,
    SSE4,
    AVX,
    AVX2,
    AVX512,
    
    // Features
    Paging,
    VirtualMemory,
    MultiCore,
    Graphics,
    
    // Environment
    BIOS,
    UEFI,
    PureMetal,
    Linux,
    Windows,
    
    // Constraints
    NoFloat,
    NoHeap,
    NoFilesystem,
    ReadOnly,
}

// Simple module representation without IR
#[derive(Debug, Clone)]
pub struct BackendModule {
    pub functions: Vec<BackendFunction>,
    pub globals: Vec<BackendGlobal>,
    pub required_capabilities: Vec<Capability>,
}

#[derive(Debug, Clone)]
pub struct BackendFunction {
    pub name: String,
    pub parameters: Vec<(String, String)>, // (name, type)
    pub body: Vec<String>, // Assembly instructions
}

#[derive(Debug, Clone)]
pub struct BackendGlobal {
    pub name: String,
    pub value: String,
    pub type_name: String,
}

pub trait Backend {
    /// Backend name for debugging
    fn name(&self) -> &str;
    
    /// Generate assembly header/setup
    fn generate_header(&self) -> String;
    
    /// Supported capabilities
    fn supported_capabilities(&self) -> Vec<Capability>;
    
    /// Required capabilities for this backend
    fn required_capabilities(&self) -> Vec<Capability> {
        Vec::new()
    }
    
    /// Format for NASM
    fn format(&self) -> &'static str;
    
    /// Can this backend handle the module's requirements?
    fn can_compile(&self, module: &BackendModule) -> bool {
        module.required_capabilities.iter()
            .all(|cap| self.supported_capabilities().contains(cap))
    }
    
    /// Generate assembly from program AST
    fn compile_program(&mut self, program: &Program) -> Result<String, String>;
    
    /// Generate function prologue
    fn function_prologue(&self, func: &BackendFunction) -> String;
    
    /// Generate function epilogue
    fn function_epilogue(&self, func: &BackendFunction) -> String;
    
    /// Generate instruction from expression
    fn compile_expression(&self, expr: &Expr) -> Result<String, String>;
}

// ========== BACKEND REGISTRY ==========

pub struct BackendRegistry {
    pub backends: Vec<Box<dyn Backend>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
        }
    }
    
    pub fn register(&mut self, backend: Box<dyn Backend>) {
        self.backends.push(backend);
    }
    
    pub fn find_backend(&self, module: &BackendModule) -> Option<&dyn Backend> {
        self.backends.iter()
            .find(|b| {
                b.can_compile(module) && self.capabilities_match(b.as_ref(), &module.required_capabilities)
            })
            .map(|b| b.as_ref())
    }

    fn capabilities_match(&self, backend: &dyn Backend, module_caps: &[Capability]) -> bool {
        let backend_caps = backend.supported_capabilities();
        module_caps.iter().all(|cap| backend_caps.contains(cap))
    }
    
    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        
        // Register all available backends
        registry.register(Box::new(Bios16Backend::new()));
        registry.register(Box::new(Bios32Backend::new()));
        registry.register(Box::new(Bios64Backend::new()));
        registry.register(Box::new(Bios64Backend::new().with_sse()));
        registry.register(Box::new(Bios64Backend::new().with_avx()));
        registry.register(Box::new(Bios64Backend::new().with_avx512()));
        registry.register(Box::new(Linux64Backend::new()));
        registry.register(Box::new(Windows64Backend::new()));
        
        registry
    }
}

// ========== BIOS 16-BIT BACKEND ==========

pub struct Bios16Backend {
    string_literals: HashMap<String, String>,
}

impl Bios16Backend {
    pub fn new() -> Self {
        Self {
            string_literals: HashMap::new(),
        }
    }
    
    fn generate_string_data(&self) -> String {
        let mut data = String::new();
        for (content, label) in &self.string_literals {
            data.push_str(&format!("{}:\n", label));
            data.push_str(&format!("    db '{}', 0\n", content.replace("'", "''")));
        }
        data
    }
}

impl Backend for Bios16Backend {
    fn name(&self) -> &str {
        "bios16"
    }
    
    fn generate_header(&self) -> String {
        String::from("; BIOS 16-bit Backend\n    org 0x7C00\n    bits 16\n\n")
    }
    
    fn format(&self) -> &'static str {
        "bin"
    }
    
    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::BIOS,
            Capability::RealMode16,
            Capability::PureMetal,
            Capability::NoHeap,
            Capability::NoFilesystem,
        ]
    }
    
    fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        let mut asm = String::new();
        
        asm.push_str("; Rython BIOS 16-bit Backend\n");
        asm.push_str("; Generated from Rython AST\n\n");
        asm.push_str("    org 0x7C00\n");
        asm.push_str("    bits 16\n\n");
        
        asm.push_str("start:\n");
        asm.push_str("    cli\n");
        asm.push_str("    xor ax, ax\n");
        asm.push_str("    mov ds, ax\n");
        asm.push_str("    mov es, ax\n");
        asm.push_str("    mov ss, ax\n");
        asm.push_str("    mov sp, 0x7C00\n");
        asm.push_str("    sti\n");
        asm.push_str("    cld\n\n");
        
        asm.push_str("    mov ax, 0x0003\n");
        asm.push_str("    int 0x10\n\n");
        
        // Compile each statement
        for stmt in &program.body {
            match stmt {
                Statement::VarDecl { name, value, type_hint: _, span: _ } => {
                    asm.push_str(&format!("; Variable: {}\n", name));
                    asm.push_str(&self.compile_expression(value)?);
                }
                Statement::Expr(expr) => {
                    asm.push_str("; Expression\n");
                    asm.push_str(&self.compile_expression(expr)?);
                }
                _ => {
                    asm.push_str("; [Statement]\n");
                }
            }
        }
        
        // Boot signature
        asm.push_str("\n    cli\n");
        asm.push_str("    hlt\n");
        asm.push_str("    jmp $\n\n");
        
        // 16-bit subroutines
        asm.push_str("print_string:\n");
        asm.push_str("    pusha\n");
        asm.push_str(".loop:\n");
        asm.push_str("    lodsb\n");
        asm.push_str("    test al, al\n");
        asm.push_str("    jz .done\n");
        asm.push_str("    int 0x10\n");
        asm.push_str("    jmp .loop\n");
        asm.push_str(".done:\n");
        asm.push_str("    popa\n");
        asm.push_str("    ret\n\n");
        
        asm.push_str("print_decimal:\n");
        asm.push_str("    ; Print decimal number in AX\n");
        asm.push_str("    pusha\n");
        asm.push_str("    mov cx, 0\n");
        asm.push_str("    mov bx, 10\n");
        asm.push_str(".div_loop:\n");
        asm.push_str("    xor dx, dx\n");
        asm.push_str("    div bx\n");
        asm.push_str("    push dx\n");
        asm.push_str("    inc cx\n");
        asm.push_str("    test ax, ax\n");
        asm.push_str("    jnz .div_loop\n");
        asm.push_str(".print_loop:\n");
        asm.push_str("    pop ax\n");
        asm.push_str("    add al, '0'\n");
        asm.push_str("    mov ah, 0x0E\n");
        asm.push_str("    int 0x10\n");
        asm.push_str("    loop .print_loop\n");
        asm.push_str("    popa\n");
        asm.push_str("    ret\n\n");
        
        // String data
        asm.push_str("; String literals\n");
        asm.push_str(&self.generate_string_data());
        
        asm.push_str("\n    times 510-($-$$) db 0\n");
        asm.push_str("    dw 0xAA55\n");
        
        Ok(asm)
    }
    
    fn function_prologue(&self, func: &BackendFunction) -> String {
        format!("{}:\n    push bp\n    mov bp, sp\n", func.name)
    }
    
    fn function_epilogue(&self, _func: &BackendFunction) -> String {
        "    mov sp, bp\n    pop bp\n    ret\n".to_string()
    }
    
    fn compile_expression(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Number(n, _) => Ok(format!("    ; Number: {}\n    mov ax, {}\n    call print_decimal\n", n, n)),
            Expr::String(s, _) => Ok(format!("    ; String: '{}'\n    mov si, str_const\n    call print_string\n", s)),
            Expr::Call { func, args, kwargs: _, span: _ } => {
                if func == "print" {
                    let mut code = String::new();
                    for arg in args {
                        let arg_code = self.compile_expression(arg)?;
                        code.push_str(&arg_code);
                    }
                    Ok(code)
                } else {
                    Ok(format!("    call {}\n", func))
                }
            }
            _ => Ok("    ; [Expression]\n".to_string()),
        }
    }
}

// ========== BIOS 32-BIT BACKEND ==========

pub struct Bios32Backend {
    string_literals: HashMap<String, String>,
}

impl Bios32Backend {
    pub fn new() -> Self {
        Self {
            string_literals: HashMap::new(),
        }
    }
    
    fn generate_string_data(&self) -> String {
        let mut data = String::new();
        for (content, label) in &self.string_literals {
            data.push_str(&format!("{}:\n", label));
            data.push_str(&format!("    db '{}', 0\n", content.replace("'", "''")));
        }
        data
    }
}

impl Backend for Bios32Backend {
    fn name(&self) -> &str {
        "bios32"
    }
    
    fn generate_header(&self) -> String {
        String::from("; BIOS 32-bit Backend\n    org 0x7C00\n    bits 16\n\n")
    }
    
    fn format(&self) -> &'static str {
        "bin"
    }
    
    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::BIOS,
            Capability::ProtectedMode32,
            Capability::PureMetal,
            Capability::Paging,
        ]
    }
    
    fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        let mut asm = String::new();
        
        asm.push_str("; Rython BIOS 32-bit Backend\n");
        asm.push_str("; Generated from Rython AST\n\n");
        asm.push_str("    org 0x7C00\n");
        asm.push_str("    bits 16\n\n");
        
        asm.push_str("start:\n");
        asm.push_str("    cli\n");
        asm.push_str("    xor ax, ax\n");
        asm.push_str("    mov ds, ax\n");
        asm.push_str("    mov es, ax\n");
        asm.push_str("    mov ss, ax\n");
        asm.push_str("    mov sp, 0x7C00\n");
        asm.push_str("    sti\n");
        asm.push_str("    cld\n\n");
        
        asm.push_str("    mov ax, 0x0003\n");
        asm.push_str("    int 0x10\n\n");
        
        // Setup protected mode
        asm.push_str("    in al, 0x92\n");
        asm.push_str("    or al, 2\n");
        asm.push_str("    out 0x92, al\n\n");
        
        asm.push_str("    lgdt [gdt32_desc]\n\n");
        
        asm.push_str("    mov eax, cr0\n");
        asm.push_str("    or eax, 1\n");
        asm.push_str("    mov cr0, eax\n\n");
        
        asm.push_str("    jmp 0x08:protected_mode\n\n");
        
        asm.push_str("    bits 32\n");
        asm.push_str("protected_mode:\n");
        asm.push_str("    mov ax, 0x10\n");
        asm.push_str("    mov ds, ax\n");
        asm.push_str("    mov es, ax\n");
        asm.push_str("    mov fs, ax\n");
        asm.push_str("    mov gs, ax\n");
        asm.push_str("    mov ss, ax\n");
        asm.push_str("    mov esp, 0x7C00\n\n");
        
        // Compile each statement
        for stmt in &program.body {
            match stmt {
                Statement::VarDecl { name, value, type_hint: _, span: _ } => {
                    asm.push_str(&format!("; Variable: {}\n", name));
                    asm.push_str(&self.compile_expression(value)?);
                }
                Statement::Expr(expr) => {
                    asm.push_str("; Expression\n");
                    asm.push_str(&self.compile_expression(expr)?);
                }
                _ => {
                    asm.push_str("; [Statement]\n");
                }
            }
        }
        
        asm.push_str("\n    cli\n");
        asm.push_str("    hlt\n");
        asm.push_str("    jmp $\n\n");
        
        // 32-bit subroutines
        asm.push_str("print_string_32:\n");
        asm.push_str("    pusha\n");
        asm.push_str(".loop:\n");
        asm.push_str("    lodsb\n");
        asm.push_str("    test al, al\n");
        asm.push_str("    jz .done\n");
        asm.push_str("    mov [edi], al\n");
        asm.push_str("    inc edi\n");
        asm.push_str("    mov byte [edi], 0x0F\n");
        asm.push_str("    inc edi\n");
        asm.push_str("    jmp .loop\n");
        asm.push_str(".done:\n");
        asm.push_str("    popa\n");
        asm.push_str("    ret\n\n");
        
        asm.push_str("print_decimal_32:\n");
        asm.push_str("    ; Print decimal number in EAX\n");
        asm.push_str("    pusha\n");
        asm.push_str("    mov ecx, 0\n");
        asm.push_str("    mov ebx, 10\n");
        asm.push_str("    mov edi, 0xB8000 + 160  ; Second line\n");
        asm.push_str(".div_loop:\n");
        asm.push_str("    xor edx, edx\n");
        asm.push_str("    div ebx\n");
        asm.push_str("    push dx\n");
        asm.push_str("    inc ecx\n");
        asm.push_str("    test eax, eax\n");
        asm.push_str("    jnz .div_loop\n");
        asm.push_str(".print_loop:\n");
        asm.push_str("    pop ax\n");
        asm.push_str("    add al, '0'\n");
        asm.push_str("    mov [edi], al\n");
        asm.push_str("    inc edi\n");
        asm.push_str("    mov byte [edi], 0x0F\n");
        asm.push_str("    inc edi\n");
        asm.push_str("    loop .print_loop\n");
        asm.push_str("    popa\n");
        asm.push_str("    ret\n\n");
        
        // String data
        asm.push_str("; String literals\n");
        asm.push_str(&self.generate_string_data());
        
        // GDT
        asm.push_str("gdt32:\n");
        asm.push_str("    dq 0x0000000000000000\n");
        asm.push_str("    dq 0x00CF9A000000FFFF\n");
        asm.push_str("    dq 0x00CF92000000FFFF\n");
        asm.push_str("gdt32_end:\n\n");
        
        asm.push_str("gdt32_desc:\n");
        asm.push_str("    dw gdt32_end - gdt32 - 1\n");
        asm.push_str("    dd gdt32\n\n");
        
        asm.push_str("    times 510-($-$$) db 0\n");
        asm.push_str("    dw 0xAA55\n");
        
        Ok(asm)
    }
    
    fn function_prologue(&self, func: &BackendFunction) -> String {
        format!("{}:\n    push ebp\n    mov ebp, esp\n", func.name)
    }
    
    fn function_epilogue(&self, _func: &BackendFunction) -> String {
        "    mov esp, ebp\n    pop ebp\n    ret\n".to_string()
    }
    
    fn compile_expression(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Number(n, _) => Ok(format!("    mov eax, {}\n    call print_decimal_32\n", n)),
            Expr::String(s, _) => {
                let label = format!("str_{}", s.hash_code());
                Ok(format!("    ; String: '{}'\n    mov esi, {}\n    mov edi, 0xB8000\n    call print_string_32\n", s, label))
            }
            Expr::Call { func, args, kwargs: _, span: _ } => {
                if func == "print" {
                    let mut code = String::new();
                    for arg in args {
                        code.push_str(&self.compile_expression(arg)?);
                    }
                    Ok(code)
                } else {
                    Ok(format!("    call {}\n", func))
                }
            }
            _ => Ok("    ; [Expression]\n".to_string()),
        }
    }
}

// ========== BIOS 64-BIT BACKEND ==========

pub struct Bios64Backend {
    use_sse: bool,
    use_avx: bool,
    use_avx512: bool,
    string_counter: RefCell<u32>,
    string_literals: RefCell<HashMap<String, String>>,
    #[allow(dead_code)]
    code_size_limit: usize,
}

impl Bios64Backend {
    pub fn new() -> Self {
        Self {
            use_sse: false,
            use_avx: false,
            use_avx512: false,
            string_counter: RefCell::new(0),
            string_literals: RefCell::new(HashMap::new()),
            code_size_limit: 4096,  // 4KB max for kernel
        }
    }
    
    // Add this method to check size
    #[allow(dead_code)]
    fn check_code_size(&self, code: &str, strings: &HashMap<String, String>) -> Result<(), String> {
        let code_bytes = code.len();
        let string_bytes: usize = strings.values()
            .map(|s| s.len() + 1)  // +1 for null terminator
            .sum();
        let total = code_bytes + string_bytes;
        
        if total > self.code_size_limit {
            Err(format!(
                "Code size limit exceeded: {} bytes (limit: {}). Too many print statements or string data.",
                total, self.code_size_limit
            ))
        } else {
            Ok(())
        }
    }
    
    
    pub fn with_sse(mut self) -> Self {
        self.use_sse = true;
        self
    }
    
    pub fn with_avx(mut self) -> Self {
        self.use_avx = true;
        self
    }
    
    pub fn with_avx512(mut self) -> Self {
        self.use_avx512 = true;
        self
    }
    
    fn get_string_label(&self, content: &str) -> String {
        let mut literals = self.string_literals.borrow_mut();
        if let Some(label) = literals.get(content) {
            return label.clone();
        }
        
        let mut counter = self.string_counter.borrow_mut();
        let label = format!("str_{}", *counter);
        *counter += 1;
        literals.insert(content.to_string(), label.clone());
        label
    }
    
    fn generate_string_data(&self) -> String {
        let literals = self.string_literals.borrow();
        let mut data = String::new();
        for (content, label) in &*literals {
            data.push_str(&format!("{}:\n", label));
            data.push_str(&format!("    db '{}', 0\n", content.replace("'", "''")));
        }
        data
    }
}

impl Backend for Bios64Backend {
    fn name(&self) -> &str {
        "bios64"
    }
    
    fn generate_header(&self) -> String {
        let mut header = String::from("; BIOS 64-bit Backend\n    org 0x7C00\n    bits 16\n\n");
        
        if self.use_sse {
            header.push_str("    ; SSE enabled\n");
        }
        if self.use_avx {
            header.push_str("    ; AVX enabled\n");
        }
        if self.use_avx512 {
            header.push_str("    ; AVX-512 enabled\n");
        }
        
        header
    }
    
    fn format(&self) -> &'static str {
        "bin"
    }
    
    fn supported_capabilities(&self) -> Vec<Capability> {
        let mut caps = vec![
            Capability::BIOS,
            Capability::LongMode64,
            Capability::PureMetal,
            Capability::Paging,
        ];
        
        if self.use_sse {
            caps.push(Capability::SSE);
            caps.push(Capability::SSE2);
        }
        
        if self.use_avx {
            caps.push(Capability::AVX);
        }
        
        if self.use_avx512 {
            caps.push(Capability::AVX512);
        }
        
        caps
    }
    
    fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        let mut asm = String::new();
        
        asm.push_str("; Rython BIOS 64-bit Backend\n");
        asm.push_str("; Generated from Rython AST\n\n");
        asm.push_str("    org 0x7C00\n");
        asm.push_str("    bits 16\n\n");
        
        // Add feature flags
        if self.use_sse {
            asm.push_str("    ; SSE enabled\n");
        }
        if self.use_avx {
            asm.push_str("    ; AVX enabled\n");
        }
        if self.use_avx512 {
            asm.push_str("    ; AVX-512 enabled\n");
        }
        asm.push_str("\n");
        
        asm.push_str("start:\n");
        asm.push_str("    cli\n");
        asm.push_str("    xor ax, ax\n");
        asm.push_str("    mov ds, ax\n");
        asm.push_str("    mov es, ax\n");
        asm.push_str("    mov ss, ax\n");
        asm.push_str("    mov sp, 0x7C00\n");
        asm.push_str("    sti\n");
        asm.push_str("    cld\n\n");
        
        asm.push_str("    mov ax, 0x0003\n");
        asm.push_str("    int 0x10\n\n");
        
        // Simple A20 enable
        asm.push_str("    in al, 0x92\n");
        asm.push_str("    or al, 2\n");
        asm.push_str("    out 0x92, al\n\n");
        
        // Load 32-bit GDT
        asm.push_str("    lgdt [gdt32_desc]\n\n");
        
        // Enter protected mode
        asm.push_str("    mov eax, cr0\n");
        asm.push_str("    or eax, 1\n");
        asm.push_str("    mov cr0, eax\n\n");
        asm.push_str("    jmp 0x08:protected_mode\n\n");
        
        // ========== 32-bit code ==========
        asm.push_str("    bits 32\n");
        asm.push_str("protected_mode:\n");
        asm.push_str("    mov ax, 0x10\n");
        asm.push_str("    mov ds, ax\n");
        asm.push_str("    mov es, ax\n");
        asm.push_str("    mov fs, ax\n");
        asm.push_str("    mov gs, ax\n");
        asm.push_str("    mov ss, ax\n");
        asm.push_str("    mov esp, 0x90000\n\n");
        
        // Setup paging (32-bit code)
        asm.push_str("    ; Setup paging\n");
        asm.push_str("    mov edi, 0x1000\n");
        asm.push_str("    mov cr3, edi\n");
        asm.push_str("    xor eax, eax\n");
        asm.push_str("    mov ecx, 4096\n");
        asm.push_str("    rep stosd\n");
        
        asm.push_str("    mov edi, 0x1000\n");
        asm.push_str("    mov dword [edi], 0x2003\n");
        asm.push_str("    add edi, 0x1000\n");
        asm.push_str("    mov dword [edi], 0x3003\n");
        asm.push_str("    add edi, 0x1000\n");
        
        asm.push_str("    mov ebx, 0x00000083\n");
        asm.push_str("    mov ecx, 512\n");
        asm.push_str(".set_entry:\n");
        asm.push_str("    mov dword [edi], ebx\n");
        asm.push_str("    add ebx, 0x200000\n");
        asm.push_str("    add edi, 8\n");
        asm.push_str("    loop .set_entry\n\n");
        
        // Enable PAE
        asm.push_str("    mov eax, cr4\n");
        asm.push_str("    or eax, (1 << 5)\n");
        asm.push_str("    mov cr4, eax\n\n");
        
        // Set CR3
        asm.push_str("    mov eax, 0x1000\n");
        asm.push_str("    mov cr3, eax\n\n");
        
        // Enable long mode
        asm.push_str("    mov ecx, 0xC0000080\n");
        asm.push_str("    rdmsr\n");
        asm.push_str("    or eax, (1 << 8)\n");
        asm.push_str("    wrmsr\n\n");
        
        // Enable paging
        asm.push_str("    mov eax, cr0\n");
        asm.push_str("    or eax, (1 << 31)\n");
        asm.push_str("    mov cr0, eax\n\n");
        
        // Load 64-bit GDT
        asm.push_str("    lgdt [gdt64_desc]\n\n");
        
        // Jump to 64-bit mode
        asm.push_str("    jmp 0x08:long_mode\n\n");
        
        // ========== 64-bit code ==========
        asm.push_str("    bits 64\n");
        asm.push_str("long_mode:\n");
        
        asm.push_str("    mov ax, 0x10\n");
        asm.push_str("    mov ds, ax\n");
        asm.push_str("    mov es, ax\n");
        asm.push_str("    mov fs, ax\n");
        asm.push_str("    mov gs, ax\n");
        asm.push_str("    mov ss, ax\n");
        asm.push_str("    mov rsp, 0x90000\n\n");
        
        // Clear screen (64-bit)
        asm.push_str("    mov rdi, 0xB8000\n");
        asm.push_str("    mov rax, 0x0720072007200720\n");
        asm.push_str("    mov rcx, 1000\n");
        asm.push_str("    rep stosq\n\n");
        
        // Print message (64-bit)
        asm.push_str("    mov rsi, msg_64\n");
        asm.push_str("    mov rdi, 0xB8000\n");
        asm.push_str("    call print_string_64\n\n");
        
        // Compile program statements
        for stmt in &program.body {
            match stmt {
                Statement::Expr(expr) => {
                    asm.push_str("; Expression\n");
                    asm.push_str(&self.compile_expression(expr)?);
                }
                Statement::FunctionDef { name, args, body, span: _ } => {
                    asm.push_str(&format!("; Function: {}\n", name));
                    asm.push_str(&format!("{}:\n", name));
                    
                    // Create a Program for the function body with a default span
                    let _func_program = Program {
                        body: body.to_vec(),
                        span: Span::single(Position::start()),
                    };
                    
                    asm.push_str(&self.function_prologue(&BackendFunction {
                        name: name.clone(),
                        parameters: args.iter().map(|arg| (arg.clone(), "int".to_string())).collect(),
                        body: Vec::new(),
                    }));
                    
                    // Compile function body
                    for body_stmt in body {
                        if let Statement::Expr(expr) = body_stmt {
                            asm.push_str(&self.compile_expression(expr)?);
                        }
                    }
                    
                    asm.push_str(&self.function_epilogue(&BackendFunction {
                        name: name.clone(),
                        parameters: args.iter().map(|arg| (arg.clone(), "int".to_string())).collect(),
                        body: Vec::new(),
                    }));
                }
                Statement::VarDecl { name, value, type_hint: _, span: _ } => {
                    asm.push_str(&format!("; Variable: {}\n", name));
                    asm.push_str(&self.compile_expression(value)?);
                }
                _ => {
                    asm.push_str("; [Other statement]\n");
                }
            }
        }
        
        asm.push_str("\n    cli\n");
        asm.push_str("    hlt\n");
        asm.push_str("    jmp $\n\n");
        
        // ========== 64-bit subroutines ==========
        asm.push_str("print_string_64:\n");
        asm.push_str("    push rdi\n");
        asm.push_str(".loop:\n");
        asm.push_str("    lodsb\n");
        asm.push_str("    test al, al\n");
        asm.push_str("    jz .done\n");
        asm.push_str("    stosb\n");
        asm.push_str("    mov al, 0x0F\n");
        asm.push_str("    stosb\n");
        asm.push_str("    jmp .loop\n");
        asm.push_str(".done:\n");
        asm.push_str("    pop rdi\n");
        asm.push_str("    ret\n\n");
        
        asm.push_str("print_decimal_64:\n");
        asm.push_str("    ; Print decimal number in RAX\n");
        asm.push_str("    push rdi\n");
        asm.push_str("    push rcx\n");
        asm.push_str("    push rdx\n");
        asm.push_str("    push rbx\n");
        asm.push_str("    \n");
        asm.push_str("    mov rdi, 0xB8000 + 160  ; Second line\n");
        asm.push_str("    mov rcx, 0\n");
        asm.push_str("    mov rbx, 10\n");
        asm.push_str(".div_loop:\n");
        asm.push_str("    xor rdx, rdx\n");
        asm.push_str("    div rbx\n");
        asm.push_str("    push dx\n");
        asm.push_str("    inc rcx\n");
        asm.push_str("    test rax, rax\n");
        asm.push_str("    jnz .div_loop\n");
        asm.push_str(".print_loop:\n");
        asm.push_str("    pop ax\n");
        asm.push_str("    add al, '0'\n");
        asm.push_str("    stosb\n");
        asm.push_str("    mov al, 0x0F\n");
        asm.push_str("    stosb\n");
        asm.push_str("    loop .print_loop\n");
        asm.push_str("    \n");
        asm.push_str("    pop rbx\n");
        asm.push_str("    pop rdx\n");
        asm.push_str("    pop rcx\n");
        asm.push_str("    pop rdi\n");
        asm.push_str("    ret\n\n");
        
        // ========== GDTs ==========
        asm.push_str("gdt32:\n");
        asm.push_str("    dq 0x0000000000000000\n");
        asm.push_str("    dq 0x00CF9A000000FFFF\n");
        asm.push_str("    dq 0x00CF92000000FFFF\n");
        asm.push_str("gdt32_end:\n\n");
        
        asm.push_str("gdt32_desc:\n");
        asm.push_str("    dw gdt32_end - gdt32 - 1\n");
        asm.push_str("    dd gdt32\n\n");
        
        asm.push_str("gdt64:\n");
        asm.push_str("    dq 0x0000000000000000\n");
        asm.push_str("    dq 0x00209A0000000000\n");
        asm.push_str("    dq 0x0000920000000000\n");
        asm.push_str("gdt64_end:\n\n");
        
        asm.push_str("gdt64_desc:\n");
        asm.push_str("    dw gdt64_end - gdt64 - 1\n");
        asm.push_str("    dq gdt64\n\n");
        
        // ========== Data ==========
        asm.push_str("msg_64:\n");
        asm.push_str("    db 'Rython 64-bit', 0\n\n");
        
        // String literals
        asm.push_str("; String literals\n");
        asm.push_str(&self.generate_string_data());
        
        asm.push_str("    times 510-($-$$) db 0\n");
        asm.push_str("    dw 0xAA55\n");
        
        Ok(asm)
    }
    
    fn function_prologue(&self, func: &BackendFunction) -> String {
        let mut prologue = String::new();
        prologue.push_str(&format!("{}:\n", func.name));
        prologue.push_str("    push rbp\n");
        prologue.push_str("    mov rbp, rsp\n");
        
        // Allocate stack space for locals
        let stack_size = func.parameters.len() * 8;
        if stack_size > 0 {
            prologue.push_str(&format!("    sub rsp, {}\n", stack_size));
        }
        
        prologue
    }
    
    fn function_epilogue(&self, _func: &BackendFunction) -> String {
        let mut epilogue = String::new();
        epilogue.push_str("    mov rsp, rbp\n");
        epilogue.push_str("    pop rbp\n");
        epilogue.push_str("    ret\n");
        epilogue
    }
    
    fn compile_expression(&self, expr: &Expr) -> Result<String, String> {
        let mut code = String::new();
        
        match expr {
            Expr::Number(n, _) => {
                code.push_str(&format!("    ; Number: {}\n", n));
                code.push_str(&format!("    mov rax, {}\n", n));
                code.push_str("    call print_decimal_64\n");
            }
            Expr::String(s, _) => {
                let label = self.get_string_label(s);
                code.push_str(&format!("    ; String: '{}'\n", s));
                code.push_str(&format!("    mov rsi, {}\n", label));
                code.push_str("    mov rdi, 0xB8000 + 320  ; Third line\n");
                code.push_str("    call print_string_64\n");
            }
            Expr::Call { func, args, kwargs: _, span: _ } => {
                if func == "print" {
                    for arg in args {
                        let arg_code = self.compile_expression(arg)?;
                        code.push_str(&arg_code);
                    }
                } else {
                    code.push_str(&format!("    call {}\n", func));
                }
            }
            Expr::BinOp { left, op, right, span: _ } => {
                let left_code = self.compile_expression(left)?;
                let right_code = self.compile_expression(right)?;
                code.push_str(&left_code);
                code.push_str(&right_code);
                
                code.push_str("    ; Binary operation\n");
                code.push_str("    pop rbx\n");
                code.push_str("    pop rax\n");
                
                match op {
                    crate::parser::Op::Add => {
                        code.push_str("    add rax, rbx\n");
                    }
                    crate::parser::Op::Sub => {
                        code.push_str("    sub rax, rbx\n");
                    }
                    crate::parser::Op::Mul => {
                        code.push_str("    imul rax, rbx\n");
                    }
                    crate::parser::Op::Div => {
                        code.push_str("    xor rdx, rdx\n");
                        code.push_str("    idiv rbx\n");
                    }
                    _ => {
                        code.push_str("    add rax, rbx\n");
                    }
                }
                
                code.push_str("    push rax\n");
            }
            _ => {
                code.push_str("    ; [Expression]\n");
            }
        }
        
        Ok(code)
    }
}

// ========== LINUX 64-BIT BACKEND ==========

pub struct Linux64Backend {
    string_counter: RefCell<u32>,
    string_literals: RefCell<HashMap<String, String>>,
}

impl Linux64Backend {
    pub fn new() -> Self {
        Self {
            string_counter: RefCell::new(0),
            string_literals: RefCell::new(HashMap::new()),
        }
    }
    
    fn get_string_label(&self, content: &str) -> String {
        let mut literals = self.string_literals.borrow_mut();
        if let Some(label) = literals.get(content) {
            return label.clone();
        }
        
        let mut counter = self.string_counter.borrow_mut();
        let label = format!("str_{}", *counter);
        *counter += 1;
        literals.insert(content.to_string(), label.clone());
        label
    }
    
    fn generate_string_data(&self) -> String {
        let literals = self.string_literals.borrow();
        let mut data = String::new();
        for (content, label) in &*literals {
            data.push_str(&format!("{}:\n", label));
            data.push_str(&format!("    db '{}', 0\n", content.replace("'", "''")));
        }
        data
    }
}

impl Backend for Linux64Backend {
    fn name(&self) -> &str {
        "linux64"
    }
    
    fn generate_header(&self) -> String {
        String::from("; Linux 64-bit Backend\n    bits 64\n    default rel\n\n    section .text\n    global _start\n\n")
    }
    
    fn format(&self) -> &'static str {
        "elf64"
    }
    
    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Linux,
            Capability::LongMode64,
            Capability::VirtualMemory,
        ]
    }
    
    fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        let mut asm = String::new();
        
        asm.push_str("; Rython Linux 64-bit Backend\n");
        asm.push_str("; Generated from Rython AST\n\n");
        asm.push_str("    bits 64\n");
        asm.push_str("    default rel\n\n");
        
        asm.push_str("    section .text\n");
        asm.push_str("    global _start\n\n");
        
        asm.push_str("_start:\n");
        
        // Compile statements
        for stmt in &program.body {
            if let Statement::Expr(expr) = stmt {
                asm.push_str(&self.compile_expression(expr)?);
            }
        }
        
        // System exit
        asm.push_str("    ; Exit\n");
        asm.push_str("    mov rax, 60    ; sys_exit\n");
        asm.push_str("    xor rdi, rdi   ; exit code 0\n");
        asm.push_str("    syscall\n");
        
        // String print function
        asm.push_str("\n; Print string function\n");
        asm.push_str("print_string:\n");
        asm.push_str("    ; rsi = string address\n");
        asm.push_str("    push rcx\n");
        asm.push_str("    push rdx\n");
        asm.push_str("    push rdi\n");
        asm.push_str("    push rsi\n");
        asm.push_str("    \n");
        asm.push_str("    ; Calculate string length\n");
        asm.push_str("    mov rdi, rsi\n");
        asm.push_str("    xor rcx, rcx\n");
        asm.push_str("    dec rcx\n");
        asm.push_str(".count_loop:\n");
        asm.push_str("    inc rcx\n");
        asm.push_str("    cmp byte [rdi + rcx], 0\n");
        asm.push_str("    jne .count_loop\n");
        asm.push_str("    \n");
        asm.push_str("    ; Write to stdout\n");
        asm.push_str("    mov rax, 1        ; sys_write\n");
        asm.push_str("    mov rdi, 1        ; stdout\n");
        asm.push_str("    mov rdx, rcx      ; length\n");
        asm.push_str("    syscall\n");
        asm.push_str("    \n");
        asm.push_str("    pop rsi\n");
        asm.push_str("    pop rdi\n");
        asm.push_str("    pop rdx\n");
        asm.push_str("    pop rcx\n");
        asm.push_str("    ret\n\n");
        
        // Data section
        asm.push_str("    section .data\n");
        asm.push_str(&self.generate_string_data());
        
        Ok(asm)
    }
    
    fn function_prologue(&self, func: &BackendFunction) -> String {
        format!("{}:\n    push rbp\n    mov rbp, rsp\n", func.name)
    }
    
    fn function_epilogue(&self, _func: &BackendFunction) -> String {
        "    mov rsp, rbp\n    pop rbp\n    ret\n".to_string()
    }
    
    fn compile_expression(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Number(n, _) => Ok(format!("    ; Number: {}\n    mov rax, {}\n", n, n)),
            Expr::String(s, _) => {
                let label = self.get_string_label(s);
                Ok(format!(
                    "    ; String: '{}'\n    mov rsi, {}\n    call print_string\n",
                    s, label
                ))
            }
            Expr::Call { func, args, kwargs: _, span: _ } if func == "print" => {
                if let Some(arg) = args.get(0) {
                    self.compile_expression(arg)
                } else {
                    Ok("    ; Empty print\n".to_string())
                }
            }
            _ => Ok(format!("    ; {:?}\n", expr)),
        }
    }
}

// ========== WINDOWS 64-BIT BACKEND ==========

pub struct Windows64Backend {
    string_counter: RefCell<u32>,
    string_literals: RefCell<HashMap<String, String>>,
}

impl Windows64Backend {
    pub fn new() -> Self {
        Self {
            string_counter: RefCell::new(0),
            string_literals: RefCell::new(HashMap::new()),
        }
    }
    
    fn get_string_label(&self, content: &str) -> String {
        let mut literals = self.string_literals.borrow_mut();
        if let Some(label) = literals.get(content) {
            return label.clone();
        }
        
        let mut counter = self.string_counter.borrow_mut();
        let label = format!("str_{}", *counter);
        *counter += 1;
        literals.insert(content.to_string(), label.clone());
        label
    }
    
    fn generate_string_data(&self) -> String {
        let literals = self.string_literals.borrow();
        let mut data = String::new();
        for (content, label) in &*literals {
            data.push_str(&format!("{} db '{}', 0\n", label, content.replace("'", "''")));
        }
        data
    }
}

impl Backend for Windows64Backend {
    fn name(&self) -> &str {
        "windows64"
    }
    
    fn generate_header(&self) -> String {
        String::from("; Windows 64-bit Backend\n    bits 64\n    default rel\n\n    section .text\n")
    }
    
    fn format(&self) -> &'static str {
        "win64"
    }
    
    fn supported_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Windows,
            Capability::LongMode64,
        ]
    }
    
    fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        let mut asm = String::new();
        
        asm.push_str("; Rython Windows 64-bit Backend\n");
        asm.push_str("; Generated from Rython AST\n\n");
        asm.push_str("    bits 64\n");
        asm.push_str("    default rel\n\n");
        
        asm.push_str("    section .text\n");
        asm.push_str("    extern ExitProcess\n");
        asm.push_str("    extern printf\n");
        asm.push_str("    global main\n\n");
        
        asm.push_str("main:\n");
        asm.push_str("    sub rsp, 8 * (4 + 1)  ; Allocate shadow space + one QWORD for alignment\n");
        
        // Compile statements
        for stmt in &program.body {
            if let Statement::Expr(expr) = stmt {
                asm.push_str(&self.compile_expression(expr)?);
            }
        }
        
        // Windows exit
        asm.push_str("    ; Exit\n");
        asm.push_str("    xor ecx, ecx          ; exit code 0\n");
        asm.push_str("    call ExitProcess\n");
        
        asm.push_str("\n    section .data\n");
        asm.push_str(&self.generate_string_data());
        
        Ok(asm)
    }
    
    fn function_prologue(&self, func: &BackendFunction) -> String {
        format!("{}:\n    push rbp\n    mov rbp, rsp\n", func.name)
    }
    
    fn function_epilogue(&self, _func: &BackendFunction) -> String {
        "    mov rsp, rbp\n    pop rbp\n    ret\n".to_string()
    }
    
    fn compile_expression(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Number(n, _) => Ok(format!("    ; Number: {}\n    mov rax, {}\n", n, n)),
            Expr::String(s, _) => {
                let label = self.get_string_label(s);
                Ok(format!(
                    "    ; String: '{}'\n    lea rcx, [{}]\n    sub rsp, 32\n    call printf\n    add rsp, 32\n",
                    s, label
                ))
            }
            Expr::Call { func, args, kwargs: _, span: _ } if func == "print" => {
                if let Some(arg) = args.get(0) {
                    self.compile_expression(arg)
                } else {
                    Ok("    ; Empty print\n".to_string())
                }
            }
            _ => Ok(format!("    ; {:?}\n", expr)),
        }
    }
}

// Helper trait for string hashing
trait HashCode {
    fn hash_code(&self) -> u64;
}

impl HashCode for str {
    fn hash_code(&self) -> u64 {
        let mut hash: u64 = 5381;
        for &byte in self.as_bytes() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte as u64);
        }
        hash
    }
}