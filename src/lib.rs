pub use machine::*;
use machine::{memory::MemoryDump, Machine};

pub mod machine;

pub fn create_rv32(memory_dump: MemoryDump) -> Machine {
    Machine::new(memory_dump)
}

/// Available program modes
///
/// For more info on this see (TBD)
pub enum ProgramMode {
    BareMetal,
    Kernel,
    OsProvided,
}
