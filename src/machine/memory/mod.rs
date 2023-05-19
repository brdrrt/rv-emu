pub mod constants;

pub type MemoryDump = Vec<u8>;

// Since the «constants» module provides the specifications that are needed to implement this memory, everything from there is imported without an alias
use constants::*;

pub struct Memory {
    pub contents: MemoryDump,
}

impl Memory {
    pub fn size(&self) -> usize {
        self.contents.len()
    }

    pub fn new(mut memory_dump: MemoryDump) -> Self {
        memory_dump.resize(MEMORY_SIZE, 0); // Resize to be the desired lenght

        Self {
            contents: memory_dump,
        }
    }
}

/// Memory bus
///
/// This doesn't emulate the control/address buses and there is no MAR or MDR on the CPU
pub struct MemoryBus<'a> {
    pub memory: &'a mut Memory,
}

impl<'a> MemoryBus<'a> {
    pub fn new(memory: &'a mut Memory) -> Self {
        Self { memory }
    }

    pub fn load(&self, address: usize, size: usize) -> Result<usize, MemoryError> {
        if address >= RAM_BASE {
            match size {
                8 => Ok(self.load8(address)),
                16 => todo!(),
                32 => Ok(self.load32(address)),
                64 => todo!(),
                _ => Err(MemoryError::UnsupportedAddressingSize),
            }
        } else {
            todo!("Accessing anything other than actual memory is yet to be implemented")
        }
    }

    pub fn store(&mut self, address: usize, size: usize, value: usize) -> Result<(), MemoryError> {
        if address >= RAM_BASE {
            match size {
                8 => todo!(),
                16 => todo!(),
                32 => Ok(self.store32(address, value)),
                64 => todo!(),
                _ => Err(MemoryError::UnsupportedAddressingSize),
            }
        } else {
            todo!("Accessing anything other than actual memory is yet to be implemented")
        }
    }

    // TODO: Return as the correct type instead of usize

    fn load8(&self, address: usize) -> usize {
        let index = (address - RAM_BASE) as usize;
        return self.memory.contents[index] as usize;
    }

    fn load32(&self, address: usize) -> usize {
        let index = (address - RAM_BASE) as usize;
        return (self.memory.contents[index] as usize)
            | ((self.memory.contents[index + 1] as usize) << 8)
            | ((self.memory.contents[index + 2] as usize) << 16)
            | ((self.memory.contents[index + 3] as usize) << 24);
    }

    fn store32(&mut self, address: usize, value: usize) {
        let index = (address - RAM_BASE) as usize;
        self.memory.contents[index] = (value & 0xff) as u8;
        self.memory.contents[index + 1] = ((value >> 8) & 0xff) as u8;
        self.memory.contents[index + 2] = ((value >> 16) & 0xff) as u8;
        self.memory.contents[index + 3] = ((value >> 24) & 0xff) as u8;
    }
}

#[derive(Debug)]
pub enum MemoryError {
    UnsupportedAddressingSize,
}
