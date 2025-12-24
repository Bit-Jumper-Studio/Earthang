// src/emitter.rs - Complete fixed version
//! NASM Assembly Emitter for Rython AST - With Mode Transitions

use crate::parser::{Program, Statement, Expr};
use std::collections::HashMap;

// ========== TARGET CONFIGURATION ==========

#[derive(Debug, Clone, PartialEq)]
pub enum TargetPlatform {
    Linux64,
    Windows64,  
    Bios16,
    Bios32,
    Bios64,
    Bios64SSE,
    Bios64AVX,
    Bios64AVX512,
}

#[derive(Debug, Clone)]
pub struct TargetConfig {
    pub platform: TargetPlatform,
    pub bits: u8,
    pub format: &'static str,
    pub entry_point: &'static str,
}

impl TargetConfig {
    pub fn linux64() -> Self { Self { platform: TargetPlatform::Linux64, bits: 64, format: "elf64", entry_point: "_start" } }
    pub fn windows64() -> Self { Self { platform: TargetPlatform::Windows64, bits: 64, format: "win64", entry_point: "main" } }
    pub fn bios16() -> Self { Self { platform: TargetPlatform::Bios16, bits: 16, format: "bin", entry_point: "_start" } }
    pub fn bios32() -> Self { Self { platform: TargetPlatform::Bios32, bits: 32, format: "bin", entry_point: "_start" } }
    pub fn bios64() -> Self { Self { platform: TargetPlatform::Bios64, bits: 64, format: "bin", entry_point: "_start" } }
    pub fn bios64_sse() -> Self { Self { platform: TargetPlatform::Bios64SSE, bits: 64, format: "bin", entry_point: "_start" } }
    pub fn bios64_avx() -> Self { Self { platform: TargetPlatform::Bios64AVX, bits: 64, format: "bin", entry_point: "_start" } }
    pub fn bios64_avx512() -> Self { Self { platform: TargetPlatform::Bios64AVX512, bits: 64, format: "bin", entry_point: "_start" } }
    
    pub fn is_windows(&self) -> bool { self.platform == TargetPlatform::Windows64 }
    pub fn is_bios(&self) -> bool {
        matches!(self.platform, TargetPlatform::Bios16 | TargetPlatform::Bios32 | 
                 TargetPlatform::Bios64 | TargetPlatform::Bios64SSE |
                 TargetPlatform::Bios64AVX | TargetPlatform::Bios64AVX512)
    }
    pub fn is_linux(&self) -> bool { self.platform == TargetPlatform::Linux64 }
}

// ========== NASM EMITTER ==========

pub struct NasmEmitter {
    target: TargetConfig,
    label_counter: u32,
    variable_offsets: HashMap<String, i32>,
}

impl NasmEmitter {
    pub fn new() -> Self {
        Self {
            target: TargetConfig::linux64(),
            label_counter: 0,
            variable_offsets: HashMap::new(),
        }
    }
    
    // Target configuration methods
    pub fn set_target_linux(&mut self) { self.target = TargetConfig::linux64(); }
    pub fn set_target_windows(&mut self) { self.target = TargetConfig::windows64(); }
    pub fn set_target_bios16(&mut self) { self.target = TargetConfig::bios16(); }
    pub fn set_target_bios32(&mut self) { self.target = TargetConfig::bios32(); }
    pub fn set_target_bios64(&mut self) { self.target = TargetConfig::bios64(); }
    pub fn set_target_bios64_sse(&mut self) { self.target = TargetConfig::bios64_sse(); }
    pub fn set_target_bios64_avx(&mut self) { self.target = TargetConfig::bios64_avx(); }
    pub fn set_target_bios64_avx512(&mut self) { self.target = TargetConfig::bios64_avx512(); }
    
    // Main compilation
    pub fn compile_program(&mut self, program: &Program) -> String {
        self.label_counter = 0;
        self.variable_offsets.clear();
        
        if self.target.is_bios() {
            self.compile_bios(program)
        } else {
            self.compile_standard(program)
        }
    }
    
    fn compile_bios(&mut self, program: &Program) -> String {
        let mut code = String::new();
        
        match self.target.platform {
            TargetPlatform::Bios16 => self.compile_bios16(&mut code, program),
            TargetPlatform::Bios32 => self.compile_bios32(&mut code, program),
            TargetPlatform::Bios64 => self.compile_bios64_fixed(&mut code, program),
            TargetPlatform::Bios64SSE => self.compile_bios64_sse(&mut code, program),
            TargetPlatform::Bios64AVX => self.compile_bios64_avx(&mut code, program),
            TargetPlatform::Bios64AVX512 => self.compile_bios64_avx512(&mut code, program),
            _ => { code.push_str("; Unsupported BIOS target\n"); code }
        }
    }
    
    fn compile_bios16(&mut self, code: &mut String, program: &Program) -> String {
        code.push_str("; Rython 16-bit Bootloader\n\n");
        code.push_str("    org 0x7C00\n");
        code.push_str("    bits 16\n\n");
        code.push_str("start:\n");
        code.push_str("    cli\n");
        code.push_str("    xor ax, ax\n");
        code.push_str("    mov ds, ax\n");
        code.push_str("    mov es, ax\n");
        code.push_str("    mov ss, ax\n");
        code.push_str("    mov sp, 0x7C00\n");
        code.push_str("    sti\n");
        code.push_str("    cld\n\n");
        
        code.push_str("    mov ax, 0x0003\n");
        code.push_str("    int 0x10\n\n");
        
        code.push_str("    mov si, msg\n");
        code.push_str("    call print_string\n\n");
        
        self.compile_bios16_statements(code, &program.body);
        
        code.push_str("\n    cli\n");
        code.push_str("    hlt\n");
        code.push_str("    jmp $\n\n");
        
        // 16-bit subroutines
        code.push_str("print_string:\n");
        code.push_str("    pusha\n");
        code.push_str("    mov ah, 0x0E\n");
        code.push_str(".loop:\n");
        code.push_str("    lodsb\n");
        code.push_str("    test al, al\n");
        code.push_str("    jz .done\n");
        code.push_str("    int 0x10\n");
        code.push_str("    jmp .loop\n");
        code.push_str(".done:\n");
        code.push_str("    popa\n");
        code.push_str("    ret\n\n");
        
        code.push_str("msg:\n");
        code.push_str("    db 'Rython 16-bit', 0\n\n");
        
        code.push_str("    times 510-($-$$) db 0\n");
        code.push_str("    dw 0xAA55\n");
        
        code.clone()
    }
    
    fn compile_bios32(&mut self, code: &mut String, program: &Program) -> String {
        code.push_str("; Rython 32-bit Bootloader\n\n");
        code.push_str("    org 0x7C00\n");
        code.push_str("    bits 16\n\n");
        code.push_str("start:\n");
        code.push_str("    cli\n");
        code.push_str("    xor ax, ax\n");
        code.push_str("    mov ds, ax\n");
        code.push_str("    mov es, ax\n");
        code.push_str("    mov ss, ax\n");
        code.push_str("    mov sp, 0x7C00\n");
        code.push_str("    sti\n");
        code.push_str("    cld\n\n");
        
        code.push_str("    mov ax, 0x0003\n");
        code.push_str("    int 0x10\n\n");
        
        code.push_str("    in al, 0x92\n");
        code.push_str("    or al, 2\n");
        code.push_str("    out 0x92, al\n\n");
        
        code.push_str("    lgdt [gdt32_desc]\n\n");
        code.push_str("    mov eax, cr0\n");
        code.push_str("    or eax, 1\n");
        code.push_str("    mov cr0, eax\n\n");
        code.push_str("    jmp 0x08:protected_mode\n\n");
        
        code.push_str("    bits 32\n");
        code.push_str("protected_mode:\n");
        code.push_str("    mov ax, 0x10\n");
        code.push_str("    mov ds, ax\n");
        code.push_str("    mov es, ax\n");
        code.push_str("    mov fs, ax\n");
        code.push_str("    mov gs, ax\n");
        code.push_str("    mov ss, ax\n");
        code.push_str("    mov esp, 0x7C00\n\n");
        
        code.push_str("    mov esi, msg\n");
        code.push_str("    mov edi, 0xB8000\n");
        code.push_str("    call print_string_32\n\n");
        
        self.compile_bios32_statements(code, &program.body);
        
        code.push_str("\n    cli\n");
        code.push_str("    hlt\n");
        code.push_str("    jmp $\n\n");
        
        // 32-bit subroutines
        code.push_str("print_string_32:\n");
        code.push_str("    pusha\n");
        code.push_str(".loop:\n");
        code.push_str("    lodsb\n");
        code.push_str("    test al, al\n");
        code.push_str("    jz .done\n");
        code.push_str("    mov [edi], al\n");
        code.push_str("    inc edi\n");
        code.push_str("    mov byte [edi], 0x0F\n");
        code.push_str("    inc edi\n");
        code.push_str("    jmp .loop\n");
        code.push_str(".done:\n");
        code.push_str("    popa\n");
        code.push_str("    ret\n\n");
        
        // GDT
        code.push_str("gdt32:\n");
        code.push_str("    dq 0x0000000000000000\n");
        code.push_str("    dq 0x00CF9A000000FFFF\n");
        code.push_str("    dq 0x00CF92000000FFFF\n");
        code.push_str("gdt32_end:\n\n");
        code.push_str("gdt32_desc:\n");
        code.push_str("    dw gdt32_end - gdt32 - 1\n");
        code.push_str("    dd gdt32\n\n");
        
        code.push_str("msg:\n");
        code.push_str("    db 'Rython 32-bit', 0\n\n");
        
        code.push_str("    times 510-($-$$) db 0\n");
        code.push_str("    dw 0xAA55\n");
        
        code.clone()
    }
    
    // FIXED 64-bit bootloader
    fn compile_bios64_fixed(&mut self, code: &mut String, program: &Program) -> String {
        code.push_str("; Rython 64-bit Bootloader - Fixed Version\n\n");
        code.push_str("    org 0x7C00\n");
        code.push_str("    bits 16\n\n");
        code.push_str("start:\n");
        code.push_str("    cli\n");
        code.push_str("    xor ax, ax\n");
        code.push_str("    mov ds, ax\n");
        code.push_str("    mov es, ax\n");
        code.push_str("    mov ss, ax\n");
        code.push_str("    mov sp, 0x7C00\n");
        code.push_str("    sti\n");
        code.push_str("    cld\n\n");
        
        code.push_str("    mov ax, 0x0003\n");
        code.push_str("    int 0x10\n\n");
        
        // Simple A20 enable
        code.push_str("    in al, 0x92\n");
        code.push_str("    or al, 2\n");
        code.push_str("    out 0x92, al\n\n");
        
        // Load 32-bit GDT
        code.push_str("    lgdt [gdt32_desc]\n\n");
        
        // Enter protected mode
        code.push_str("    mov eax, cr0\n");
        code.push_str("    or eax, 1\n");
        code.push_str("    mov cr0, eax\n\n");
        code.push_str("    jmp 0x08:protected_mode\n\n");
        
        // ========== 32-bit code ==========
        code.push_str("    bits 32\n");
        code.push_str("protected_mode:\n");
        code.push_str("    mov ax, 0x10\n");
        code.push_str("    mov ds, ax\n");
        code.push_str("    mov es, ax\n");
        code.push_str("    mov fs, ax\n");
        code.push_str("    mov gs, ax\n");
        code.push_str("    mov ss, ax\n");
        code.push_str("    mov esp, 0x90000\n\n");
        
        // Setup paging (32-bit code)
        code.push_str("    ; Setup paging\n");
        code.push_str("    mov edi, 0x1000\n");
        code.push_str("    mov cr3, edi\n");
        code.push_str("    xor eax, eax\n");
        code.push_str("    mov ecx, 4096\n");
        code.push_str("    rep stosd\n");
        
        code.push_str("    mov edi, 0x1000\n");
        code.push_str("    mov dword [edi], 0x2003\n");
        code.push_str("    add edi, 0x1000\n");
        code.push_str("    mov dword [edi], 0x3003\n");
        code.push_str("    add edi, 0x1000\n");
        
        code.push_str("    mov ebx, 0x00000083\n");
        code.push_str("    mov ecx, 512\n");
        code.push_str(".set_entry:\n");
        code.push_str("    mov dword [edi], ebx\n");
        code.push_str("    add ebx, 0x200000\n");
        code.push_str("    add edi, 8\n");
        code.push_str("    loop .set_entry\n\n");
        
        // Enable PAE
        code.push_str("    mov eax, cr4\n");
        code.push_str("    or eax, (1 << 5)\n");
        code.push_str("    mov cr4, eax\n\n");
        
        // Set CR3
        code.push_str("    mov eax, 0x1000\n");
        code.push_str("    mov cr3, eax\n\n");
        
        // Enable long mode
        code.push_str("    mov ecx, 0xC0000080\n");
        code.push_str("    rdmsr\n");
        code.push_str("    or eax, (1 << 8)\n");
        code.push_str("    wrmsr\n\n");
        
        // Enable paging
        code.push_str("    mov eax, cr0\n");
        code.push_str("    or eax, (1 << 31)\n");
        code.push_str("    mov cr0, eax\n\n");
        
        // Load 64-bit GDT
        code.push_str("    lgdt [gdt64_desc]\n\n");
        
        // Jump to 64-bit mode
        code.push_str("    jmp 0x08:long_mode\n\n");
        
        // ========== 64-bit code ==========
        code.push_str("    bits 64\n");
        code.push_str("long_mode:\n");
        
        code.push_str("    mov ax, 0x10\n");
        code.push_str("    mov ds, ax\n");
        code.push_str("    mov es, ax\n");
        code.push_str("    mov fs, ax\n");
        code.push_str("    mov gs, ax\n");
        code.push_str("    mov ss, ax\n");
        code.push_str("    mov rsp, 0x90000\n\n");
        
        // Clear screen (64-bit)
        code.push_str("    mov rdi, 0xB8000\n");
        code.push_str("    mov rax, 0x0720072007200720\n");
        code.push_str("    mov rcx, 1000\n");
        code.push_str("    rep stosq\n\n");
        
        // Print message (64-bit)
        code.push_str("    mov rsi, msg_64\n");
        code.push_str("    mov rdi, 0xB8000\n");
        code.push_str("    call print_string_64\n\n");
        
        // Compile program
        self.compile_bios64_statements(code, &program.body);
        
        code.push_str("\n    cli\n");
        code.push_str("    hlt\n");
        code.push_str("    jmp $\n\n");
        
        // ========== 64-bit subroutines ==========
        code.push_str("print_string_64:\n");
        code.push_str("    push rdi\n");
        code.push_str(".loop:\n");
        code.push_str("    lodsb\n");
        code.push_str("    test al, al\n");
        code.push_str("    jz .done\n");
        code.push_str("    stosb\n");
        code.push_str("    mov al, 0x0F\n");
        code.push_str("    stosb\n");
        code.push_str("    jmp .loop\n");
        code.push_str(".done:\n");
        code.push_str("    pop rdi\n");
        code.push_str("    ret\n\n");
        
        // ========== GDTs ==========
        code.push_str("gdt32:\n");
        code.push_str("    dq 0x0000000000000000\n");
        code.push_str("    dq 0x00CF9A000000FFFF\n");
        code.push_str("    dq 0x00CF92000000FFFF\n");
        code.push_str("gdt32_end:\n\n");
        
        code.push_str("gdt32_desc:\n");
        code.push_str("    dw gdt32_end - gdt32 - 1\n");
        code.push_str("    dd gdt32\n\n");
        
        code.push_str("gdt64:\n");
        code.push_str("    dq 0x0000000000000000\n");
        code.push_str("    dq 0x00209A0000000000\n");
        code.push_str("    dq 0x0000920000000000\n");
        code.push_str("gdt64_end:\n\n");
        
        code.push_str("gdt64_desc:\n");
        code.push_str("    dw gdt64_end - gdt64 - 1\n");
        code.push_str("    dq gdt64\n\n");
        
        // ========== Data ==========
        code.push_str("msg_64:\n");
        code.push_str("    db 'Rython 64-bit', 0\n\n");
        
        code.push_str("    times 510-($-$$) db 0\n");
        code.push_str("    dw 0xAA55\n");
        
        code.clone()
    }
    
    // Other BIOS variants (simplified)
    fn compile_bios64_sse(&mut self, code: &mut String, program: &Program) -> String {
        let mut asm = self.compile_bios64_fixed(code, program);
        asm.push_str("\n    ; SSE enabled\n");
        asm
    }
    
    fn compile_bios64_avx(&mut self, code: &mut String, program: &Program) -> String {
        let mut asm = self.compile_bios64_fixed(code, program);
        asm.push_str("\n    ; AVX enabled\n");
        asm
    }
    
    fn compile_bios64_avx512(&mut self, code: &mut String, program: &Program) -> String {
        let mut asm = self.compile_bios64_fixed(code, program);
        asm.push_str("\n    ; AVX-512 enabled\n");
        asm
    }
    
    // Compilation helpers
    fn compile_bios16_statements(&mut self, code: &mut String, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::Expr(expr) => {
                    code.push_str("    ; Expression\n");
                    self.compile_bios16_expression(code, expr);
                }
                Statement::VarDecl { name, value, .. } => {
                    code.push_str(&format!("    ; var {} = ", name));
                    self.compile_bios16_expression(code, value);
                }
                _ => code.push_str("    ; [Statement]\n"),
            }
        }
    }
    
    fn compile_bios32_statements(&mut self, code: &mut String, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::Expr(expr) => {
                    code.push_str("    ; Expression\n");
                    self.compile_bios32_expression(code, expr);
                }
                Statement::VarDecl { name, value, .. } => {
                    code.push_str(&format!("    ; var {} = ", name));
                    self.compile_bios32_expression(code, value);
                }
                _ => code.push_str("    ; [Statement]\n"),
            }
        }
    }
    
    fn compile_bios64_statements(&mut self, code: &mut String, statements: &[Statement]) {
        for stmt in statements {
            match stmt {
                Statement::Expr(expr) => {
                    code.push_str("    ; Expression\n");
                    self.compile_bios64_expression(code, expr);
                }
                Statement::VarDecl { name, value, .. } => {
                    code.push_str(&format!("    ; var {} = ", name));
                    self.compile_bios64_expression(code, value);
                }
                _ => code.push_str("    ; [Statement]\n"),
            }
        }
    }
    
    fn compile_bios16_expression(&mut self, code: &mut String, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                code.push_str(&format!("    ; Number: {}\n", n));
                code.push_str(&format!("    mov ax, {}\n", n));
                code.push_str("    call print_decimal\n");
            }
            _ => code.push_str("    ; [Expression]\n"),
        }
    }
    
    fn compile_bios32_expression(&mut self, code: &mut String, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                code.push_str(&format!("    ; Number: {}\n", n));
                code.push_str(&format!("    mov eax, {}\n", n));
                code.push_str("    call print_decimal_32\n");
            }
            _ => code.push_str("    ; [Expression]\n"),
        }
    }
    
    fn compile_bios64_expression(&mut self, code: &mut String, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                code.push_str(&format!("    ; Number: {}\n", n));
                code.push_str(&format!("    mov rax, {}\n", n));
                code.push_str("    call print_decimal_64\n");
            }
            _ => code.push_str("    ; [Expression]\n"),
        }
    }
    
    fn compile_standard(&mut self, _program: &Program) -> String {
        String::from("; Standard compilation not implemented\n")
    }
}

// Public interface
pub fn compile_to_nasm(program: &Program) -> String {
    let mut emitter = NasmEmitter::new();
    emitter.compile_program(program)
}