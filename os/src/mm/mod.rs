//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use heap_allocator::heap_test;
pub use frame_allocator::frame_allocator_test;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE, kernel_token};
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_str, translated_refmut, 
    PageTableEntry, PTEFlags, PageTable, UserBuffer, UserBufferIterator
};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}

