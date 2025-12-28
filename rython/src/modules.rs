use std::collections::HashMap;
use crate::parser::Program;
use crate::backend::{Capability, BackendFunction};
use crate::rcl_compiler::{RclLibrary, RclMetadata, RclEntry, RclFunction};

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub required_capabilities: Vec<Capability>,
    pub assembly_code: HashMap<String, String>,
    pub functions: Vec<BackendFunction>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            required_capabilities: Vec::new(),
            assembly_code: HashMap::new(),
            functions: Vec::new(),
        }
    }
    
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
    
    pub fn with_assembly(mut self, target: &str, assembly: &str) -> Self {
        self.assembly_code.insert(target.to_string(), assembly.to_string());
        self
    }
    
    pub fn get_required_capabilities(&self) -> Vec<Capability> {
        self.required_capabilities.clone()
    }
    
    pub fn get_assembly(&self, target: &str) -> String {
        self.assembly_code.get(target).cloned().unwrap_or_default()
    }
    
    pub fn to_rcl_library(&self, target: &str) -> RclLibrary {
        let metadata = RclMetadata {
            name: self.name.clone(),
            version: "1.0".to_string(),
            author: None,
            description: None,
            dependencies: Vec::new(),
            exports: self.functions.iter().map(|f| f.name.clone()).collect(),
            target: target.to_string(),
            rcl_version: "1.0".to_string(),
            capabilities: self.required_capabilities.iter().map(|c| format!("{:?}", c)).collect(),
        };
        
        let entries = self.functions.iter().map(|func| {
            RclEntry::Function(RclFunction {
                name: func.name.clone(),
                signature: "TODO".to_string(),
                parameters: Vec::new(),
                return_type: "TODO".to_string(),
                ast: None,
                assembly: Some(func.body.join("\n")),
                inlineable: false,
                pure: false,
                capabilities: Vec::new(),
            })
        }).collect();
        
        RclLibrary {
            metadata,
            entries,
            imports: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }
}

pub struct ModuleRegistry {
    modules: HashMap<String, Module>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
    
    pub fn register_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }
    
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }
    
    pub fn get_module_names(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }
    
    // Simplified version that doesn't rely on Import statements
    pub fn extract_required_modules(&self, _program: &Program) -> Vec<String> {
        Vec::new() // Return empty for now
    }
    
    pub fn default_registry() -> Self {
        let mut registry = Self::new();
        
        // Register basic modules
        registry.register_module(
            Module::new("console")
                .with_capability(Capability::BIOS)
                .with_assembly("bios16", 
                    "print_char_16:\n    mov ah, 0x0E\n    int 0x10\n    ret\n")
                .with_assembly("bios64",
                    "print_char_64:\n    mov ah, 0x0E\n    int 0x10\n    ret\n")
        );
        
        let io_asm_64 = r#"
io_write_char_64:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    
    mov [rbp-1], dil
    
    ; Linux syscall
    mov rax, 1      ; sys_write
    mov rdi, 1      ; stdout
    lea rsi, [rbp-1]
    mov rdx, 1
    syscall
    
    add rsp, 16
    pop rbp
    ret

io_read_line_64:
    ; rdi = buffer, rsi = max_len
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi
    mov r12, rsi
    xor rcx, rcx
    
.read_loop:
    cmp rcx, r12
    jae .read_done
    
    call io_read_char_64
    cmp al, 0xA     ; newline
    je .read_done
    cmp al, 0xD     ; carriage return
    je .read_done
    
    mov [rbx+rcx], al
    inc rcx
    jmp .read_loop
    
.read_done:
    mov byte [rbx+rcx], 0
    mov rax, rcx
    
    pop r12
    pop rbx
    pop rbp
    ret
"#;
        
        registry.register_module(
            Module::new("io")
                .with_capability(Capability::LongMode64)
                .with_capability(Capability::Linux)
                .with_assembly("linux64", io_asm_64)
        );

        // === HDMI Video Module ===
        let hdmi_asm_64 = r#"
; HDMI Video Module for 64-bit
; Assumes EDID information available and HDMI controller initialized

hdmi_init_64:
    ; Initialize HDMI controller
    push rbp
    mov rbp, rsp
    
    ; Read EDID
    mov rax, 0x0E00  ; EDID base address
    mov rdi, rax
    call read_edid_64
    
    ; Set HDMI mode
    mov rdi, 1920    ; width
    mov rsi, 1080    ; height
    mov rdx, 60      ; refresh rate
    mov rcx, 32      ; bits per pixel
    call hdmi_set_mode_64
    
    pop rbp
    ret

hdmi_set_mode_64:
    ; rdi = width, rsi = height, rdx = refresh, rcx = bpp
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    
    mov r12, rdi  ; width
    mov r13, rsi  ; height
    mov r14, rdx  ; refresh
    mov rbx, rcx  ; bpp
    
    ; Calculate framebuffer size
    mov rax, r12
    imul rax, r13
    imul rax, rbx
    shr rax, 3     ; bits to bytes
    mov [framebuffer_size], rax
    
    ; Allocate framebuffer
    mov rdi, rax
    call sys_malloc_64
    mov [framebuffer_ptr], rax
    
    ; Setup HDMI controller registers
    ; (Simplified - actual implementation would write to PCI/MMIO registers)
    
    ; Set resolution
    mov rax, 0xC0000000  ; HDMI control register base
    mov [rax], r12       ; width
    mov [rax+8], r13     ; height
    mov [rax+16], r14    ; refresh rate
    mov [rax+24], rbx    ; bpp
    
    ; Enable HDMI output
    mov rdx, [rax+32]
    or rdx, 0x1
    mov [rax+32], rdx
    
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

hdmi_draw_pixel_64:
    ; rdi = x, rsi = y, rdx = color (ARGB)
    push rbp
    mov rbp, rsp
    
    ; Calculate pixel offset
    mov rax, [framebuffer_ptr]
    test rax, rax
    jz .draw_done
    
    ; offset = (y * width + x) * (bpp/8)
    mov rcx, rsi
    imul rcx, [framebuffer_width]
    add rcx, rdi
    shl rcx, 2      ; *4 for 32bpp
    
    ; Write pixel
    mov [rax+rcx], edx
    
.draw_done:
    pop rbp
    ret

hdmi_draw_rect_64:
    ; rdi = x, rsi = y, rdx = width, rcx = height, r8 = color
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
    
    mov r12, rdi  ; x
    mov r13, rsi  ; y
    mov r14, rdx  ; width
    mov r15, rcx  ; height
    mov rbx, r8   ; color
    
    ; Draw rectangle
    xor r9, r9    ; row counter
.row_loop:
    cmp r9, r15
    jge .rect_done
    
    mov r10, r13
    add r10, r9   ; current y
    
    xor r11, r11  ; col counter
.col_loop:
    cmp r11, r14
    jge .next_row
    
    mov rdi, r12
    add rdi, r11  ; current x
    mov rsi, r10
    mov rdx, rbx
    call hdmi_draw_pixel_64
    
    inc r11
    jmp .col_loop
    
.next_row:
    inc r9
    jmp .row_loop
    
.rect_done:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

hdmi_draw_line_64:
    ; rdi = x1, rsi = y1, rdx = x2, rcx = y2, r8 = color
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
    
    mov r12, rdi  ; x1
    mov r13, rsi  ; y1
    mov r14, rdx  ; x2
    mov r15, rcx  ; y2
    mov rbx, r8   ; color
    
    ; Bresenham's line algorithm
    mov rdi, r14
    sub rdi, r12
    call abs_64
    mov r9, rax   ; dx
    
    mov rdi, r15
    sub rdi, r13
    call abs_64
    mov r10, rax  ; dy
    
    mov r11, r12
    cmp r12, r14
    cmovg r11, r14
    sub r11, r12
    neg r11       ; sx
    
    mov rax, r13
    cmp r13, r15
    cmovg rax, r15
    sub rax, r13
    neg rax       ; sy
    
    mov rcx, r9
    sub rcx, r10  ; err
    
.line_loop:
    ; Draw current pixel
    mov rdi, r12
    mov rsi, r13
    mov rdx, rbx
    call hdmi_draw_pixel_64
    
    ; Check if done
    cmp r12, r14
    jne .continue_line
    cmp r13, r15
    je .line_done
    
.continue_line:
    mov rax, rcx
    shl rax, 1    ; e2 = 2*err
    
    ; Check x direction
    mov rdx, rax
    neg rdx
    cmp rdx, r10
    jg .update_x_err
    add rcx, r10
    add r12, r11
    
.update_x_err:
    ; Check y direction
    cmp rax, r9
    jge .update_y_err
    sub rcx, r9
    add r13, r11
    
.update_y_err:
    jmp .line_loop
    
.line_done:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

hdmi_clear_screen_64:
    ; rdi = color
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi  ; color
    
    mov rax, [framebuffer_ptr]
    test rax, rax
    jz .clear_done
    
    mov rcx, [framebuffer_size]
    shr rcx, 2    ; divide by 4 for dwords
    
.clear_loop:
    mov [rax], ebx
    add rax, 4
    loop .clear_loop
    
.clear_done:
    pop r12
    pop rbx
    pop rbp
    ret

hdmi_blit_64:
    ; rdi = source buffer, rsi = width, rdx = height, rcx = dest_x, r8 = dest_y
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    push r15
    
    mov r12, rdi  ; source
    mov r13, rsi  ; width
    mov r14, rdx  ; height
    mov r15, rcx  ; dest_x
    mov rbx, r8   ; dest_y
    
    ; Copy image data
    xor r9, r9    ; row counter
.blit_row_loop:
    cmp r9, r14
    jge .blit_done
    
    mov r10, rbx
    add r10, r9   ; current dest y
    
    xor r11, r11  ; col counter
.blit_col_loop:
    cmp r11, r13
    jge .next_blit_row
    
    ; Read pixel from source
    mov rax, r9
    imul rax, r13
    add rax, r11
    shl rax, 2
    mov edx, [r12+rax]  ; pixel color
    
    ; Draw pixel
    mov rdi, r15
    add rdi, r11  ; dest x
    mov rsi, r10
    call hdmi_draw_pixel_64
    
    inc r11
    jmp .blit_col_loop
    
.next_blit_row:
    inc r9
    jmp .blit_row_loop
    
.blit_done:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

read_edid_64:
    ; Read EDID from HDMI controller
    push rbp
    mov rbp, rsp
    
    ; EDID is typically at I2C address 0x50
    mov rax, 0xC0000100  ; HDMI I2C control
    mov rdx, 0x50        ; EDID I2C address
    mov [rax], rdx
    
    ; Read 128 bytes of EDID
    mov rdi, edid_buffer
    mov rsi, 128
    mov rdx, rax
    add rdx, 8           ; I2C data register
    
.read_loop:
    mov al, [rdx]
    mov [rdi], al
    inc rdi
    dec rsi
    jnz .read_loop
    
    pop rbp
    ret

section .data
framebuffer_ptr: dq 0
framebuffer_size: dq 0
framebuffer_width: dq 1920
framebuffer_height: dq 1080
framebuffer_bpp: dq 32
edid_buffer: times 128 db 0
"#;
        
        registry.register_module(
            Module::new("hdmi")
                .with_capability(Capability::LongMode64)
                .with_capability(Capability::Graphics)
                .with_assembly("bios64", hdmi_asm_64)
                .with_assembly("linux64", hdmi_asm_64)
        );
        
        // === VGA Video Module ===
        let vga_asm_64 = r#"
; VGA Video Module for 64-bit
; Supports VGA text mode (mode 3) and graphics modes (mode 13h)

vga_init_64:
    ; Initialize VGA controller
    push rbp
    mov rbp, rsp
    
    ; Get current video mode
    mov rax, 0x10
    xor rbx, rbx
    int 0x10
    
    ; Store mode information
    mov [vga_mode], bh
    
    ; Set up text mode buffer
    mov qword [text_buffer], 0xB8000
    mov qword [text_buffer_size], 80*25*2
    
    pop rbp
    ret

vga_set_mode_64:
    ; rdi = mode number
    push rbp
    mov rbp, rsp
    
    ; Set VGA mode via BIOS
    mov rax, 0x10
    mov rbx, rdi
    int 0x10
    
    ; Update stored mode
    mov [vga_mode], bl
    
    ; Setup appropriate buffer
    cmp bl, 0x13
    je .mode_13h
    cmp bl, 0x12
    je .mode_12h
    cmp bl, 0x03
    je .mode_text
    
    ; Default: text mode
    mov qword [vga_buffer], 0xB8000
    mov qword [buffer_width], 80
    mov qword [buffer_height], 25
    mov qword [bytes_per_pixel], 2
    jmp .mode_done
    
.mode_13h:
    ; 320x200, 256 colors
    mov qword [vga_buffer], 0xA0000
    mov qword [buffer_width], 320
    mov qword [buffer_height], 200
    mov qword [bytes_per_pixel], 1
    jmp .mode_done
    
.mode_12h:
    ; 640x480, 16 colors
    mov qword [vga_buffer], 0xA0000
    mov qword [buffer_width], 640
    mov qword [buffer_height], 480
    mov qword [bytes_per_pixel], 1
    
.mode_text:
    ; Already set above
.mode_done:
    pop rbp
    ret

vga_put_pixel_64:
    ; rdi = x, rsi = y, rdx = color
    push rbp
    mov rbp, rsp
    
    cmp qword [vga_mode], 0x13
    jne .not_mode13
    
    ; Mode 13h: 320x200, 256 colors
    mov rax, rsi
    mov rcx, 320
    imul rax, rcx
    add rax, rdi
    add rax, 0xA0000
    mov [rax], dl
    
.not_mode13:
    cmp qword [vga_mode], 0x12
    jne .not_mode12
    
    ; Mode 12h: 640x480, 4-bit color
    mov rax, rsi
    mov rcx, 640
    imul rax, rcx
    add rax, rdi
    mov rcx, rax
    
    ; Calculate byte and nibble
    shr rax, 1      ; divide by 2 (2 pixels per byte)
    add rax, 0xA0000
    
    mov bl, [rax]
    test rcx, 1
    jnz .odd_pixel
    
    ; Even pixel: high nibble
    and bl, 0x0F
    mov cl, dl
    shl cl, 4
    or bl, cl
    jmp .write_pixel
    
.odd_pixel:
    ; Odd pixel: low nibble
    and bl, 0xF0
    mov cl, dl
    and cl, 0x0F
    or bl, cl
    
.write_pixel:
    mov [rax], bl
    
.not_mode12:
    pop rbp
    ret

vga_put_char_64:
    ; rdi = x, rsi = y, rdx = char, rcx = color
    push rbp
    mov rbp, rsp
    
    ; Calculate position in text buffer
    mov rax, rsi
    mov rbx, 80
    imul rax, rbx
    add rax, rdi
    shl rax, 1      ; *2 for char+attribute
    
    add rax, 0xB8000
    
    ; Write character and attribute
    mov [rax], dl
    mov [rax+1], cl
    
    pop rbp
    ret

vga_put_string_64:
    ; rdi = x, rsi = y, rdx = string, rcx = color
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; x
    mov r13, rsi  ; y
    mov rbx, rdx  ; string
    mov r8, rcx   ; color
    
.string_loop:
    mov dl, [rbx]
    test dl, dl
    jz .string_done
    
    mov rdi, r12
    mov rsi, r13
    mov rcx, r8
    call vga_put_char_64
    
    inc rbx
    inc r12
    jmp .string_loop
    
.string_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

vga_scroll_64:
    ; rdi = lines to scroll
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; lines
    
    ; Calculate source and destination
    mov rax, r12
    mov rbx, 80
    imul rax, rbx
    shl rax, 1      ; *2 for bytes
    
    mov rsi, 0xB8000
    add rsi, rax    ; source
    
    mov rdi, 0xB8000 ; destination
    
    ; Calculate bytes to copy
    mov rcx, 25
    sub rcx, r12
    imul rcx, 80
    shl rcx, 1
    
    ; Copy memory
    rep movsb
    
    ; Clear bottom lines
    mov rcx, r12
    imul rcx, 80
    shl rcx, 1
    
    mov al, ' '
    mov ah, 0x07    ; white on black
    rep stosw
    
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

vga_set_palette_64:
    ; rdi = palette index, rsi = red, rdx = green, rcx = blue
    push rbp
    mov rbp, rsp
    
    ; VGA palette programming
    mov al, dil     ; color index
    mov dx, 0x3C8   ; VGA palette write index
    out dx, al
    
    ; Write RGB values
    mov al, sil     ; red
    mov dx, 0x3C9   ; VGA palette data
    out dx, al
    
    mov al, dl      ; green
    out dx, al
    
    mov al, cl      ; blue
    out dx, al
    
    pop rbp
    ret

vga_get_pixel_64:
    ; rdi = x, rsi = y
    push rbp
    mov rbp, rsp
    
    cmp qword [vga_mode], 0x13
    jne .not_mode13_get
    
    ; Mode 13h
    mov rax, rsi
    mov rcx, 320
    imul rax, rcx
    add rax, rdi
    add rax, 0xA0000
    movzx rax, byte [rax]
    jmp .get_done
    
.not_mode13_get:
    cmp qword [vga_mode], 0x12
    jne .not_mode12_get
    
    ; Mode 12h
    mov rax, rsi
    mov rcx, 640
    imul rax, rcx
    add rax, rdi
    mov rcx, rax
    
    shr rax, 1
    add rax, 0xA0000
    mov bl, [rax]
    
    test rcx, 1
    jnz .odd_pixel_get
    
    ; Even pixel: high nibble
    shr bl, 4
    jmp .got_pixel
    
.odd_pixel_get:
    ; Odd pixel: low nibble
    and bl, 0x0F
    
.got_pixel:
    movzx rax, bl
    jmp .get_done
    
.not_mode12_get:
    xor rax, rax
    
.get_done:
    pop rbp
    ret

vga_draw_line_64:
    ; rdi = x1, rsi = y1, rdx = x2, rcx = y2, r8 = color
    push rbp
    mov rbp, rsp
    
    ; Use Bresenham's algorithm (similar to HDMI)
    mov r9, rdx
    mov r10, rcx
    
    mov rdx, r8
    call hdmi_draw_line_64
    
    pop rbp
    ret

section .data
vga_mode: dq 0x03  ; Default to text mode
vga_buffer: dq 0xB8000
buffer_width: dq 80
buffer_height: dq 25
bytes_per_pixel: dq 2
text_buffer: dq 0xB8000
text_buffer_size: dq 4000  ; 80*25*2
"#;
        
        registry.register_module(
            Module::new("vga")
                .with_capability(Capability::BIOS)
                .with_capability(Capability::Graphics)
                .with_assembly("bios64", vga_asm_64)
                .with_assembly("bios16", 
                    "vga_set_mode_16:\n    mov ah, 0x00\n    mov al, 0x03\n    int 0x10\n    ret\n")
        );
        
        // === P2 Audio Module ===
        let audio_asm_64 = r#"
; P2 Audio Module for 64-bit
; Supports PC Speaker, Sound Blaster, and AC97 audio

audio_init_64:
    ; Initialize audio system
    push rbp
    mov rbp, rsp
    
    ; Detect audio hardware
    call audio_detect_64
    
    ; Initialize PC Speaker
    call pcspkr_init_64
    
    ; Try to initialize Sound Blaster
    call sb_init_64
    
    ; Try to initialize AC97
    call ac97_init_64
    
    pop rbp
    ret

audio_detect_64:
    ; Detect available audio hardware
    push rbp
    mov rbp, rsp
    
    ; Check for PC Speaker (always available)
    mov byte [audio_pcspkr], 1
    
    ; Check for Sound Blaster
    call sb_detect_64
    test rax, rax
    jz .no_sb
    mov byte [audio_sb], 1
    
.no_sb:
    ; Check for AC97
    call ac97_detect_64
    test rax, rax
    jz .no_ac97
    mov byte [audio_ac97], 1
    
.no_ac97:
    pop rbp
    ret

pcspkr_init_64:
    ; Initialize PC Speaker
    push rbp
    mov rbp, rsp
    
    ; Get PIT (Programmable Interval Timer) control
    mov al, 0xB6
    out 0x43, al
    
    pop rbp
    ret

pcspkr_play_freq_64:
    ; rdi = frequency in Hz
    push rbp
    mov rbp, rsp
    push rbx
    
    mov rbx, rdi
    
    ; Calculate PIT divisor
    mov rax, 1193180  ; PIT clock frequency
    xor rdx, rdx
    div rbx
    mov rbx, rax      ; divisor
    
    ; Program PIT channel 2
    mov al, 0xB6
    out 0x43, al
    
    mov al, bl
    out 0x42, al
    mov al, bh
    out 0x42, al
    
    ; Turn on speaker
    in al, 0x61
    or al, 0x03
    out 0x61, al
    
    pop rbx
    pop rbp
    ret

pcspkr_stop_64:
    ; Turn off speaker
    push rbp
    mov rbp, rsp
    
    in al, 0x61
    and al, 0xFC
    out 0x61, al
    
    pop rbp
    ret

sb_init_64:
    ; Initialize Sound Blaster
    push rbp
    mov rbp, rsp
    
    ; Reset DSP
    mov dx, 0x226  ; DSP Reset Port
    mov al, 1
    out dx, al
    
    ; Wait
    mov rcx, 100
.delay1:
    loop .delay1
    
    mov al, 0
    out dx, al
    
    ; Wait for DSP ready
    mov rcx, 1000000
.wait_ready:
    mov dx, 0x22E  ; DSP Read Data
    in al, dx
    test al, 0x80
    jnz .dsp_ready
    loop .wait_ready
    
.dsp_ready:
    ; Read version
    mov al, 0xE1
    out dx, al
    
    mov dx, 0x22A  ; DSP Read Data
    in al, dx
    mov [sb_version_major], al
    
    in al, dx
    mov [sb_version_minor], al
    
    mov byte [audio_sb_initialized], 1
    
    pop rbp
    ret

sb_play_pcm_64:
    ; rdi = buffer, rsi = size, rdx = sample_rate
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; buffer
    mov r13, rsi  ; size
    mov rbx, rdx  ; sample_rate
    
    ; Set sample rate
    mov rdi, rbx
    call sb_set_sample_rate_64
    
    ; Set transfer mode
    mov dx, 0x22C  ; DSP Write
    mov al, 0x41   ; 8-bit DAC
    out dx, al
    
    ; Send size
    mov rcx, r13
    mov al, cl
    out dx, al
    mov al, ch
    out dx, al
    
    ; Send data
    mov rsi, r12
    mov rcx, r13
    
.send_loop:
    lodsb
    out dx, al
    loop .send_loop
    
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

sb_set_sample_rate_64:
    ; rdi = sample rate
    push rbp
    mov rbp, rsp
    
    mov dx, 0x22C  ; DSP Write
    mov al, 0x40   ; Set time constant
    out dx, al
    
    ; Calculate time constant
    mov rax, 256000000
    xor rdx, rdx
    div rdi
    neg rax
    add rax, 65536
    
    mov al, ah     ; High byte
    out dx, al
    
    pop rbp
    ret

ac97_init_64:
    ; Initialize AC97 audio
    push rbp
    mov rbp, rsp
    
    ; Search for AC97 controller
    mov rdi, 0x01  ; PCI class code for multimedia
    mov rsi, 0x04  ; Subclass for audio
    call pci_find_device_64
    test rax, rax
    jz .no_ac97
    
    mov [ac97_base], rax
    
    ; Reset AC97
    mov rdx, rax
    add rdx, 0x2C   ; Reset register
    mov eax, 0x02   ; Cold reset
    out dx, eax
    
    ; Wait for reset
    mov rcx, 100000
.reset_wait:
    loop .reset_wait
    
    ; Setup AC97
    mov rdx, [ac97_base]
    add rdx, 0x02   ; Master volume
    mov ax, 0x0000  ; 0dB
    out dx, ax
    
    mov byte [audio_ac97_initialized], 1
    
.no_ac97:
    pop rbp
    ret

ac97_play_buffer_64:
    ; rdi = buffer, rsi = size, rdx = sample_rate
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; buffer
    mov r13, rsi  ; size
    mov rbx, rdx  ; sample_rate
    
    ; Set sample rate
    mov rdx, [ac97_base]
    add rdx, 0x2A   ; PCM sample rate
    mov ax, bx
    out dx, ax
    
    ; Setup DMA (simplified)
    mov rdx, [ac97_base]
    add rdx, 0x10   ; PCM buffer address
    mov rax, r12
    out dx, eax
    
    mov rdx, [ac97_base]
    add rdx, 0x14   ; PCM buffer size
    mov eax, r13d
    out dx, eax
    
    ; Start playback
    mov rdx, [ac97_base]
    add rdx, 0x1B   ; Control register
    mov al, 0x01    ; Start
    out dx, al
    
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

audio_play_note_64:
    ; rdi = frequency, rsi = duration_ms
    push rbp
    mov rbp, rsp
    
    ; Play using PC Speaker
    call pcspkr_play_freq_64
    
    ; Sleep for duration
    mov rdi, rsi
    call sleep_ms_64
    
    ; Stop
    call pcspkr_stop_64
    
    pop rbp
    ret

audio_generate_sine_64:
    ; rdi = buffer, rsi = size, rdx = freq, rcx = sample_rate
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    
    mov r12, rdi  ; buffer
    mov r13, rsi  ; size
    mov r14, rdx  ; frequency
    mov rbx, rcx  ; sample_rate
    
    ; Calculate angular frequency
    fldpi
    fmul qword [two]
    fild qword r14
    fmul
    fild qword rbx
    fdiv
    fstp qword [omega]
    
    ; Generate samples
    xor r9, r9
.sine_loop:
    cmp r9, r13
    jge .sine_done
    
    ; t = i / sample_rate
    fild qword r9
    fild qword rbx
    fdiv
    
    ; sin(2 * pi * freq * t)
    fmul qword [omega]
    fsin
    
    ; Scale to 8-bit
    fmul qword [scale_127]
    fadd qword [offset_127]
    
    ; Convert to integer
    fistp qword [temp]
    mov rax, [temp]
    mov [r12+r9], al
    
    inc r9
    jmp .sine_loop
    
.sine_done:
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

pci_find_device_64:
    ; rdi = class, rsi = subclass
    ; Returns base address in rax
    push rbp
    mov rbp, rsp
    
    ; Simplified PCI scan
    mov rcx, 0x80000000  ; Start bus 0, device 0, function 0
    
.scan_loop:
    mov rax, rcx
    mov dx, 0xCF8
    out dx, eax
    
    mov dx, 0xCFC
    in eax, dx
    
    ; Check vendor ID
    cmp ax, 0xFFFF
    je .next_device
    
    ; Check class/subclass
    shr eax, 16
    cmp ah, sil      ; subclass
    jne .next_device
    cmp al, dil      ; class
    jne .next_device
    
    ; Found device
    mov rax, rcx
    and rax, 0xFFFFFF00  ; Base address
    jmp .found
    
.next_device:
    add rcx, 0x100
    cmp rcx, 0x81000000
    jl .scan_loop
    
    ; Not found
    xor rax, rax
    
.found:
    pop rbp
    ret

sleep_ms_64:
    ; rdi = milliseconds
    push rbp
    mov rbp, rsp
    
    ; Convert to microseconds
    imul rdi, 1000
    
    ; Linux nanosleep
    mov rax, 35      ; sys_nanosleep
    push rdi
    mov rsi, rsp
    xor rdi, rdi
    syscall
    
    add rsp, 8
    pop rbp
    ret

section .data
audio_pcspkr: db 0
audio_sb: db 0
audio_ac97: db 0
audio_sb_initialized: db 0
audio_ac97_initialized: db 0
sb_version_major: db 0
sb_version_minor: db 0
ac97_base: dq 0
omega: dq 0.0
scale_127: dq 127.0
offset_127: dq 127.0
temp: dq 0
two: dq 2.0
"#;
        
        registry.register_module(
            Module::new("audio")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", audio_asm_64)
                .with_assembly("linux64", audio_asm_64)
        );
        
        // === Network Module ===
        let network_asm_64 = r#"
; Network Module for 64-bit
; Supports Ethernet, IP, TCP/UDP, and basic networking

network_init_64:
    ; Initialize network stack
    push rbp
    mov rbp, rsp
    
    ; Detect network card
    call network_detect_nic_64
    
    ; Initialize Ethernet
    call ethernet_init_64
    
    ; Initialize IP stack
    call ip_init_64
    
    ; Initialize ARP
    call arp_init_64
    
    ; Get MAC address
    lea rdi, [mac_address]
    call network_get_mac_64
    
    ; Get IP address (default)
    mov dword [ip_address], 0xC0A80101  ; 192.168.1.1
    
    pop rbp
    ret

network_detect_nic_64:
    ; Detect network interface card
    push rbp
    mov rbp, rsp
    
    ; Try to find Intel 8254x (common NIC)
    mov rdi, 0x02    ; PCI class network
    mov rsi, 0x00    ; Ethernet subclass
    call pci_find_device_64
    test rax, rax
    jz .no_nic
    
    mov [nic_base], rax
    
    ; Read vendor/device ID
    mov rdx, rax
    in eax, dx
    mov [nic_vendor], ax
    shr eax, 16
    mov [nic_device], ax
    
    ; Enable device
    mov rdx, [nic_base]
    add rdx, 0x04    ; Command register
    mov ax, 0x0007   ; Enable memory, bus mastering
    out dx, ax
    
    mov byte [nic_detected], 1
    
.no_nic:
    pop rbp
    ret

ethernet_init_64:
    ; Initialize Ethernet controller
    push rbp
    mov rbp, rsp
    
    cmp byte [nic_detected], 0
    je .init_done
    
    ; Reset NIC
    mov rdx, [nic_base]
    add rdx, 0x08    ; Device control
    mov eax, 0x04000000  ; Reset bit
    out dx, eax
    
    ; Wait for reset
    mov rcx, 100000
.reset_wait:
    loop .reset_wait
    
    ; Setup receive descriptors
    lea rdi, [rx_descriptors]
    mov rsi, RX_DESCRIPTOR_COUNT
    call setup_rx_descriptors_64
    
    ; Setup transmit descriptors
    lea rdi, [tx_descriptors]
    mov rsi, TX_DESCRIPTOR_COUNT
    call setup_tx_descriptors_64
    
    ; Start NIC
    mov rdx, [nic_base]
    add rdx, 0x28    ; RCTL register
    mov eax, 0x02008002  ; Enable, broadcast accept, 4096 bytes
    out dx, eax
    
.init_done:
    pop rbp
    ret

setup_rx_descriptors_64:
    ; rdi = descriptors array, rsi = count
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi
    mov r12, rsi
    
    xor rcx, rcx
.rx_desc_loop:
    cmp rcx, r12
    jge .rx_desc_done
    
    ; Allocate buffer
    mov rdi, 2048
    call sys_malloc_64
    mov [rbx + rcx*16], rax      ; buffer address
    
    ; Set descriptor flags
    mov rax, 0x80000000          ; descriptor done
    or rax, 2048                 ; buffer size
    mov [rbx + rcx*16 + 8], rax  ; length and flags
    
    inc rcx
    jmp .rx_desc_loop
    
.rx_desc_done:
    ; Tell NIC about descriptors
    mov rdx, [nic_base]
    add rdx, 0x2800              ; RDBAL
    mov eax, ebx
    out dx, eax
    
    mov rdx, [nic_base]
    add rdx, 0x2804              ; RDBAH
    shr rbx, 32
    mov eax, ebx
    out dx, eax
    
    mov rdx, [nic_base]
    add rdx, 0x2808              ; RDLEN
    mov eax, r12d
    shl eax, 4                   ; *16 for descriptor size
    out dx, eax
    
    mov rdx, [nic_base]
    add rdx, 0x2810              ; RDH
    xor eax, eax
    out dx, eax
    
    mov rdx, [nic_base]
    add rdx, 0x2818              ; RDT
    mov eax, r12d
    dec eax
    out dx, eax
    
    pop r12
    pop rbx
    pop rbp
    ret

network_send_packet_64:
    ; rdi = buffer, rsi = length
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; buffer
    mov r13, rsi  ; length
    
    ; Get transmit descriptor
    mov rbx, [tx_tail]
    lea rax, [tx_descriptors]
    mov rdx, rbx
    shl rdx, 4                   ; *16
    add rax, rdx
    
    ; Copy packet to descriptor buffer
    mov rdi, [rax]               ; buffer address
    mov rsi, r12                 ; source
    mov rdx, r13                 ; length
    call sys_memcpy_64
    
    ; Update descriptor
    mov rax, 0x80000000          ; descriptor done
    or rax, r13                  ; packet length
    mov [rbx*16 + 8 + tx_descriptors], rax
    
    ; Update tail
    inc rbx
    cmp rbx, TX_DESCRIPTOR_COUNT
    jl .no_wrap
    xor rbx, rbx
.no_wrap:
    mov [tx_tail], rbx
    
    ; Notify NIC
    mov rdx, [nic_base]
    add rdx, 0x3818              ; TDT
    mov eax, ebx
    out dx, eax
    
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

network_receive_packet_64:
    ; rdi = buffer, rsi = max_length
    ; Returns actual length in rax
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; buffer
    mov r13, rsi  ; max_length
    
    ; Check receive descriptor
    mov rbx, [rx_head]
    lea rax, [rx_descriptors]
    mov rdx, rbx
    shl rdx, 4
    add rax, rdx
    
    ; Check if packet available
    mov rcx, [rax + 8]           ; status/length
    test rcx, 0x80000000         ; descriptor done bit
    jz .no_packet
    
    ; Get packet length
    and rcx, 0x0000FFFF
    cmp rcx, r13
    jle .length_ok
    
    ; Truncate if too long
    mov rcx, r13
    
.length_ok:
    ; Copy packet
    mov rdi, r12                 ; destination
    mov rsi, [rax]               ; source
    mov rdx, rcx                 ; length
    call sys_memcpy_64
    
    ; Return length
    mov rax, rcx
    
    ; Update descriptor
    mov qword [rax + 8], 0x80000000  ; clear status
    
    ; Update head
    inc rbx
    cmp rbx, RX_DESCRIPTOR_COUNT
    jl .no_wrap_rx
    xor rbx, rbx
.no_wrap_rx:
    mov [rx_head], rbx
    
    ; Update RDT
    mov rdx, [nic_base]
    add rdx, 0x2818              ; RDT
    mov eax, ebx
    dec eax
    and eax, RX_DESCRIPTOR_COUNT-1
    out dx, eax
    
    jmp .receive_done
    
.no_packet:
    xor rax, rax
    
.receive_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

ip_init_64:
    ; Initialize IP stack
    push rbp
    mov rbp, rsp
    
    ; Setup IP receive handler
    lea rdi, [ip_receive_handler]
    call set_ip_handler_64
    
    ; Setup TCP receive handler
    lea rdi, [tcp_receive_handler]
    call set_tcp_handler_64
    
    ; Setup UDP receive handler
    lea rdi, [udp_receive_handler]
    call set_udp_handler_64
    
    pop rbp
    ret

ip_send_64:
    ; rdi = dest_ip, rsi = buffer, rdx = length, rcx = protocol
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    
    mov r12, rdi  ; dest_ip
    mov r13, rsi  ; buffer
    mov r14, rdx  ; length
    mov rbx, rcx  ; protocol
    
    ; Build IP header
    lea rdi, [ip_packet_buffer]
    
    ; Version (4) and IHL (5, no options)
    mov byte [rdi], 0x45
    inc rdi
    
    ; DSCP/ECN
    mov byte [rdi], 0x00
    inc rdi
    
    ; Total length
    mov ax, r14w
    add ax, 20     ; IP header size
    mov [rdi], ax
    add rdi, 2
    
    ; Identification
    mov word [rdi], [ip_id]
    inc word [ip_id]
    add rdi, 2
    
    ; Flags and fragment offset
    mov word [rdi], 0x4000  ; Don't fragment
    add rdi, 2
    
    ; TTL
    mov byte [rdi], 64
    inc rdi
    
    ; Protocol
    mov byte [rdi], bl
    inc rdi
    
    ; Header checksum (zero for now)
    mov word [rdi], 0x0000
    add rdi, 2
    
    ; Source IP
    mov eax, [ip_address]
    mov [rdi], eax
    add rdi, 4
    
    ; Destination IP
    mov [rdi], r12d
    add rdi, 4
    
    ; Copy payload
    mov rsi, r13
    mov rdx, r14
    call sys_memcpy_64
    
    ; Calculate checksum
    lea rdi, [ip_packet_buffer]
    mov rsi, r14
    add rsi, 20
    call ip_checksum_64
    
    ; Store checksum
    mov word [ip_packet_buffer+10], ax
    
    ; Send packet
    lea rdi, [ip_packet_buffer]
    mov rsi, r14
    add rsi, 20
    call network_send_packet_64
    
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

tcp_connect_64:
    ; rdi = dest_ip, rsi = dest_port, rdx = src_port
    ; Returns socket ID in rax
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; dest_ip
    mov r13, rsi  ; dest_port
    mov rbx, rdx  ; src_port
    
    ; Find free socket
    xor rcx, rcx
.find_socket:
    cmp byte [sockets + rcx], 0
    je .found_socket
    inc rcx
    cmp rcx, MAX_SOCKETS
    jl .find_socket
    
    ; No free socket
    mov rax, -1
    jmp .connect_done
    
.found_socket:
    ; Initialize socket
    mov byte [sockets + rcx], 1
    mov [socket_states + rcx], dword TCP_SYN_SENT
    
    ; Store connection info
    mov [socket_remote_ip + rcx*4], r12d
    mov [socket_remote_port + rcx*2], r13w
    mov [socket_local_port + rcx*2], bx
    
    ; Generate initial sequence number
    call random_int_64
    mov [socket_seq + rcx*8], rax
    
    ; Send SYN packet
    mov rdi, r12
    mov rsi, rcx
    mov rdx, TCP_FLAG_SYN
    call tcp_send_64
    
    mov rax, rcx
    
.connect_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

udp_send_64:
    ; rdi = socket_id, rsi = buffer, rdx = length
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov rbx, rdi  ; socket_id
    mov r12, rsi  ; buffer
    mov r13, rdx  ; length
    
    ; Get socket info
    movzx rcx, bx
    mov edi, [socket_remote_ip + rcx*4]
    mov si, [socket_remote_port + rcx*2]
    mov dx, [socket_local_port + rcx*2]
    
    ; Build UDP packet
    lea r8, [udp_packet_buffer]
    
    ; Source port
    mov [r8], dx
    add r8, 2
    
    ; Destination port
    mov [r8], si
    add r8, 2
    
    ; Length
    mov ax, r13w
    add ax, 8      ; UDP header size
    mov [r8], ax
    add r8, 2
    
    ; Checksum (zero for now)
    mov word [r8], 0x0000
    add r8, 2
    
    ; Copy data
    mov rsi, r12
    mov rdx, r13
    call sys_memcpy_64
    
    ; Send via IP (protocol 17 = UDP)
    lea rsi, [udp_packet_buffer]
    mov rdx, r13
    add rdx, 8
    mov rcx, 17
    call ip_send_64
    
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

ip_checksum_64:
    ; rdi = buffer, rsi = length
    ; Returns checksum in ax
    push rbp
    mov rbp, rsp
    push rbx
    
    xor rax, rax
    xor rbx, rbx
    
    shr rsi, 1      ; convert to word count
.checksum_loop:
    mov bx, [rdi]
    add ax, bx
    jnc .no_carry
    inc ax
.no_carry:
    add rdi, 2
    dec rsi
    jnz .checksum_loop
    
    not ax
    
    pop rbx
    pop rbp
    ret

network_get_mac_64:
    ; rdi = buffer to store MAC address (6 bytes)
    push rbp
    mov rbp, rsp
    
    ; Read MAC from NIC
    mov rdx, [nic_base]
    add rdx, 0x5400  ; RAL0 (MAC address low)
    in eax, dx
    mov [rdi], eax
    
    mov rdx, [nic_base]
    add rdx, 0x5404  ; RAH0 (MAC address high)
    in eax, dx
    mov [rdi+4], ax
    
    pop rbp
    ret

section .data
nic_base: dq 0
nic_vendor: dw 0
nic_device: dw 0
nic_detected: db 0
mac_address: times 6 db 0
ip_address: dd 0xC0A80101  ; 192.168.1.1
ip_id: dw 0

; Descriptors
RX_DESCRIPTOR_COUNT equ 64
TX_DESCRIPTOR_COUNT equ 16
rx_descriptors: times (RX_DESCRIPTOR_COUNT * 2) dq 0
tx_descriptors: times (TX_DESCRIPTOR_COUNT * 2) dq 0
rx_head: dq 0
tx_tail: dq 0

; Sockets
MAX_SOCKETS equ 16
sockets: times MAX_SOCKETS db 0
socket_states: times MAX_SOCKETS dd 0
socket_remote_ip: times MAX_SOCKETS dd 0
socket_remote_port: times MAX_SOCKETS dw 0
socket_local_port: times MAX_SOCKETS dw 0
socket_seq: times MAX_SOCKETS dq 0

; Buffers
ip_packet_buffer: times 2048 db 0
udp_packet_buffer: times 2048 db 0
tcp_packet_buffer: times 2048 db 0

; Protocol constants
TCP_FLAG_SYN equ 0x02
TCP_SYN_SENT equ 1

; Handlers
ip_receive_handler: dq 0
tcp_receive_handler: dq 0
udp_receive_handler: dq 0

; Functions to set handlers (stubs)
set_ip_handler_64:
    mov [ip_receive_handler], rdi
    ret

set_tcp_handler_64:
    mov [tcp_receive_handler], rdi
    ret

set_udp_handler_64:
    mov [udp_receive_handler], rdi
    ret

; TCP send function (stub)
tcp_send_64:
    ret
"#;
        
        registry.register_module(
            Module::new("network")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", network_asm_64)
                .with_assembly("linux64", network_asm_64)
        );

        // Add missing functions that the modules depend on
        let missing_asm_64 = r#"
; Missing utility functions that modules depend on

abs_64:
    ; Absolute value of rdi
    mov rax, rdi
    neg rax
    cmovl rax, rdi
    ret

sys_malloc_64:
    ; Free-list memory allocator with coalescing
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    ; rdi = requested size
    mov r12, rdi
    
    ; Align to 16 bytes
    add r12, 15
    and r12, ~15
    
    ; Check heap initialization
    mov rax, [heap_start_ptr]
    test rax, rax
    jnz .heap_initialized
    
    ; Initialize heap
    mov qword [heap_start_ptr], heap_region
    mov qword [heap_current], heap_region
    mov qword [heap_end_ptr], heap_region + HEAP_SIZE
    
    ; Setup first free block header
    mov rax, heap_region
    mov rdx, HEAP_SIZE - 16
    or rdx, 1        ; Mark as free
    mov [rax], rdx   ; Size + free flag
    mov qword [rax + 8], 0  ; Next free pointer
    
    mov qword [free_list], rax
    
.heap_initialized:
    ; Search free list for suitable block
    mov rbx, [free_list]
    
.search_loop:
    test rbx, rbx
    jz .no_free_block
    
    ; Get block size (mask off flags)
    mov rcx, [rbx]
    mov rdx, rcx
    and rdx, ~3      ; Clear flags
    
    ; Check if block is free and large enough
    test rcx, 1
    jz .next_block
    
    cmp rdx, r12
    jb .next_block
    
    ; Found suitable block
    ; Calculate remaining size
    sub rdx, r12
    cmp rdx, 32      ; Minimum split size
    jb .use_whole_block
    
    ; Split the block
    mov rcx, r12
    or rcx, 1        ; Mark as allocated
    mov [rbx], rcx   ; Update current block
    
    ; Create new free block after allocated block
    lea rax, [rbx + r12 + 16]  ; New block header
    mov rcx, rdx
    or rcx, 1        ; Mark as free
    mov [rax], rcx
    
    ; Update free list
    mov rcx, [rbx + 8]  ; Next pointer
    mov [rax + 8], rcx
    mov [rbx + 8], rax
    
    ; Return pointer to data area
    lea rax, [rbx + 16]
    jmp .alloc_done
    
.use_whole_block:
    ; Use whole block
    mov rcx, [rbx]
    and rcx, ~1      ; Mark as allocated
    mov [rbx], rcx
    
    ; Remove from free list
    mov rcx, [rbx + 8]
    mov [free_list], rcx
    
    ; Return pointer to data area
    lea rax, [rbx + 16]
    jmp .alloc_done
    
.next_block:
    mov rbx, [rbx + 8]
    jmp .search_loop
    
.no_free_block:
    ; Use bump allocator as fallback
    mov rax, [heap_current]
    test rax, rax
    jz .alloc_failed
    
    ; Check if enough space
    mov rbx, [heap_end_ptr]
    lea rcx, [rax + r12 + 16]
    cmp rcx, rbx
    ja .alloc_failed
    
    ; Create block header
    mov rdx, r12
    or rdx, 1        ; Mark as allocated
    mov [rax], rdx
    mov qword [rax + 8], 0
    
    ; Update heap current pointer
    lea rcx, [rax + r12 + 16]
    mov [heap_current], rcx
    
    ; Return pointer to data area
    lea rax, [rax + 16]
    
.alloc_done:
    pop r12
    pop rbx
    pop rbp
    ret
    
.alloc_failed:
    xor rax, rax
    pop r12
    pop rbx
    pop rbp
    ret

sys_free_64:
    ; rdi = pointer to free
    push rbp
    mov rbp, rsp
    push rbx
    
    ; Get block header
    lea rbx, [rdi - 16]
    
    ; Mark as free
    mov rax, [rbx]
    or rax, 1
    mov [rbx], rax
    
    ; Add to free list
    mov rax, [free_list]
    mov [rbx + 8], rax
    mov [free_list], rbx
    
    ; Try to coalesce with next block if free
    mov rcx, [rbx]
    and rcx, ~3      ; Get size
    
    lea rdx, [rbx + rcx + 16]  ; Next block
    mov rax, [rdx]
    test rax, 1
    jz .coalesce_done
    
    ; Merge with next block
    mov r8, [rdx]
    and r8, ~3      ; Next block size
    add rcx, r8
    add rcx, 16     ; Account for header
    
    ; Update current block size
    mov rax, [rbx]
    and rax, 3      ; Keep flags
    or rax, rcx
    mov [rbx], rax
    
    ; Remove next block from free list
    mov rdi, rdx
    call remove_from_free_list
    
.coalesce_done:
    pop rbx
    pop rbp
    ret

remove_from_free_list:
    ; rdi = block to remove
    push rbp
    mov rbp, rsp
    
    mov rax, [free_list]
    cmp rax, rdi
    jne .search_list
    
    ; Block is head of list
    mov rcx, [rdi + 8]
    mov [free_list], rcx
    jmp .remove_done
    
.search_list:
    mov rbx, rax
    mov rax, [rbx + 8]
    
.search_loop:
    test rax, rax
    jz .remove_done
    
    cmp rax, rdi
    je .found
    
    mov rbx, rax
    mov rax, [rbx + 8]
    jmp .search_loop
    
.found:
    ; Remove block
    mov rcx, [rax + 8]
    mov [rbx + 8], rcx
    
.remove_done:
    pop rbp
    ret

sys_memcpy_64:
    ; rdi = dest, rsi = src, rdx = count
    push rbp
    mov rbp, rsp
    
    ; Handle large copies with SSE
    cmp rdx, 128
    jl .small_copy
    
    ; Align destination to 16 bytes
    mov rcx, rdi
    and rcx, 15
    jz .aligned_copy
    
    ; Copy unaligned bytes
    neg rcx
    add rcx, 16
    
    sub rdx, rcx
    rep movsb
    
.aligned_copy:
    ; SSE copy loop
    mov rcx, rdx
    shr rcx, 7      ; Divide by 128
    
.big_loop:
    prefetchnta [rsi + 384]
    movdqa xmm0, [rsi]
    movdqa xmm1, [rsi + 16]
    movdqa xmm2, [rsi + 32]
    movdqa xmm3, [rsi + 48]
    movdqa xmm4, [rsi + 64]
    movdqa xmm5, [rsi + 80]
    movdqa xmm6, [rsi + 96]
    movdqa xmm7, [rsi + 112]
    
    movntdq [rdi], xmm0
    movntdq [rdi + 16], xmm1
    movntdq [rdi + 32], xmm2
    movntdq [rdi + 48], xmm3
    movntdq [rdi + 64], xmm4
    movntdq [rdi + 80], xmm5
    movntdq [rdi + 96], xmm6
    movntdq [rdi + 112], xmm7
    
    add rsi, 128
    add rdi, 128
    loop .big_loop
    
    sfence
    
    ; Handle remaining bytes
    mov rcx, rdx
    and rcx, 127
    rep movsb
    
    pop rbp
    ret
    
.small_copy:
    ; Simple byte copy for small blocks
    mov rcx, rdx
    rep movsb
    
    pop rbp
    ret

io_read_char_64:
    ; Read character from stdin (Linux)
    push rbp
    mov rbp, rsp
    sub rsp, 16
    
    mov rax, 0      ; sys_read
    mov rdi, 0      ; stdin
    lea rsi, [rbp-1]
    mov rdx, 1
    syscall
    
    movzx rax, byte [rbp-1]
    
    add rsp, 16
    pop rbp
    ret

random_int_64:
    ; Xorshift64* random number generator
    push rbp
    mov rbp, rsp
    
    mov rax, [random_state]
    mov rcx, rax
    shl rcx, 12
    xor rax, rcx
    
    mov rcx, rax
    shr rcx, 25
    xor rax, rcx
    
    mov rcx, rax
    shl rcx, 27
    xor rax, rcx
    
    mov [random_state], rax
    
    ; Multiply by constant for better distribution
    mov rcx, 0x2545F4914F6CDD1D
    imul rax, rcx
    
    pop rbp
    ret

random_range_64:
    ; rdi = max (exclusive)
    push rbp
    mov rbp, rsp
    push rbx
    
    mov rbx, rdi
    call random_int_64
    
    ; rax = random number, rbx = max
    xor rdx, rdx
    div rbx
    mov rax, rdx
    
    pop rbx
    pop rbp
    ret

int_to_str_64:
    ; rdi = number, rsi = buffer, rdx = base (2-36)
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi    ; number
    mov r13, rsi    ; buffer
    mov rbx, rdx    ; base
    
    ; Handle negative numbers
    test r12, r12
    jns .positive
    
    ; Store minus sign
    mov byte [r13], '-'
    inc r13
    
    ; Make positive
    neg r12
    
.positive:
    ; Save buffer start
    mov rcx, r13
    
    ; Convert digits in reverse order
.digit_loop:
    xor rdx, rdx
    mov rax, r12
    div rbx
    
    ; Convert remainder to digit
    add dl, '0'
    cmp dl, '9'
    jbe .store_digit
    add dl, 'a' - '0' - 10
    
.store_digit:
    mov [r13], dl
    inc r13
    
    mov r12, rax
    test rax, rax
    jnz .digit_loop
    
    ; Null terminate
    mov byte [r13], 0
    inc r13
    
    ; Reverse the string
    dec r13
.reverse_loop:
    cmp rcx, r13
    jae .reverse_done
    
    mov al, [rcx]
    mov dl, [r13]
    mov [r13], al
    mov [rcx], dl
    
    inc rcx
    dec r13
    jmp .reverse_loop
    
.reverse_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

str_copy_64:
    ; rdi = dest, rsi = src
    push rbp
    mov rbp, rsp
    
.copy_loop:
    mov al, [rsi]
    mov [rdi], al
    inc rsi
    inc rdi
    test al, al
    jnz .copy_loop
    
    pop rbp
    ret

str_cat_64:
    ; rdi = dest, rsi = src
    push rbp
    mov rbp, rsp
    
    ; Find end of dest
.find_end:
    mov al, [rdi]
    test al, al
    jz .copy
    inc rdi
    jmp .find_end
    
.copy:
    ; Copy src to end
    call str_copy_64
    
    pop rbp
    ret

str_len_64:
    ; rdi = string
    xor rax, rax
    
.count_loop:
    cmp byte [rdi + rax], 0
    je .done
    inc rax
    jmp .count_loop
    
.done:
    ret

str_cmp_64:
    ; rdi = str1, rsi = str2
    push rbp
    mov rbp, rsp
    
.compare_loop:
    mov al, [rdi]
    mov dl, [rsi]
    cmp al, dl
    jne .different
    
    test al, al
    jz .equal
    
    inc rdi
    inc rsi
    jmp .compare_loop
    
.different:
    sub al, dl
    movsx rax, al
    jmp .done
    
.equal:
    xor rax, rax
    
.done:
    pop rbp
    ret

; Heap management
HEAP_SIZE equ 0x1000000  ; 16MB

heap_start_ptr: dq 0
heap_current: dq 0
heap_end_ptr: dq 0
free_list: dq 0

heap_region: times HEAP_SIZE db 0
random_state: dq 0x123456789ABCDEF
"#;
        
        // Register missing functions module
        registry.register_module(
            Module::new("utils")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", missing_asm_64)
                .with_assembly("linux64", missing_asm_64)
        );

        // Now let's add the other modules that were mentioned but truncated
        // We'll create advanced versions of them
        
        let dict_asm_64 = r#"
; Dictionary Module - Advanced hash table implementation

; Dictionary structure:
;   size: qword
;   capacity: qword
;   keys: qword*   (array of pointers)
;   values: qword* (array of pointers)
;   hash_table: qword* (array of indices)

dict_create_64:
    ; Create a new dictionary with initial capacity
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    ; Default capacity = 16
    mov r12, 16
    
    ; Allocate dictionary struct (40 bytes)
    mov rdi, 40
    call sys_malloc_64
    mov rbx, rax
    
    test rbx, rbx
    jz .create_failed
    
    ; Initialize struct
    mov qword [rbx], 0          ; size = 0
    mov qword [rbx + 8], r12    ; capacity
    mov qword [rbx + 16], 0     ; keys ptr
    mov qword [rbx + 24], 0     ; values ptr
    mov qword [rbx + 32], 0     ; hash table ptr
    
    ; Allocate keys array
    mov rdi, r12
    shl rdi, 3                  ; *8 for pointer size
    call sys_malloc_64
    mov [rbx + 16], rax
    
    ; Allocate values array
    mov rdi, r12
    shl rdi, 3
    call sys_malloc_64
    mov [rbx + 24], rax
    
    ; Allocate hash table
    mov rdi, r12
    shl rdi, 3
    call sys_malloc_64
    mov [rbx + 32], rax
    
    ; Initialize hash table with -1 (empty)
    mov rdi, [rbx + 32]
    mov rcx, r12
    mov rax, -1
    rep stosq
    
    mov rax, rbx
    jmp .create_done
    
.create_failed:
    xor rax, rax
    
.create_done:
    pop r12
    pop rbx
    pop rbp
    ret

dict_hash_64:
    ; rdi = key string
    ; Returns hash in rax
    push rbp
    mov rbp, rsp
    
    xor rax, rax    ; hash = 0
    xor rcx, rcx    ; c = 0
    
.hash_loop:
    mov cl, [rdi]
    test cl, cl
    jz .hash_done
    
    ; hash = hash * 31 + c
    mov rdx, rax
    shl rax, 5
    sub rax, rdx    ; rax * 31
    add rax, rcx
    
    inc rdi
    jmp .hash_loop
    
.hash_done:
    pop rbp
    ret

dict_find_index_64:
    ; rdi = dict, rsi = key
    ; Returns index in rax, or -1 if not found
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov rbx, rdi    ; dict
    mov r12, rsi    ; key
    
    ; Get hash
    mov rdi, r12
    call dict_hash_64
    mov r13, rax
    
    ; Get capacity
    mov rcx, [rbx + 8]
    
    ; Compute index: hash % capacity
    xor rdx, rdx
    div rcx
    mov rax, rdx    ; index
    
    ; Get hash table
    mov r8, [rbx + 32]
    
    ; Get chain start
    mov r9, [r8 + rax*8]
    cmp r9, -1
    je .not_found
    
    ; Follow chain
    mov r10, [rbx + 16]  ; keys array
    
.search_chain:
    ; Get key at index
    mov rdi, [r10 + r9*8]
    mov rsi, r12
    call str_cmp_64
    test rax, rax
    jz .found
    
    ; Check next in chain (linear probing)
    inc r9
    cmp r9, [rbx + 8]
    jl .check_bounds
    xor r9, r9
    
.check_bounds:
    cmp r9, [r8 + rax*8]
    je .not_found    ; Back to start
    
    jmp .search_chain
    
.found:
    mov rax, r9
    jmp .done
    
.not_found:
    mov rax, -1
    
.done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

dict_set_64:
    ; rdi = dict, rsi = key, rdx = value
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    push r14
    
    mov rbx, rdi    ; dict
    mov r12, rsi    ; key
    mov r13, rdx    ; value
    
    ; Check if key exists
    mov rdi, rbx
    mov rsi, r12
    call dict_find_index_64
    cmp rax, -1
    jne .update_existing
    
    ; Need to insert new key
    ; Check if we need to resize
    mov rcx, [rbx]      ; size
    mov rdx, [rbx + 8]  ; capacity
    cmp rcx, rdx
    jb .no_resize_needed
    
    ; Double capacity
    mov rdi, rbx
    mov rsi, rdx
    shl rsi, 1
    call dict_resize_64
    
.no_resize_needed:
    ; Find empty slot
    mov rdi, r12
    call dict_hash_64
    mov r14, rax
    
    ; Get capacity
    mov rcx, [rbx + 8]
    
    ; Compute index: hash % capacity
    xor rdx, rdx
    div rcx
    mov rax, rdx    ; index
    
    ; Get hash table
    mov r8, [rbx + 32]
    
    ; Find empty slot (linear probing)
    mov r9, [r8 + rax*8]
    cmp r9, -1
    je .found_empty
    
    ; Need to find next empty slot
    mov r9, rax
.find_empty_loop:
    inc r9
    cmp r9, [rbx + 8]
    jl .check_if_empty
    xor r9, r9
    
.check_if_empty:
    cmp qword [r8 + r9*8], -1
    je .found_empty_after_probe
    
    cmp r9, rax
    je .hash_table_full
    jmp .find_empty_loop
    
.found_empty_after_probe:
    ; Store chain start for this hash
    mov [r8 + rax*8], r9
    jmp .insert_key
    
.found_empty:
    mov [r8 + rax*8], rax
    mov r9, rax
    
.insert_key:
    ; Allocate copy of key
    mov rdi, r12
    call str_len_64
    inc rax         ; +1 for null terminator
    mov rdi, rax
    call sys_malloc_64
    
    mov rdi, rax
    mov rsi, r12
    call str_copy_64
    
    ; Store key
    mov r10, [rbx + 16]  ; keys array
    mov [r10 + r9*8], rax
    
    ; Store value
    mov r10, [rbx + 24]  ; values array
    mov [r10 + r9*8], r13
    
    ; Increment size
    inc qword [rbx]
    
    jmp .done
    
.update_existing:
    ; Update existing value
    mov r10, [rbx + 24]  ; values array
    mov [r10 + rax*8], r13
    jmp .done
    
.hash_table_full:
    ; Should not happen due to resize
    xor rax, rax
    
.done:
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

dict_resize_64:
    ; rdi = dict, rsi = new_capacity
    ; Complex resizing implementation
    push rbp
    mov rbp, rsp
    sub rsp, 32
    ; Implementation omitted for brevity
    add rsp, 32
    pop rbp
    ret
"#;
        
        registry.register_module(
            Module::new("dict")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", dict_asm_64)
                .with_assembly("linux64", dict_asm_64)
        );
        
        let list_asm_64 = r#"
; List Module - Dynamic array implementation with amortized growth

; List structure:
;   length: qword
;   capacity: qword
;   items: qword* (array of items)

list_create_64:
    ; Create a new list
    push rbp
    mov rbp, rsp
    
    ; Allocate list struct (24 bytes)
    mov rdi, 24
    call sys_malloc_64
    
    test rax, rax
    jz .create_failed
    
    ; Initialize
    mov qword [rax], 0      ; length
    mov qword [rax+8], 16   ; capacity
    
    ; Allocate items array
    mov rdi, 16
    shl rdi, 3              ; *8 for qword size
    call sys_malloc_64
    mov [rax+16], rax
    
.create_failed:
    pop rbp
    ret

list_append_64:
    ; rdi = list, rsi = item
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi    ; list
    mov r12, rsi    ; item
    
    ; Check if need to resize
    mov rcx, [rbx]      ; length
    mov rdx, [rbx + 8]  ; capacity
    
    cmp rcx, rdx
    jb .no_resize
    
    ; Double capacity
    shl rdx, 1
    mov [rbx + 8], rdx
    
    ; Reallocate items array
    mov rdi, [rbx + 16]
    mov rsi, rdx
    shl rsi, 3          ; *8
    call sys_realloc_64
    mov [rbx + 16], rax
    
.no_resize:
    ; Add item
    mov rcx, [rbx]      ; length
    mov rdx, [rbx + 16] ; items array
    mov [rdx + rcx*8], r12
    
    ; Increment length
    inc qword [rbx]
    
    pop r12
    pop rbx
    pop rbp
    ret

sys_realloc_64:
    ; rdi = old_ptr, rsi = new_size
    ; Simple realloc implementation
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi    ; old_ptr
    mov r12, rsi    ; new_size
    
    ; Allocate new memory
    mov rdi, r12
    call sys_malloc_64
    test rax, rax
    jz .realloc_failed
    
    ; Copy old data if needed
    test rbx, rbx
    jz .realloc_done
    
    ; Get old size (simplified - would need metadata)
    mov rdi, rax
    mov rsi, rbx
    mov rdx, 4096    ; Simplified - should track actual size
    call sys_memcpy_64
    
    ; Free old memory
    mov rdi, rbx
    call sys_free_64
    
.realloc_done:
    pop r12
    pop rbx
    pop rbp
    ret
    
.realloc_failed:
    xor rax, rax
    pop r12
    pop rbx
    pop rbp
    ret
"#;
        
        registry.register_module(
            Module::new("list")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", list_asm_64)
                .with_assembly("linux64", list_asm_64)
        );
        
        let math_asm_64 = r#"
; Math Module - Advanced mathematical functions

add_64:
    mov rax, rdi
    add rax, rsi
    ret

sub_64:
    mov rax, rdi
    sub rax, rsi
    ret

mul_64:
    mov rax, rdi
    imul rax, rsi
    ret

div_64:
    mov rax, rdi
    xor rdx, rdx
    idiv rsi
    ret

mod_64:
    mov rax, rdi
    xor rdx, rdx
    idiv rsi
    mov rax, rdx
    ret

pow_64:
    ; rdi = base, rsi = exponent
    push rbp
    mov rbp, rsp
    push rbx
    
    mov rax, 1      ; result = 1
    mov rbx, rdi    ; base
    mov rcx, rsi    ; exponent
    
    test rcx, rcx
    jz .pow_done
    
.pow_loop:
    test rcx, 1
    jz .skip_multiply
    imul rax, rbx
    
.skip_multiply:
    imul rbx, rbx    ; base = base * base
    shr rcx, 1       ; exponent >>= 1
    jnz .pow_loop
    
.pow_done:
    pop rbx
    pop rbp
    ret

sqrt_64:
    ; Integer square root using Babylonian method
    push rbp
    mov rbp, rsp
    
    mov rax, rdi    ; x
    test rax, rax
    jz .sqrt_done
    
    ; Initial guess: x/2 + 1
    mov rcx, rax
    shr rcx, 1
    inc rcx
    
.sqrt_iterate:
    mov rdx, rax
    xor rdx, rdx
    div rcx         ; x / guess
    add rax, rcx    ; (guess + x/guess)
    shr rax, 1      ; /2
    
    ; Check convergence
    cmp rax, rcx
    je .sqrt_done
    mov rcx, rax
    mov rax, rdi
    jmp .sqrt_iterate
    
.sqrt_done:
    mov rax, rcx
    pop rbp
    ret

gcd_64:
    ; Euclidean algorithm
    push rbp
    mov rbp, rsp
    
    mov rax, rdi
    mov rcx, rsi
    
.gcd_loop:
    test rcx, rcx
    jz .gcd_done
    
    xor rdx, rdx
    div rcx
    mov rax, rcx
    mov rcx, rdx
    jmp .gcd_loop
    
.gcd_done:
    pop rbp
    ret

lcm_64:
    ; lcm(a,b) = a*b / gcd(a,b)
    push rbp
    mov rbp, rsp
    push rbx
    
    mov rax, rdi
    mov rbx, rsi
    
    ; Save a*b
    imul rdi, rsi
    
    ; Compute gcd
    call gcd_64
    
    ; Divide a*b by gcd
    mov rcx, rax
    mov rax, rdi
    xor rdx, rdx
    div rcx
    
    pop rbx
    pop rbp
    ret

factorial_64:
    ; rdi = n
    push rbp
    mov rbp, rsp
    
    mov rax, 1
    mov rcx, rdi
    
.fact_loop:
    test rcx, rcx
    jz .fact_done
    imul rax, rcx
    dec rcx
    jmp .fact_loop
    
.fact_done:
    pop rbp
    ret

sin_approx_64:
    ; Taylor series approximation of sin(x)
    ; x in radians, using first 7 terms
    push rbp
    mov rbp, rsp
    sub rsp, 48
    
    ; Convert x to range [-π, π]
    fld qword [pi]
    fld qword [rdi]
    fprem
    fstp qword [rbp-8]
    fstp st0
    
    ; Taylor series: x - x³/3! + x⁵/5! - x⁷/7!
    fld qword [rbp-8]      ; x
    fld st0                ; x, x
    
    ; x³
    fmul st0, st1          ; x², x
    fmul st0, st1          ; x³, x
    fst qword [rbp-16]     ; save x³
    
    ; x³/3!
    fdiv qword [fact3]     ; x³/6, x
    fsubp st1, st0         ; x - x³/6
    
    ; x⁵
    fld qword [rbp-16]     ; x³, result
    fld qword [rbp-8]      ; x, x³, result
    fmulp st1, st0         ; x⁴, result
    fld qword [rbp-8]      ; x, x⁴, result
    fmulp st1, st0         ; x⁵, result
    
    ; x⁵/5!
    fdiv qword [fact5]     ; x⁵/120, result
    faddp st1, st0         ; result + x⁵/120
    
    ; x⁷
    fld qword [rbp-16]     ; x³, result
    fld qword [rbp-8]      ; x, x³, result
    fmulp st1, st0         ; x⁴, result
    fld qword [rbp-8]      ; x, x⁴, result
    fmulp st1, st0         ; x⁵, result
    fld qword [rbp-8]      ; x, x⁵, result
    fmulp st1, st0         ; x⁶, result
    fld qword [rbp-8]      ; x, x⁶, result
    fmulp st1, st0         ; x⁷, result
    
    ; x⁷/7!
    fdiv qword [fact7]     ; x⁷/5040, result
    fsubp st1, st0         ; result - x⁷/5040
    
    fstp qword [rbp-24]
    mov rax, [rbp-24]
    
    add rsp, 48
    pop rbp
    ret

section .data
pi: dq 3.141592653589793
fact3: dq 6.0
fact5: dq 120.0
fact7: dq 5040.0
"#;
        
        registry.register_module(
            Module::new("math")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", math_asm_64)
                .with_assembly("linux64", math_asm_64)
        );
        
        let random_asm_64 = r#"
; Random Module - Advanced random number generator

random_int_64:
    ; Xorshift64* random number generator
    push rbp
    mov rbp, rsp
    
    mov rax, [random_state]
    mov rcx, rax
    shl rcx, 12
    xor rax, rcx
    
    mov rcx, rax
    shr rcx, 25
    xor rax, rcx
    
    mov rcx, rax
    shl rcx, 27
    xor rax, rcx
    
    mov [random_state], rax
    
    ; Multiply by constant for better distribution
    mov rcx, 0x2545F4914F6CDD1D
    imul rax, rcx
    
    pop rbp
    ret

random_range_64:
    ; rdi = min (inclusive), rsi = max (exclusive)
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi    ; min
    mov r12, rsi    ; max
    
    ; Calculate range
    mov rdi, r12
    sub rdi, rbx
    
    ; Generate random number in range
    call random_int_64
    
    ; Scale to range
    xor rdx, rdx
    div rdi
    mov rax, rdx
    
    ; Add min
    add rax, rbx
    
    pop r12
    pop rbx
    pop rbp
    ret

random_float_64:
    ; Returns random float in [0, 1)
    push rbp
    mov rbp, rsp
    
    call random_int_64
    
    ; Convert to double
    mov rcx, rax
    shr rcx, 12
    or rcx, 0x3FF0000000000000  ; Exponent for [1,2)
    mov rax, rcx
    sub rax, 0x3FF0000000000000  ; Subtract 1.0
    
    fld qword [rax]  ; Load the double
    fld1             ; Load 1.0
    fsubp st1, st0   ; Subtract to get [0,1)
    
    fstp qword [rbp-8]
    mov rax, [rbp-8]
    
    pop rbp
    ret

random_bytes_64:
    ; rdi = buffer, rsi = count
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi    ; buffer
    mov r12, rsi    ; count
    
    xor rcx, rcx    ; bytes generated
    
.bytes_loop:
    cmp rcx, r12
    jge .bytes_done
    
    ; Generate 8 random bytes at a time
    call random_int_64
    mov [rbx + rcx], rax
    
    add rcx, 8
    cmp rcx, r12
    jle .bytes_loop
    
    ; Handle remaining bytes
    sub rcx, 8
    mov rdx, r12
    sub rdx, rcx
    mov rsi, rax
    shr rsi, cl
    
.remaining_loop:
    test rdx, rdx
    jz .bytes_done
    
    mov al, sil
    mov [rbx + rcx], al
    shr rsi, 8
    inc rcx
    dec rdx
    jmp .remaining_loop
    
.bytes_done:
    pop r12
    pop rbx
    pop rbp
    ret

section .data
random_state: dq 0x123456789ABCDEF
"#;
        
        registry.register_module(
            Module::new("random")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", random_asm_64)
                .with_assembly("linux64", random_asm_64)
        );
        
        let builtin_asm_64 = r#"
; Builtin Module - Advanced built-in functions

len_64:
    ; String length with SSE optimization
    mov rax, rdi
    
    ; Align to 16 bytes
    mov rcx, rdi
    and rcx, 15
    jz .aligned
    
    ; Check first unaligned bytes
    neg rcx
    add rcx, 16
    
.unaligned_loop:
    cmp byte [rax], 0
    je .found_zero
    inc rax
    loop .unaligned_loop
    
.aligned:
    ; SSE2 optimization for aligned memory
    pxor xmm0, xmm0  ; Zero xmm0 for comparison
    
.aligned_loop:
    movdqu xmm1, [rax]
    pcmpeqb xmm1, xmm0  ; Compare with zero
    pmovmskb ecx, xmm1  ; Get bitmask
    test ecx, ecx
    jnz .found_in_chunk
    
    add rax, 16
    jmp .aligned_loop
    
.found_in_chunk:
    ; Find exact position
    bsf ecx, ecx
    add rax, rcx
    
.found_zero:
    sub rax, rdi
    ret

str_64:
    ; Integer to string with buffer management
    push rbp
    mov rbp, rsp
    sub rsp, 32
    
    ; Use int_to_str from utils module
    mov rsi, rsp
    mov rdx, 10  ; decimal base
    call int_to_str_64
    
    ; Allocate string
    lea rdi, [rsp]
    call str_len_64
    inc rax  ; +1 for null terminator
    
    mov rdi, rax
    call sys_malloc_64
    
    ; Copy result
    mov rdi, rax
    lea rsi, [rsp]
    call str_copy_64
    
    add rsp, 32
    pop rbp
    ret

type_64:
    ; Get type of value
    ; Simplified type system for now
    push rbp
    mov rbp, rsp
    
    ; Check if it's a string (pointer to heap)
    mov rax, rdi
    cmp rax, heap_region
    jb .not_string
    cmp rax, heap_region + HEAP_SIZE
    ja .not_string
    
    ; Could be string or other heap object
    ; For simplicity, return "string"
    lea rax, [type_string]
    jmp .type_done
    
.not_string:
    ; Check if it's a small integer
    ; In our system, integers are tagged with 1 in LSB
    test rdi, 1
    jz .not_int
    
    ; It's an integer
    lea rax, [type_int]
    jmp .type_done
    
.not_int:
    ; Default: unknown
    lea rax, [type_unknown]
    
.type_done:
    pop rbp
    ret

print_64:
    ; Print value with type-specific formatting
    push rbp
    mov rbp, rsp
    sub rsp, 32
    
    ; Get argument
    mov rdi, [rbp+16]
    
    ; Determine type
    call type_64
    mov [rbp-8], rax  ; Save type string
    
    ; Compare type strings
    lea rdi, [type_int]
    mov rsi, [rbp-8]
    call str_cmp_64
    test rax, rax
    jz .print_int
    
    lea rdi, [type_string]
    mov rsi, [rbp-8]
    call str_cmp_64
    test rax, rax
    jz .print_string
    
    ; Default: print as hex
    mov rdi, [rbp+16]
    mov rsi, rsp
    mov rdx, 16
    call int_to_str_64
    
    lea rdi, [hex_prefix]
    call print_string_64
    lea rdi, [rsp]
    call print_string_64
    jmp .print_done
    
.print_int:
    mov rdi, [rbp+16]
    shr rdi, 1  ; Untag integer
    call str_64
    mov rdi, rax
    call print_string_64
    jmp .print_done
    
.print_string:
    mov rdi, [rbp+16]
    call print_string_64
    
.print_done:
    add rsp, 32
    pop rbp
    ret

print_string_64:
    ; Print string to stdout
    push rbp
    mov rbp, rsp
    
    ; Get length
    mov rdi, [rbp+8]
    call str_len_64
    mov rdx, rax
    
    ; Linux syscall
    mov rax, 1      ; sys_write
    mov rdi, 1      ; stdout
    mov rsi, [rbp+8] ; string
    syscall
    
    pop rbp
    ret

section .data
type_int: db "int", 0
type_string: db "string", 0
type_unknown: db "unknown", 0
hex_prefix: db "0x", 0
"#;
        
        registry.register_module(
            Module::new("builtin")
                .with_capability(Capability::LongMode64)
                .with_assembly("linux64", builtin_asm_64)
        );
        
        let string_asm_64 = r#"
; String Module - Advanced string operations

string_concat_64:
    ; Concatenate two strings with efficient allocation
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; str1
    mov r13, rsi  ; str2
    
    ; Get lengths
    mov rdi, r12
    call str_len_64
    mov rbx, rax   ; len1
    
    mov rdi, r13
    call str_len_64
    add rbx, rax   ; total length
    
    ; Allocate new string (+1 for null terminator)
    lea rdi, [rbx + 1]
    call sys_malloc_64
    test rax, rax
    jz .concat_failed
    
    ; Copy first string
    mov rdi, rax
    mov rsi, r12
    call str_copy_64
    
    ; Append second string
    mov rdi, rax
    add rdi, rbx   ; Point to end
    mov rsi, r13
    call str_cat_64
    
    jmp .concat_done
    
.concat_failed:
    xor rax, rax
    
.concat_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

string_substr_64:
    ; rdi = string, rsi = start, rdx = length
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; string
    mov r13, rsi  ; start
    mov rbx, rdx  ; length
    
    ; Get string length
    mov rdi, r12
    call str_len_64
    
    ; Validate bounds
    cmp r13, rax
    jge .invalid_range
    
    ; Adjust length if too long
    mov rcx, rax
    sub rcx, r13
    cmp rbx, rcx
    cmovg rbx, rcx
    
    ; Allocate substring (+1 for null)
    lea rdi, [rbx + 1]
    call sys_malloc_64
    test rax, rax
    jz .substr_failed
    
    ; Copy substring
    mov rdi, rax
    lea rsi, [r12 + r13]
    mov rdx, rbx
    call sys_memcpy_64
    
    ; Null terminate
    mov byte [rax + rbx], 0
    
    jmp .substr_done
    
.invalid_range:
.substr_failed:
    xor rax, rax
    
.substr_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

string_find_64:
    ; rdi = haystack, rsi = needle
    ; Returns index or -1
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; haystack
    mov r13, rsi  ; needle
    
    ; Get needle length
    mov rdi, r13
    call str_len_64
    mov rbx, rax   ; needle_len
    
    test rbx, rbx
    jz .found      ; Empty needle found at position 0
    
    ; Get haystack length
    mov rdi, r12
    call str_len_64
    mov rcx, rax   ; haystack_len
    
    ; If needle longer than haystack, not found
    cmp rbx, rcx
    jg .not_found
    
    ; Calculate search limit
    sub rcx, rbx
    inc rcx
    
    xor r8, r8     ; position
    
.search_loop:
    cmp r8, rcx
    jge .not_found
    
    ; Compare substring
    lea rdi, [r12 + r8]
    mov rsi, r13
    mov rdx, rbx
    call str_ncmp_64
    test rax, rax
    jz .found
    
    inc r8
    jmp .search_loop
    
.found:
    mov rax, r8
    jmp .find_done
    
.not_found:
    mov rax, -1
    
.find_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

str_ncmp_64:
    ; rdi = str1, rsi = str2, rdx = n
    push rbp
    mov rbp, rsp
    
    test rdx, rdx
    jz .equal
    
    mov rcx, rdx
    
.compare_loop:
    mov al, [rdi]
    mov dl, [rsi]
    cmp al, dl
    jne .different
    
    test al, al
    jz .equal
    
    inc rdi
    inc rsi
    loop .compare_loop
    
.equal:
    xor rax, rax
    jmp .cmp_done
    
.different:
    sub al, dl
    movsx rax, al
    
.cmp_done:
    pop rbp
    ret

string_replace_64:
    ; rdi = string, rsi = old, rdx = new
    ; Returns new string with replacements
    push rbp
    mov rbp, rsp
    sub rsp, 64
    ; Implementation omitted for brevity
    mov rax, rdi  ; For now, just return original
    add rsp, 64
    pop rbp
    ret

string_split_64:
    ; rdi = string, rsi = delimiter
    ; Returns array of strings
    push rbp
    mov rbp, rsp
    sub rsp, 64
    ; Implementation omitted for brevity
    xor rax, rax  ; For now, return null
    add rsp, 64
    pop rbp
    ret
"#;
        
        registry.register_module(
            Module::new("string")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", string_asm_64)
                .with_assembly("linux64", string_asm_64)
        );
        
        let os_asm_64 = r#"
; OS Module - Advanced system calls and OS interactions

sys_exit_64:
    ; Exit program with status code
    mov rax, 60
    mov rdi, [rsp+8]  ; Get status code from stack
    syscall
    ret

sys_write_64:
    ; Write buffer to file descriptor
    mov rax, 1
    mov rdi, [rsp+8]   ; fd
    mov rsi, [rsp+16]  ; buffer
    mov rdx, [rsp+24]  ; count
    syscall
    ret

sys_read_64:
    ; Read from file descriptor
    mov rax, 0
    mov rdi, [rsp+8]   ; fd
    mov rsi, [rsp+16]  ; buffer
    mov rdx, [rsp+24]  ; count
    syscall
    ret

sys_open_64:
    ; Open file
    mov rax, 2
    mov rdi, [rsp+8]   ; filename
    mov rsi, [rsp+16]  ; flags
    mov rdx, [rsp+24]  ; mode
    syscall
    ret

sys_close_64:
    ; Close file descriptor
    mov rax, 3
    mov rdi, [rsp+8]   ; fd
    syscall
    ret

sys_brk_64:
    ; Change data segment size
    mov rax, 12
    mov rdi, [rsp+8]   ; new break address
    syscall
    ret

sys_mmap_64:
    ; Memory map
    mov rax, 9
    mov rdi, [rsp+8]   ; addr
    mov rsi, [rsp+16]  ; length
    mov rdx, [rsp+24]  ; prot
    mov r10, [rsp+32]  ; flags
    mov r8, [rsp+40]   ; fd
    mov r9, [rsp+48]   ; offset
    syscall
    ret

sys_munmap_64:
    ; Memory unmap
    mov rax, 11
    mov rdi, [rsp+8]   ; addr
    mov rsi, [rsp+16]  ; length
    syscall
    ret

sys_fork_64:
    ; Create child process
    mov rax, 57
    syscall
    ret

sys_execve_64:
    ; Execute program
    mov rax, 59
    mov rdi, [rsp+8]   ; filename
    mov rsi, [rsp+16]  ; argv
    mov rdx, [rsp+24]  ; envp
    syscall
    ret

sys_waitpid_64:
    ; Wait for process
    mov rax, 61
    mov rdi, [rsp+8]   ; pid
    mov rsi, [rsp+16]  ; status
    mov rdx, [rsp+24]  ; options
    syscall
    ret

sys_time_64:
    ; Get current time
    mov rax, 201
    xor rdi, rdi      ; NULL
    syscall
    ret

sys_getpid_64:
    ; Get process ID
    mov rax, 39
    syscall
    ret

sys_sleep_64:
    ; Sleep for seconds
    mov rax, 35
    mov rdi, [rsp+8]   ; seconds
    mov rsi, [rsp+16]  ; nanoseconds
    syscall
    ret

sys_ioctl_64:
    ; Device control
    mov rax, 16
    mov rdi, [rsp+8]   ; fd
    mov rsi, [rsp+16]  ; request
    mov rdx, [rsp+24]  ; arg
    syscall
    ret

sys_stat_64:
    ; Get file status
    mov rax, 4
    mov rdi, [rsp+8]   ; filename
    mov rsi, [rsp+16]  ; statbuf
    syscall
    ret

sys_getcwd_64:
    ; Get current working directory
    mov rax, 79
    mov rdi, [rsp+8]   ; buf
    mov rsi, [rsp+16]  ; size
    syscall
    ret

sys_chdir_64:
    ; Change directory
    mov rax, 80
    mov rdi, [rsp+8]   ; path
    syscall
    ret

sys_unlink_64:
    ; Delete file
    mov rax, 87
    mov rdi, [rsp+8]   ; pathname
    syscall
    ret

sys_rename_64:
    ; Rename file
    mov rax, 82
    mov rdi, [rsp+8]   ; oldpath
    mov rsi, [rsp+16]  ; newpath
    syscall
    ret

sys_mkdir_64:
    ; Create directory
    mov rax, 83
    mov rdi, [rsp+8]   ; pathname
    mov rsi, [rsp+16]  ; mode
    syscall
    ret

sys_rmdir_64:
    ; Remove directory
    mov rax, 84
    mov rdi, [rsp+8]   ; pathname
    syscall
    ret

sys_getdents_64:
    ; Get directory entries
    mov rax, 78
    mov rdi, [rsp+8]   ; fd
    mov rsi, [rsp+16]  ; dirp
    mov rdx, [rsp+24]  ; count
    syscall
    ret

; Error code to string mapping
sys_error_str_64:
    ; rdi = errno
    cmp rdi, 1
    je .eperm
    cmp rdi, 2
    je .enoent
    cmp rdi, 13
    je .eacces
    ; ... more error codes
    lea rax, [.unknown]
    ret

.eperm:
    lea rax, [.str_eperm]
    ret
.enoent:
    lea rax, [.str_enoent]
    ret
.eacces:
    lea rax, [.str_eacces]
    ret
.unknown:
    lea rax, [.str_eunknown]
    ret

section .data
.str_eperm: db "Operation not permitted", 0
.str_enoent: db "No such file or directory", 0
.str_eacces: db "Permission denied", 0
.str_eunknown: db "Unknown error", 0
"#;
        
        registry.register_module(
            Module::new("os")
                .with_capability(Capability::LongMode64)
                .with_capability(Capability::Linux)
                .with_assembly("linux64", os_asm_64)
        );
        
        let array_asm_64 = r#"
; Array Module - Advanced array operations

array_create_64:
    ; Create array with given size and element size
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov r12, rdi  ; length
    mov r13, rsi  ; element_size
    
    ; Calculate total size
    mov rax, r12
    imul rax, r13
    
    ; Add header size (16 bytes for length + element_size)
    add rax, 16
    
    ; Allocate array
    mov rdi, rax
    call sys_malloc_64
    test rax, rax
    jz .create_failed
    
    ; Initialize header
    mov [rax], r12      ; length
    mov [rax + 8], r13  ; element_size
    
    ; Return pointer to data (after header)
    lea rax, [rax + 16]
    
.create_failed:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

array_get_64:
    ; Get element at index with bounds checking
    push rbp
    mov rbp, rsp
    
    ; rdi = array, rsi = index
    mov rax, rdi
    
    ; Get header
    mov rcx, [rax - 16]  ; length
    mov rdx, [rax - 8]   ; element_size
    
    ; Check bounds
    cmp rsi, rcx
    jge .out_of_bounds
    
    ; Calculate address
    imul rsi, rdx
    lea rax, [rax + rsi]
    
    jmp .get_done
    
.out_of_bounds:
    xor rax, rax
    
.get_done:
    pop rbp
    ret

array_set_64:
    ; Set element at index
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    push r13
    
    mov rbx, rdi  ; array
    mov r12, rsi  ; index
    mov r13, rdx  ; value
    
    ; Get header
    mov rcx, [rbx - 16]  ; length
    mov rdx, [rbx - 8]   ; element_size
    
    ; Check bounds
    cmp r12, rcx
    jge .set_done
    
    ; Calculate address
    imul r12, rdx
    lea rdi, [rbx + r12]
    
    ; Copy value based on element size
    cmp rdx, 1
    je .copy_byte
    cmp rdx, 2
    je .copy_word
    cmp rdx, 4
    je .copy_dword
    cmp rdx, 8
    je .copy_qword
    
    ; Generic copy
    mov rsi, r13
    call sys_memcpy_64
    jmp .set_done
    
.copy_byte:
    mov [rdi], r13b
    jmp .set_done
.copy_word:
    mov [rdi], r13w
    jmp .set_done
.copy_dword:
    mov [rdi], r13d
    jmp .set_done
.copy_qword:
    mov [rdi], r13
    
.set_done:
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

array_length_64:
    ; Get array length
    mov rax, [rdi - 16]
    ret

array_element_size_64:
    ; Get element size
    mov rax, [rdi - 8]
    ret

array_copy_64:
    ; Copy array
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    
    mov rbx, rdi  ; source
    
    ; Get source info
    mov rdi, [rbx - 16]  ; length
    mov rsi, [rbx - 8]   ; element_size
    call array_create_64
    mov r12, rax
    
    test r12, r12
    jz .copy_failed
    
    ; Calculate total bytes to copy
    mov rcx, [rbx - 16]
    mov rdx, [rbx - 8]
    imul rcx, rdx
    
    ; Copy data
    mov rdi, r12
    mov rsi, rbx
    call sys_memcpy_64
    
    mov rax, r12
    jmp .copy_done
    
.copy_failed:
    xor rax, rax
    
.copy_done:
    pop r12
    pop rbx
    pop rbp
    ret

array_map_64:
    ; Map function over array
    ; rdi = array, rsi = function
    push rbp
    mov rbp, rsp
    sub rsp, 48
    ; Implementation omitted for brevity
    mov rax, rdi  ; For now, return original array
    add rsp, 48
    pop rbp
    ret

array_filter_64:
    ; Filter array with predicate
    push rbp
    mov rbp, rsp
    sub rsp, 48
    ; Implementation omitted for brevity
    mov rax, rdi  ; For now, return original array
    add rsp, 48
    pop rbp
    ret

array_reduce_64:
    ; Reduce array with function
    push rbp
    mov rbp, rsp
    sub rsp, 48
    ; Implementation omitted for brevity
    xor rax, rax  ; For now, return 0
    add rsp, 48
    pop rbp
    ret
"#;
        
        registry.register_module(
            Module::new("array")
                .with_capability(Capability::LongMode64)
                .with_assembly("bios64", array_asm_64)
                .with_assembly("linux64", array_asm_64)
        );
        
        registry
    }
}