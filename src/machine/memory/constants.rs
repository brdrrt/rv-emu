pub const MEMORY_SIZE: usize = 4 * (1 << 10); // 4 KiB

/// The address at which the actual physical memory starts, everything before this isn't real memory (e.g. Memory Mapped I/O)
pub const RAM_BASE: usize = 0x80;
