//! This module defines constants as per the RISC-V ISA spec, some of them might be implementation specific

use crate::machine::cpu::DecodeError;

pub const XLEN: u8 = 32;
pub type XLENType = u32;

#[derive(Debug)]
pub enum Instruction {
    R(RType),
    I(IType),
    S(SType),
    B,
    U,
    J,
}

#[derive(Debug)]
pub struct IType {
    /// Opcode, partially identifies the instruction
    pub opcode: u32,
    /// Destination register
    pub rd: u32,
    /// Source register n. 1
    pub rs1: u32,
    /// Immediate
    pub imm: u32,
    /// Complements the opcode in identifying the instruction
    pub funct3: u32,
}

#[derive(Debug)]
pub struct RType {
    /// Opcode, partially identifies the instruction
    pub opcode: u32,
    /// Destination register
    pub rd: u32,
    /// Source register n. 1
    pub rs1: u32,
    /// Source register n. 2
    pub rs2: u32,
    /// Complements the opcode and funct7 in identifying the instruction
    pub funct3: u32,
    /// Complements the opcode and funct3 in identifying the instruction
    pub funct7: u32,
}

#[derive(Debug)]
pub struct SType {
    /// Opcode, partially identifies the instruction
    pub opcode: u32,
    /// Source register n. 1
    pub rs1: u32,
    /// Source register n. 2
    pub rs2: u32,
    /// Immediate
    pub imm: u32,
    /// Complements the opcode and funct7 in identifying the instruction
    pub funct3: u32,
}

impl TryFrom<u32> for Instruction {
    type Error = DecodeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let opcode = value & 0x7f;
        match opcode {
            // I Type
            0x03 | 0x13 => {
                let rd = decode_destination_register(value);
                // rs2 is ignored since it doesn't actually exist in I-type instructions
                let (rs1, _) = decode_source_registers(value);
                let imm = ((value & 0xfff00000) as i32 >> 20) as u32;
                // funct7 is ignored since it doesn't actually exist in I-type instructions
                let (funct3, _) = decode_functs(value);
                Ok(Self::I(IType {
                    opcode,
                    rd,
                    rs1,
                    imm,
                    funct3,
                }))
            }
            // S Type
            0x23 => {
                let (rs1, rs2) = decode_source_registers(value);
                // funct7 is ignored since it doesn't actually exist in S-type instructions
                let (funct3, _) = decode_functs(value);
                let imm =
                    (((value & 0xfe000000) as i32 as i64 >> 20) as u32) | ((value >> 7) & 0x1f);
                Ok(Self::S(SType {
                    opcode,
                    rs1,
                    rs2,
                    imm,
                    funct3,
                }))
            }
            // R Type
            0x33 => {
                let rd = decode_destination_register(value);
                let (rs1, rs2) = decode_source_registers(value);
                let (funct3, funct7) = decode_functs(value);
                Ok(Self::R(RType {
                    opcode,
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                }))
            }
            0x0 => Err(DecodeError::OpcodeZero),
            _ => unimplemented!(),
        }
    }
}

/// Decodes the source register(s) from a raw instruction (rs1 and rs2)
fn decode_source_registers(raw_instruction: u32) -> (u32, u32) {
    (
        ((raw_instruction >> 15) & 0x1f) as u32,
        ((raw_instruction >> 20) & 0x1f) as u32,
    )
}

/// Decodes the destination register from a raw instruction (rd)
fn decode_destination_register(raw_instruction: u32) -> u32 {
    ((raw_instruction >> 7) & 0x1f) as u32
}

/// Decodes the funct3 and funct7 register from a raw instruction
fn decode_functs(raw_instruction: u32) -> (u32, u32) {
    (
        ((raw_instruction >> 12) & 0x7),
        ((raw_instruction >> 25) & 0x3F),
    )
}
