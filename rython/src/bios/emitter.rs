//! x86 Opcode Emitter for Rython BIOS
//! Generates machine code using the official Intel opcode maps

use std::collections::HashMap;

// ========== OPCODE MAP STRUCTURES ==========

#[derive(Debug, Clone)]
struct OpcodeEntry {
    #[allow(dead_code)]
    mnemonic: String,
    #[allow(dead_code)]
    operands: Vec<String>,
    #[allow(dead_code)]
    extras: Vec<String>,
    #[allow(dead_code)]
    second_mnemonic: Option<String>,
    #[allow(dead_code)]
    avx_prefix: Option<u8>,
    #[allow(dead_code)]
    vex_prefix: Option<u8>,
    #[allow(dead_code)]
    evex_prefix: bool,
    #[allow(dead_code)]
    xop_prefix: bool,
    #[allow(dead_code)]
    last_prefix: Option<u8>,
    #[allow(dead_code)]
    rex2_allowed: bool,
    #[allow(dead_code)]
    mode_64: bool,
}

#[derive(Debug, Clone)]
struct GroupTable {
    name: String,
    entries: HashMap<u8, OpcodeEntry>,
}

#[derive(Debug)]
pub struct OpcodeEmitter {
    one_byte_map: HashMap<u8, OpcodeEntry>,
    two_byte_map: HashMap<u8, OpcodeEntry>,
    three_byte_map_1: HashMap<u8, OpcodeEntry>,
    three_byte_map_2: HashMap<u8, OpcodeEntry>,
    evex_map_4: HashMap<u8, OpcodeEntry>,
    evex_map_5: HashMap<u8, OpcodeEntry>,
    evex_map_6: HashMap<u8, OpcodeEntry>,
    vex_map_7: HashMap<u8, OpcodeEntry>,
    xop_map_8: HashMap<u8, OpcodeEntry>,
    xop_map_9: HashMap<u8, OpcodeEntry>,
    xop_map_a: HashMap<u8, OpcodeEntry>,
    groups: HashMap<String, GroupTable>,
}

// ========== REGISTER ENCODINGS ==========

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    AL, CL, DL, BL, AH, CH, DH, BH,
    AX, CX, DX, BX, SP, BP, SI, DI,
    EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI,
    RAX, RCX, RDX, RBX, RSP, RBP, RSI, RDI,
    R8, R9, R10, R11, R12, R13, R14, R15,
    R8B, R9B, R10B, R11B, R12B, R13B, R14B, R15B,
    R8W, R9W, R10W, R11W, R12W, R13W, R14W, R15W,
    R8D, R9D, R10D, R11D, R12D, R13D, R14D, R15D,
    ES, CS, SS, DS, FS, GS,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Register(Register),
    Memory(MemoryOperand),
    Immediate8(u8),
    Immediate16(u16),
    Immediate32(u32),
    Immediate64(u64),
    Label(String),
    Relative8(i8),
    Relative16(i16),
    Relative32(i32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryOperand {
    pub segment: Option<Register>,
    pub base: Option<Register>,
    pub index: Option<Register>,
    pub scale: u8,
    pub displacement: i32,
}

// ========== INSTRUCTION STRUCTURE ==========

pub struct Instruction {
    pub mnemonic: String,
    pub operands: Vec<Operand>,
    pub prefixes: Vec<u8>,
    pub size_prefix: bool,
    pub address_prefix: bool,
    pub lock_prefix: bool,
    pub rep_prefix: bool,
    pub repne_prefix: bool,
}

impl OpcodeEmitter {
    pub fn new() -> Self {
        let mut emitter = Self {
            one_byte_map: HashMap::new(),
            two_byte_map: HashMap::new(),
            three_byte_map_1: HashMap::new(),
            three_byte_map_2: HashMap::new(),
            evex_map_4: HashMap::new(),
            evex_map_5: HashMap::new(),
            evex_map_6: HashMap::new(),
            vex_map_7: HashMap::new(),
            xop_map_8: HashMap::new(),
            xop_map_9: HashMap::new(),
            xop_map_a: HashMap::new(),
            groups: HashMap::new(),
        };
        
        emitter.parse_opcode_maps();
        emitter
    }
    
    fn parse_opcode_maps(&mut self) {
        // Parse the opcode.txt file content
        let content = include_str!("../../opcode.txt");
        let mut _current_table = String::new();
        let mut current_map: Option<&mut HashMap<u8, OpcodeEntry>> = None;
        let mut in_group = false;
        let mut current_group: Option<GroupTable> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            if line.starts_with("Table:") {
                _current_table = line[6..].trim().to_string();
                current_map = match _current_table.as_str() {
                    "one byte opcode" => Some(&mut self.one_byte_map),
                    "2-byte opcode (0x0f)" => Some(&mut self.two_byte_map),
                    "3-byte opcode 1 (0x0f 0x38)" => Some(&mut self.three_byte_map_1),
                    "3-byte opcode 2 (0x0f 0x3a)" => Some(&mut self.three_byte_map_2),
                    "EVEX map 4" => Some(&mut self.evex_map_4),
                    "EVEX map 5" => Some(&mut self.evex_map_5),
                    "EVEX map 6" => Some(&mut self.evex_map_6),
                    "VEX map 7" => Some(&mut self.vex_map_7),
                    "XOP map 8h" => Some(&mut self.xop_map_8),
                    "XOP map 9h" => Some(&mut self.xop_map_9),
                    "XOP map Ah" => Some(&mut self.xop_map_a),
                    _ => None,
                };
                in_group = false;
            } else if line.starts_with("GrpTable:") {
                in_group = true;
                let group_name = line[9..].trim().to_string();
                current_group = Some(GroupTable {
                    name: group_name.clone(),
                    entries: HashMap::new(),
                });
            } else if line == "EndTable" {
                if in_group {
                    if let Some(group) = current_group.take() {
                        self.groups.insert(group.name.clone(), group);
                    }
                    in_group = false;
                }
                current_map = None;
            } else if line.starts_with("reg:") && in_group {
                if let Some(ref mut group) = current_group {
                    let parts: Vec<&str> = line[4..].split_whitespace().collect();
                    if parts.len() >= 2 {
                        let reg_num = parts[0].parse::<u8>().unwrap_or(0);
                        let entry = Self::parse_opcode_entry(&parts[1..]);
                        group.entries.insert(reg_num, entry);
                    }
                }
            } else if let Some(opcode_str) = line.split(':').next() {
                if let Ok(opcode) = u8::from_str_radix(opcode_str.trim(), 16) {
                    if let Some(ref mut map) = current_map {
                        let entry_str = line[opcode_str.len()+1..].trim();
                        if entry_str.starts_with("escape") {
                            // Skip escape codes for now
                            continue;
                        }
                        let entry = Self::parse_opcode_entry(&[entry_str]);
                        map.insert(opcode, entry);
                    }
                }
            }
        }
    }
    
    fn parse_opcode_entry(parts: &[&str]) -> OpcodeEntry {
        let mut mnemonic = String::new();
        let mut operands = Vec::new();
        let mut extras = Vec::new();
        let mut second_mnemonic = None;
        let avx_prefix = None;
        let mut vex_prefix = None;
        let mut evex_prefix = false;
        let mut xop_prefix = false;
        let mut last_prefix = None;
        let mut rex2_allowed = false; // FIXED: Changed from using undefined reg1, reg2
        let mut mode_64 = false;
        
        if parts.is_empty() {
            return OpcodeEntry {
                mnemonic,
                operands,
                extras,
                second_mnemonic,
                avx_prefix,
                vex_prefix,
                evex_prefix,
                xop_prefix,
                last_prefix,
                rex2_allowed,
                mode_64,
            };
        }
        
        let first_part = parts[0];
        if first_part.contains('|') {
            let subparts: Vec<&str> = first_part.split('|').collect();
            mnemonic = subparts[0].trim().to_string();
            if subparts.len() > 1 {
                second_mnemonic = Some(subparts[1].trim().to_string());
            }
        } else {
            mnemonic = first_part.to_string();
        }
        
        // Parse operands and extras
        for part in parts.iter().skip(1) {
            if part.starts_with('(') && part.ends_with(')') {
                let extra = &part[1..part.len()-1];
                match extra {
                    "ev" => evex_prefix = true,
                    "v" => vex_prefix = Some(1),
                    "v1" => vex_prefix = Some(1),
                    "xop" => xop_prefix = true,
                    "66" => last_prefix = Some(0x66),
                    "F3" => last_prefix = Some(0xF3),
                    "F2" => last_prefix = Some(0xF2),
                    "!F3" => {},
                    "!REX2" => rex2_allowed = true,
                    "REX2" => {},
                    "i64" => mode_64 = false,
                    "o64" => mode_64 = true,
                    "d64" => mode_64 = true,
                    "f64" => {},
                    _ => extras.push(extra.to_string()),
                }
            } else if part.contains(',') {
                operands.extend(part.split(',').map(|s| s.trim().to_string()));
            } else {
                operands.push(part.to_string());
            }
        }
        
        OpcodeEntry {
            mnemonic,
            operands,
            extras,
            second_mnemonic,
            avx_prefix,
            vex_prefix,
            evex_prefix,
            xop_prefix,
            last_prefix,
            rex2_allowed,
            mode_64,
        }
    }
    
    // ========== ENCODING FUNCTIONS ==========
    
    pub fn encode_instruction(&self, instr: &Instruction) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        
        // Add prefixes
        if instr.lock_prefix {
            bytes.push(0xF0);
        }
        if instr.rep_prefix {
            bytes.push(0xF3);
        }
        if instr.repne_prefix {
            bytes.push(0xF2);
        }
        if instr.size_prefix {
            bytes.push(0x66);
        }
        if instr.address_prefix {
            bytes.push(0x67);
        }
        
        // Main opcode encoding
        let opcode_bytes = self.encode_opcode(&instr.mnemonic, &instr.operands)?;
        bytes.extend(opcode_bytes);
        
        Ok(bytes)
    }
    
    fn encode_opcode(&self, mnemonic: &str, operands: &[Operand]) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        
        // Simple mapping for common instructions used in bootloader
        match mnemonic.to_uppercase().as_str() {
            "MOV" => self.encode_mov(operands, &mut bytes)?,
            "ADD" => self.encode_add(operands, &mut bytes)?,
            "SUB" => self.encode_sub(operands, &mut bytes)?,
            "CMP" => self.encode_cmp(operands, &mut bytes)?,
            "JMP" => self.encode_jmp(operands, &mut bytes)?,
            "CALL" => self.encode_call(operands, &mut bytes)?,
            "PUSH" => self.encode_push(operands, &mut bytes)?,
            "POP" => self.encode_pop(operands, &mut bytes)?,
            "RET" => self.encode_ret(operands, &mut bytes)?,
            "INT" => self.encode_int(operands, &mut bytes)?,
            "XCHG" => self.encode_xchg(operands, &mut bytes)?,
            "LEA" => self.encode_lea(operands, &mut bytes)?,
            "NOP" => bytes.push(0x90),
            "HLT" => bytes.push(0xF4),
            "CLC" => bytes.push(0xF8),
            "STC" => bytes.push(0xF9),
            "CLI" => bytes.push(0xFA),
            "STI" => bytes.push(0xFB),
            "CLD" => bytes.push(0xFC),
            "STD" => bytes.push(0xFD),
            "CMC" => bytes.push(0xF5),
            "NEG" => self.encode_neg(operands, &mut bytes)?,
            "NOT" => self.encode_not(operands, &mut bytes)?,
            "IMUL" => self.encode_imul(operands, &mut bytes)?,
            "IDIV" => self.encode_idiv(operands, &mut bytes)?,
            "SETE" => self.encode_sete(operands, &mut bytes)?,
            "SETNE" => self.encode_setne(operands, &mut bytes)?,
            "SETL" => self.encode_setl(operands, &mut bytes)?,
            "SETLE" => self.encode_setle(operands, &mut bytes)?,
            "SETG" => self.encode_setg(operands, &mut bytes)?,
            "SETGE" => self.encode_setge(operands, &mut bytes)?,
            "SETB" => self.encode_setb(operands, &mut bytes)?,
            "SETBE" => self.encode_setbe(operands, &mut bytes)?,
            "SETA" => self.encode_seta(operands, &mut bytes)?,
            "SETAE" => self.encode_setae(operands, &mut bytes)?,
            "MOVZX" => self.encode_movzx(operands, &mut bytes)?,
            _ => return Err(format!("Unknown mnemonic: {}", mnemonic)),
        }
        
        Ok(bytes)
    }
    
    fn encode_mov(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("MOV requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(dst), Operand::Immediate8(imm)) => {
                let reg_code = self.register_code_8(dst)?;
                bytes.push(0xB0 + reg_code);
                bytes.push(*imm);
            }
            (Operand::Register(dst), Operand::Immediate16(imm)) => {
                let reg_code = self.register_code_16(dst)?;
                bytes.push(0xB8 + reg_code);
                bytes.push(*imm as u8);
                bytes.push((*imm >> 8) as u8);
            }
            (Operand::Register(dst), Operand::Immediate32(imm)) => {
                let reg_code = self.register_code_32(dst)?;
                bytes.push(0xB8 + reg_code);
                bytes.extend(&imm.to_le_bytes());
            }
            (Operand::Register(dst), Operand::Immediate64(imm)) => {
                // 64-bit immediate mov requires REX.W prefix
                bytes.push(0x48); // REX.W prefix
                let reg_code = self.register_code(dst)?;
                bytes.push(0xB8 + reg_code);
                bytes.extend(&imm.to_le_bytes());
            }
            (Operand::Register(dst), Operand::Register(src)) => {
                // Check if 64-bit registers
                let is_64bit = match (dst, src) {
                    (Register::RAX | Register::RCX | Register::RDX | Register::RBX |
                     Register::RSP | Register::RBP | Register::RSI | Register::RDI |
                     Register::R8 | Register::R9 | Register::R10 | Register::R11 |
                     Register::R12 | Register::R13 | Register::R14 | Register::R15, _) => true,
                    _ => false,
                };
                
                if is_64bit {
                    bytes.push(0x48); // REX.W prefix for 64-bit
                }
                bytes.push(0x89);
                let modrm = self.modrm_byte(3, self.register_code(src)?, self.register_code(dst)?)?;
                bytes.push(modrm);
            }
            (Operand::Register(dst), Operand::Memory(src_mem)) => {
                bytes.push(0x8B);
                self.encode_memory_operand(src_mem, self.register_code(dst)?, bytes)?;
            }
            (Operand::Memory(dst_mem), Operand::Register(src)) => {
                bytes.push(0x89);
                self.encode_memory_operand(dst_mem, self.register_code(src)?, bytes)?;
            }
            _ => return Err("Unsupported MOV operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_add(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("ADD requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(dst), Operand::Immediate8(imm)) => {
                bytes.push(0x80);
                let modrm = self.modrm_byte(0, self.register_code(dst)?, 0)?;
                bytes.push(modrm);
                bytes.push(*imm);
            }
            (Operand::Register(dst), Operand::Immediate32(imm)) => {
                bytes.push(0x81);
                let modrm = self.modrm_byte(0, self.register_code(dst)?, 0)?;
                bytes.push(modrm);
                bytes.extend(&imm.to_le_bytes());
            }
            (Operand::Register(dst), Operand::Immediate64(imm)) => {
                bytes.push(0x48); // REX.W
                bytes.push(0x81);
                let modrm = self.modrm_byte(0, self.register_code(dst)?, 0)?;
                bytes.push(modrm);
                bytes.extend(&imm.to_le_bytes());
            }
            (Operand::Register(dst), Operand::Register(src)) => {
                bytes.push(0x01);
                let modrm = self.modrm_byte(3, self.register_code(src)?, self.register_code(dst)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported ADD operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_sub(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("SUB requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(dst), Operand::Immediate32(imm)) => {
                bytes.push(0x81);
                let modrm = self.modrm_byte(5, self.register_code(dst)?, 0)?;
                bytes.push(modrm);
                bytes.extend(&imm.to_le_bytes());
            }
            (Operand::Register(dst), Operand::Register(src)) => {
                bytes.push(0x29);
                let modrm = self.modrm_byte(3, self.register_code(src)?, self.register_code(dst)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported SUB operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_cmp(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("CMP requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(dst), Operand::Immediate8(imm)) => {
                bytes.push(0x80);
                let modrm = self.modrm_byte(7, self.register_code(dst)?, 0)?;
                bytes.push(modrm);
                bytes.push(*imm);
            }
            (Operand::Register(dst), Operand::Immediate32(imm)) => {
                bytes.push(0x81);
                let modrm = self.modrm_byte(7, self.register_code(dst)?, 0)?;
                bytes.push(modrm);
                bytes.extend(&imm.to_le_bytes());
            }
            (Operand::Register(dst), Operand::Register(src)) => {
                bytes.push(0x39);
                let modrm = self.modrm_byte(3, self.register_code(src)?, self.register_code(dst)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported CMP operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_jmp(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("JMP requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Relative8(rel) => {
                bytes.push(0xEB);
                bytes.push(*rel as u8);
            }
            Operand::Relative32(rel) => {
                bytes.push(0xE9);
                bytes.extend(&(*rel as i32).to_le_bytes());
            }
            Operand::Label(_label) => {
                // Placeholder - will be resolved by linker
                bytes.push(0xE9);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
            }
            _ => return Err("Unsupported JMP operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_call(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("CALL requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Relative32(rel) => {
                bytes.push(0xE8);
                bytes.extend(&(*rel as i32).to_le_bytes());
            }
            Operand::Label(_label) => {
                bytes.push(0xE8);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
                bytes.push(0x00);
            }
            _ => return Err("Unsupported CALL operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_push(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("PUSH requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Register(reg) => {
                let code = self.register_code_16(reg)?;
                bytes.push(0x50 + code);
            }
            Operand::Immediate32(imm) => {
                bytes.push(0x68);
                bytes.extend(&imm.to_le_bytes());
            }
            Operand::Immediate8(imm) => {
                bytes.push(0x6A);
                bytes.push(*imm);
            }
            _ => return Err("Unsupported PUSH operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_pop(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("POP requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Register(reg) => {
                let code = self.register_code_16(reg)?;
                bytes.push(0x58 + code);
            }
            _ => return Err("Unsupported POP operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_ret(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        match operands.len() {
            0 => bytes.push(0xC3),
            1 => {
                if let Operand::Immediate16(imm) = operands[0] {
                    bytes.push(0xC2);
                    bytes.extend(&imm.to_le_bytes());
                } else {
                    return Err("RET expects immediate operand".to_string());
                }
            }
            _ => return Err("RET expects 0 or 1 operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_int(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("INT requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Immediate8(imm) => {
                if *imm == 3 {
                    bytes.push(0xCC); // INT3 special case
                } else {
                    bytes.push(0xCD);
                    bytes.push(*imm);
                }
            }
            _ => return Err("INT expects immediate operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_xchg(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("XCHG requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(r1), Operand::Register(r2)) => {
                if *r1 == Register::AX && *r2 == Register::AX {
                    bytes.push(0x90); // NOP is XCHG AX,AX
                } else {
                    bytes.push(0x87);
                    let modrm = self.modrm_byte(3, self.register_code(r2)?, self.register_code(r1)?)?;
                    bytes.push(modrm);
                }
            }
            _ => return Err("Unsupported XCHG operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_lea(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("LEA requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(dst), Operand::Memory(src)) => {
                bytes.push(0x8D);
                self.encode_memory_operand(src, self.register_code(dst)?, bytes)?;
            }
            _ => return Err("Unsupported LEA operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_neg(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("NEG requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Register(reg) => {
                bytes.push(0xF7);
                let modrm = self.modrm_byte(3, 3, self.register_code(reg)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported NEG operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_not(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("NOT requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Register(reg) => {
                bytes.push(0xF7);
                let modrm = self.modrm_byte(3, 2, self.register_code(reg)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported NOT operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_imul(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        match operands.len() {
            1 => {
                // Single operand: imul reg
                match &operands[0] {
                    Operand::Register(reg) => {
                        bytes.push(0xF7);
                        let modrm = self.modrm_byte(3, 5, self.register_code(reg)?)?;
                        bytes.push(modrm);
                    }
                    _ => return Err("Unsupported IMUL operand".to_string()),
                }
            }
            2 => {
                // Two operands: imul dst, src
                match (&operands[0], &operands[1]) {
                    (Operand::Register(dst), Operand::Register(src)) => {
                        bytes.push(0x0F);
                        bytes.push(0xAF);
                        let modrm = self.modrm_byte(3, self.register_code(dst)?, self.register_code(src)?)?;
                        bytes.push(modrm);
                    }
                    _ => return Err("Unsupported IMUL operands".to_string()),
                }
            }
            _ => return Err("IMUL requires 1 or 2 operands".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_idiv(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("IDIV requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Register(reg) => {
                bytes.push(0xF7);
                let modrm = self.modrm_byte(3, 7, self.register_code(reg)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported IDIV operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_sete(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x94, bytes)
    }
    
    fn encode_setne(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x95, bytes)
    }
    
    fn encode_setl(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x9C, bytes)
    }
    
    fn encode_setle(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x9E, bytes)
    }
    
    fn encode_setg(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x9F, bytes)
    }
    
    fn encode_setge(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x9D, bytes)
    }
    
    fn encode_setb(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x92, bytes)
    }
    
    fn encode_setbe(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x96, bytes)
    }
    
    fn encode_seta(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x97, bytes)
    }
    
    fn encode_setae(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        self.encode_set_instruction(operands, 0x93, bytes)
    }
    
    fn encode_set_instruction(&self, operands: &[Operand], opcode_offset: u8, bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 1 {
            return Err("SET instruction requires exactly 1 operand".to_string());
        }
        
        match &operands[0] {
            Operand::Register(reg) => {
                bytes.push(0x0F); // 2-byte opcode prefix
                bytes.push(opcode_offset);
                let modrm = self.modrm_byte(3, 0, self.register_code_8(reg)?)?;
                bytes.push(modrm);
            }
            _ => return Err("SET instruction expects register operand".to_string()),
        }
        
        Ok(())
    }
    
    fn encode_movzx(&self, operands: &[Operand], bytes: &mut Vec<u8>) -> Result<(), String> {
        if operands.len() != 2 {
            return Err("MOVZX requires exactly 2 operands".to_string());
        }
        
        match (&operands[0], &operands[1]) {
            (Operand::Register(dst), Operand::Register(src)) => {
                bytes.push(0x0F);
                bytes.push(0xB6); // MOVZX r32, r8
                let modrm = self.modrm_byte(3, self.register_code(dst)?, self.register_code(src)?)?;
                bytes.push(modrm);
            }
            _ => return Err("Unsupported MOVZX operands".to_string()),
        }
        
        Ok(())
    }
    
    // ========== HELPER FUNCTIONS ==========
    
    fn register_code(&self, reg: &Register) -> Result<u8, String> {
        match reg {
            Register::AL | Register::AX | Register::EAX | Register::RAX => Ok(0),
            Register::CL | Register::CX | Register::ECX | Register::RCX => Ok(1),
            Register::DL | Register::DX | Register::EDX | Register::RDX => Ok(2),
            Register::BL | Register::BX | Register::EBX | Register::RBX => Ok(3),
            Register::AH | Register::SP | Register::ESP | Register::RSP => Ok(4),
            Register::CH | Register::BP | Register::EBP | Register::RBP => Ok(5),
            Register::DH | Register::SI | Register::ESI | Register::RSI => Ok(6),
            Register::BH | Register::DI | Register::EDI | Register::RDI => Ok(7),
            
            Register::R8 | Register::R8B | Register::R8W | Register::R8D => Ok(0),
            Register::R9 | Register::R9B | Register::R9W | Register::R9D => Ok(1),
            Register::R10 | Register::R10B | Register::R10W | Register::R10D => Ok(2),
            Register::R11 | Register::R11B | Register::R11W | Register::R11D => Ok(3),
            Register::R12 | Register::R12B | Register::R12W | Register::R12D => Ok(4),
            Register::R13 | Register::R13B | Register::R13W | Register::R13D => Ok(5),
            Register::R14 | Register::R14B | Register::R14W | Register::R14D => Ok(6),
            Register::R15 | Register::R15B | Register::R15W | Register::R15D => Ok(7),
            
            _ => Err(format!("Unsupported register: {:?}", reg)),
        }
    }
    
    fn register_code_8(&self, reg: &Register) -> Result<u8, String> {
        self.register_code(reg)
    }
    
    fn register_code_16(&self, reg: &Register) -> Result<u8, String> {
        self.register_code(reg)
    }
    
    fn register_code_32(&self, reg: &Register) -> Result<u8, String> {
        self.register_code(reg)
    }
    
    #[allow(dead_code)]
fn is_16bit_register(&self, reg: &Register) -> Result<bool, String> {
    match reg {
        Register::AX | Register::CX | Register::DX | Register::BX |
        Register::SP | Register::BP | Register::SI | Register::DI |
        Register::R8W | Register::R9W | Register::R10W | Register::R11W |
        Register::R12W | Register::R13W | Register::R14W | Register::R15W => Ok(true),
        Register::AL | Register::CL | Register::DL | Register::BL |
        Register::AH | Register::CH | Register::DH | Register::BH |
        Register::R8B | Register::R9B | Register::R10B | Register::R11B |
        Register::R12B | Register::R13B | Register::R14B | Register::R15B => Ok(false),
        Register::EAX | Register::ECX | Register::EDX | Register::EBX |
        Register::ESP | Register::EBP | Register::ESI | Register::EDI |
        Register::R8D | Register::R9D | Register::R10D | Register::R11D |
        Register::R12D | Register::R13D | Register::R14D | Register::R15D |
        Register::RAX | Register::RCX | Register::RDX | Register::RBX |
        Register::RSP | Register::RBP | Register::RSI | Register::RDI |
        Register::R8 | Register::R9 | Register::R10 | Register::R11 |
        Register::R12 | Register::R13 | Register::R14 | Register::R15 => Ok(false),
        _ => Err(format!("Unknown register: {:?}", reg)),
    }
}
    
    fn modrm_byte(&self, mode: u8, reg: u8, rm: u8) -> Result<u8, String> {
        if mode > 3 || reg > 7 || rm > 7 {
            return Err("Invalid ModR/M fields".to_string());
        }
        Ok((mode << 6) | (reg << 3) | rm)
    }
    
    fn encode_memory_operand(&self, mem: &MemoryOperand, reg_field: u8, bytes: &mut Vec<u8>) -> Result<(), String> {
        let mod_field: u8;
        let rm_field: u8;
        let mut displacement_bytes = Vec::new();
        
        // Determine addressing mode
        match (mem.base, mem.index) {
            (Some(base), None) => {
                rm_field = self.register_code(&base)?;
                if mem.displacement == 0 && base != Register::BP && base != Register::RBP {
                    mod_field = 0;
                } else if mem.displacement >= -128 && mem.displacement <= 127 {
                    mod_field = 1;
                    displacement_bytes.push(mem.displacement as u8);
                } else {
                    mod_field = 2;
                    displacement_bytes.extend(&(mem.displacement as u32).to_le_bytes());
                }
            }
            (Some(base), Some(index)) if mem.scale == 1 => {
                // [base + index]
                rm_field = 4; // SIB byte follows
                mod_field = if mem.displacement == 0 { 0 } else if mem.displacement >= -128 && mem.displacement <= 127 { 1 } else { 2 };
                if mod_field > 0 {
                    displacement_bytes.extend(&(mem.displacement as i32).to_le_bytes());
                }
                
                // Add SIB byte
                let sib = (mem.scale << 6) | (self.register_code(&index)? << 3) | self.register_code(&base)?;
                bytes.push(sib);
            }
            _ => return Err("Unsupported memory addressing mode".to_string()),
        }
        
        let modrm = self.modrm_byte(mod_field, reg_field, rm_field)?;
        bytes.push(modrm);
        bytes.extend(displacement_bytes);
        
        Ok(())
    }
}

// ========== PUBLIC INTERFACE ==========

impl Instruction {
    pub fn modrm(self, reg: u8, rm: u8) -> Self {
        // O byte ModR/M no x86-64 para modo registro-registro (0b11)
        let modrm_byte = 0xC0 | ((reg & 0x07) << 3) | (rm & 0x07);
        // ModR/M is stored in prefixes for now
        let mut new_self = self;
        new_self.prefixes.push(modrm_byte);
        new_self
    }
    
    pub fn new(mnemonic: &str) -> Self {
        Self {
            mnemonic: mnemonic.to_string(),
            operands: Vec::new(),
            prefixes: Vec::new(),
            size_prefix: false,
            address_prefix: false,
            lock_prefix: false,
            rep_prefix: false,
            repne_prefix: false,
        }
    }
    
    pub fn operand(mut self, op: Operand) -> Self {
        self.operands.push(op);
        self
    }
    
    pub fn operands(mut self, mut ops: Vec<Operand>) -> Self {
        self.operands.append(&mut ops);
        self
    }
    
    pub fn with_size_prefix(mut self) -> Self {
        self.size_prefix = true;
        self
    }
    
    pub fn with_address_prefix(mut self) -> Self {
        self.address_prefix = true;
        self
    }
    
    pub fn with_lock_prefix(mut self) -> Self {
        self.lock_prefix = true;
        self
    }
    
    pub fn with_rep_prefix(mut self) -> Self {
        self.rep_prefix = true;
        self
    }
    
    pub fn with_repne_prefix(mut self) -> Self {
        self.repne_prefix = true;
        self
    }
}

// Convenience functions for common instructions
pub fn mov(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("MOV").operands(vec![dst, src])
}

pub fn add(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("ADD").operands(vec![dst, src])
}

pub fn sub(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("SUB").operands(vec![dst, src])
}

pub fn cmp(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("CMP").operands(vec![dst, src])
}

pub fn jmp(target: Operand) -> Instruction {
    Instruction::new("JMP").operand(target)
}

pub fn call(target: Operand) -> Instruction {
    Instruction::new("CALL").operand(target)
}

pub fn push(operand: Operand) -> Instruction {
    Instruction::new("PUSH").operand(operand)
}

pub fn pop(operand: Operand) -> Instruction {
    Instruction::new("POP").operand(operand)
}

pub fn ret() -> Instruction {
    Instruction::new("RET")
}

pub fn ret_imm(imm: u16) -> Instruction {
    Instruction::new("RET").operand(Operand::Immediate16(imm))
}

pub fn int(imm: u8) -> Instruction {
    Instruction::new("INT").operand(Operand::Immediate8(imm))
}

pub fn nop() -> Instruction {
    Instruction::new("NOP")
}

pub fn xchg(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("XCHG").operands(vec![dst, src])
}

pub fn lea(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("LEA").operands(vec![dst, src])
}

pub fn neg(operand: Operand) -> Instruction {
    Instruction::new("NEG").operand(operand)
}

pub fn not(operand: Operand) -> Instruction {
    Instruction::new("NOT").operand(operand)
}

pub fn imul1(operand: Operand) -> Instruction {
    Instruction::new("IMUL").operand(operand)
}

pub fn imul2(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("IMUL").operands(vec![dst, src])
}

pub fn idiv(operand: Operand) -> Instruction {
    Instruction::new("IDIV").operand(operand)
}

pub fn sete(operand: Operand) -> Instruction {
    Instruction::new("SETE").operand(operand)
}

pub fn setne(operand: Operand) -> Instruction {
    Instruction::new("SETNE").operand(operand)
}

pub fn setl(operand: Operand) -> Instruction {
    Instruction::new("SETL").operand(operand)
}

pub fn setle(operand: Operand) -> Instruction {
    Instruction::new("SETLE").operand(operand)
}

pub fn setg(operand: Operand) -> Instruction {
    Instruction::new("SETG").operand(operand)
}

pub fn setge(operand: Operand) -> Instruction {
    Instruction::new("SETGE").operand(operand)
}

pub fn setb(operand: Operand) -> Instruction {
    Instruction::new("SETB").operand(operand)
}

pub fn setbe(operand: Operand) -> Instruction {
    Instruction::new("SETBE").operand(operand)
}

pub fn seta(operand: Operand) -> Instruction {
    Instruction::new("SETA").operand(operand)
}

pub fn setae(operand: Operand) -> Instruction {
    Instruction::new("SETAE").operand(operand)
}

pub fn movzx(dst: Operand, src: Operand) -> Instruction {
    Instruction::new("MOVZX").operands(vec![dst, src])
}