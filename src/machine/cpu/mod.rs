use super::memory::{MemoryBus, MemoryError};
// Since the «constants» module provides everything spec-related that is needed to implement this CPU, everything from there is imported without an alias
use self::constants::*;
use log::debug;

pub mod constants;

#[derive(Debug)]
pub enum CpuError {
    Fetch(FetchError),
    Decode(DecodeError),
    Execute(ExecuteError),
}

#[derive(Debug)]
pub enum FetchError {
    Memory(MemoryError),
}

#[derive(Debug)]
pub enum DecodeError {
    OpcodeZero,
}

#[derive(Debug)]
pub enum ExecuteError {
    // A memory error can be encountered during execution of a load or store instruction
    Memory(MemoryError),
}

impl From<FetchError> for CpuError {
    fn from(value: FetchError) -> Self {
        Self::Fetch(value)
    }
}

impl From<DecodeError> for CpuError {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
    }
}

pub struct Cpu {
    /// Registers x0-x31, with x0 emulated as being hardwired to zero
    pub registers: [XLENType; 32],
    /// Program counter
    pub pc: XLENType,
}

impl Cpu {
    pub fn new(reset_vector: usize) -> Self {
        Self {
            registers: [0; 32],
            pc: reset_vector as u32,
        }
    }

    pub fn rewind(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub fn advance(&mut self, mut memory_bus: &mut MemoryBus) -> Result<(), CpuError> {
        self.registers[0] = 0; // Emulates x0 being hardwired to zero
        debug!(
            "New instruction cycle started\nRegisters: {:?}\nPC: {:?}",
            self.registers, self.pc,
        );
        // 1) Fetch
        let raw_instruction = self.fetch(&memory_bus)?;
        // Increment the program counter (by four bytes, since every instruction is 32 bits long)
        // Note: In the compressed instruction set instructions can be 16 bits long only
        self.pc += 4;
        // 2) Decode
        let instruction = self.decode(raw_instruction)?;
        // 3) Execute
        self.execute(instruction, &mut memory_bus)?;
        Ok(())
    }

    /// Emulates the CPU receiveing a reset signal
    pub fn reset(&mut self, mut memory_bus: &mut MemoryBus) -> Result<(), CpuError> {
        while self.pc < memory_bus.memory.size() as u32 {
            self.advance(&mut memory_bus)?;
        }
        Ok(())
    }

    fn fetch(&self, memory_bus: &MemoryBus) -> Result<u32, FetchError> {
        // Note: While here the fetch is always for 4 bytes (indicated by size: 32 (bits)), in the compressed instruction set instructions can be 16 bits long only
        let raw_instruction = memory_bus.load(self.pc as usize, 32)? as u32;
        debug!(
            "Fetch phase succeded\nRaw instruction: {:?}",
            raw_instruction
        );
        Ok(raw_instruction)
    }

    fn decode(&self, raw_instruction: u32) -> Result<Instruction, DecodeError> {
        let decoded_instruction = Instruction::try_from(raw_instruction)?;
        debug!(
            "Decode phase succeded\nDecoded instruction: {:?}",
            decoded_instruction
        );
        Ok(decoded_instruction)
    }

    fn execute(
        &mut self,
        instruction: Instruction,
        memory_bus: &mut MemoryBus,
    ) -> Result<(), ExecuteError> {
        debug!("Execute phase started");
        match instruction {
            Instruction::I(instruction) => {
                match instruction.opcode {
                    // Load instructions
                    0x03 => {
                        // Memory address
                        let address = self.registers[instruction.rs1 as usize]
                            .wrapping_add(instruction.imm)
                            as usize; // As usize since it will always be used to index the contents of the memory
                        match instruction.funct3 {
                            // lb
                            0x0 => {
                                let val = memory_bus.load(address, 8)?;
                                self.registers[instruction.rd as usize] = val as i8 as i32 as u32;
                            }
                            // lh
                            0x1 => {
                                let val = memory_bus.load(address, 16)?;
                                self.registers[instruction.rd as usize] = val as i16 as i32 as u32;
                            }
                            // lw
                            0x2 => {
                                let val = memory_bus.load(address, 32)?;
                                self.registers[instruction.rd as usize] = val as i32 as u32;
                            }
                            // lbu
                            0x4 => {
                                let val = memory_bus.load(address, 8)?;
                                self.registers[instruction.rd as usize] = val as u32;
                            }
                            // lhu
                            0x5 => {
                                let val = memory_bus.load(address, 16)?;
                                self.registers[instruction.rd as usize] = val as u32;
                            }
                            _ => {}
                        }
                    }
                    // Operations on registers
                    0x13 => {
                        match instruction.funct3 {
                            // addi
                            0x0 => {
                                self.registers[instruction.rd as usize] = self.registers
                                    [instruction.rs1 as usize]
                                    .wrapping_add(instruction.imm);
                            }
                            _ => unimplemented!(
                                "Unsupported instruction, detected while analyzing funct3"
                            ),
                        }
                    }
                    _ => unimplemented!("Unsupported instruction, detected while analyzing opcode"),
                }
            }
            Instruction::R(instruction) => {
                match instruction.opcode {
                    0x33 => {
                        match (instruction.funct3, instruction.funct7) {
                            // add
                            (0x0, 0x0) => {
                                self.registers[instruction.rd as usize] = self.registers
                                    [instruction.rs1 as usize]
                                    .wrapping_add(self.registers[instruction.rs2 as usize]);
                            },
                            // sub
                            (0x0, 0x20) => {
                                self.registers[instruction.rd as usize] = self.registers
                                    [instruction.rs1 as usize]
                                    .wrapping_sub(self.registers[instruction.rs2 as usize]);
                            }
                            _ => unimplemented!(
                                "Unsupported instruction, detected while analyzing funct3 and funct7"
                            ),
                        }
                    }
                    _ => unimplemented!("Unsupported instruction, detected while analyzing opcode"),
                }
            }
            Instruction::S(instruction) => {
                // Memory address
                let address =
                    self.registers[instruction.rs1 as usize].wrapping_add(instruction.imm);
                match instruction.opcode {
                    0x23 => {
                        match instruction.funct3 {
                            0x0 => memory_bus.store(
                                address as usize,
                                8,
                                self.registers[instruction.rs2 as usize] as usize,
                            )?, // sb
                            0x1 => memory_bus.store(
                                address as usize,
                                16,
                                self.registers[instruction.rs2 as usize] as usize,
                            )?, // sh
                            0x2 => memory_bus.store(
                                address as usize,
                                32,
                                self.registers[instruction.rs2 as usize] as usize,
                            )?, // sw
                            _ => {}
                        }
                    }
                    _ => unimplemented!("Unsupported instruction, detected while analyzing opcode"),
                }
            }
            _ => todo!(),
        };
        debug!("Succesfully executed instruction");
        Ok(())
    }
}

impl From<ExecuteError> for CpuError {
    fn from(value: ExecuteError) -> Self {
        Self::Execute(value)
    }
}

impl From<MemoryError> for FetchError {
    fn from(value: MemoryError) -> Self {
        Self::Memory(value)
    }
}

impl From<MemoryError> for ExecuteError {
    fn from(value: MemoryError) -> Self {
        Self::Memory(value)
    }
}
