//! Constants used in rCore


/// user app's stack size
pub const USER_STACK_SIZE: usize = 4096 * 2;
/// kernel stack size
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
/// kernel heap size
pub const KERNEL_HEAP_SIZE: usize = 0x200_0000;
/// page size is 4k
pub const PAGE_SIZE: usize = 0x1000;
/// page size bits is 12 bits
pub const PAGE_SIZE_BITS: usize = 0xc;
/// the physical memory end, 128 MB
pub const MEMORY_END: usize = 0x88000000;

/// the max number of syscall
pub const MAX_SYSCALL_NUM: usize = 500;

// virtual memory space settings
pub const MAXVA: usize = usize::MAX;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - PAGE_SIZE;

pub use crate::board::{CLOCK_FREQ, MMIO};

pub const BIG_STRIDE: u64 = u64::MAX;
