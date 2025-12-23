; ============================================
; Rython BIOS Bootloader
; Generated from Rython AST
; ============================================

    org 0x7C00
    bits 16

start:
    ; Initialize segments and stack
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti
    cld

    ; Clear screen
    mov ax, 0x0003
    int 0x10

    ; Print welcome message
    mov si, msg_welcome
    call print_string

    ; ===== User Program Starts =====
    ; Call print_int(1 args)
    ; Number: 42
    mov ax, 42
    call print_decimal
    mov si, msg_newline
    call print_string
    call print_decimal
    mov si, msg_newline
    call print_string

    ; Halt system
    mov si, msg_halt
    call print_string
halt_loop:
    hlt
    jmp short halt_loop

; ===== BIOS Subroutines =====

print_string:
    push ax
    push si
.ps_loop:
    lodsb
    or al, al
    jz .ps_done
    mov ah, 0x0E
    int 0x10
    jmp .ps_loop
.ps_done:
    pop si
    pop ax
    ret

print_decimal:
    push ax
    push bx
    push cx
    push dx
    push si
    
    xor cx, cx
    mov bx, 10
.pd_divide:
    xor dx, dx
    div bx
    add dl, '0'
    push dx
    inc cx
    test ax, ax
    jnz .pd_divide
    
.pd_print:
    pop ax
    mov ah, 0x0E
    int 0x10
    loop .pd_print
    
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

print_hex_word:
    push ax
    push bx
    push cx
    push dx
    
    mov cx, 4
.phw_digit:
    rol ax, 4
    mov dx, ax
    and dx, 0x000F
    add dl, '0'
    cmp dl, '9'
    jbe .phw_print
    add dl, 7
.phw_print:
    mov ah, 0x0E
    int 0x10
    loop .phw_digit
    
    pop dx
    pop cx
    pop bx
    pop ax
    ret

custom_bios_function:
    ; Custom BIOS function
    mov si, msg_bios
    call print_string
    ret

; ===== Data Section =====
msg_welcome:
    db 'Rython BIOS Bootloader', 13, 10
    db '=======================', 13, 10, 13, 10, 0

msg_halt:
    db 13, 10, 'Program complete. System halted.', 13, 10, 0

msg_newline:
    db 13, 10, 0

msg_bios:
    db 'BIOS function called!', 13, 10, 0

    ; Boot signature
    times 510-($-$$) db 0
    dw 0xAA55
