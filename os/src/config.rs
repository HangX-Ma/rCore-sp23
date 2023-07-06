//! Constants used in rCore


/// user app's stack size
pub const USER_STACK_SIZE: usize = 4096 * 2;
/// kernel stack size
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
/// kernel heap size
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
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
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}


/*
#[cfg(feature = "board_k210")]
pub const CLOCK_FREQ: usize = 403000000 / 62;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
*/
pub use crate::board::{CLOCK_FREQ, MMIO};
