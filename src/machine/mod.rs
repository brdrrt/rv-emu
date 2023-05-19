use self::{
    cpu::{Cpu, CpuError},
    memory::{Memory, MemoryBus, MemoryDump},
};

pub mod cpu;
pub mod memory;

/// A generic machine
pub struct Machine {
    pub cpu: Cpu,
    pub memory: Memory,
}

#[derive(Debug)]
pub enum MachineError {
    Cpu(CpuError),
}

impl From<CpuError> for MachineError {
    fn from(value: CpuError) -> Self {
        Self::Cpu(value)
    }
}

impl Machine {
    pub fn new(memory_dump: MemoryDump) -> Self {
        let memory = memory_dump;
        Self {
            cpu: Cpu::new(0x80), // TODO: Make reset vector adjustable
            memory: Memory::new(memory),
        }
    }

    /// Boots and runs the machine normally
    pub fn boot(&mut self) -> Result<(), MachineError> {
        self.cpu.reset(&mut MemoryBus::new(&mut self.memory))?;
        Ok(())
    }
}
