#![no_std] // tell rustc not use the standard library
#![no_main] // the simplest way to disable the 'start' program to initialize env
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(strict_provenance)]
// customized tests
#![reexport_test_harness_main = "test_main"] // help us create new `main` entry for test
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![feature(pointer_byte_offsets)]

#[path = "boards/qemu.rs"]
mod board;

extern crate alloc;
#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod console;
mod config;
mod lang_items;
mod loader;
mod mm;
mod logging;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
pub mod trap;
mod timer;
// ch2-problems
mod stack_btrace;
use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

fn clear_bss() {
    extern "C" {
        static mut sbss: u64;
        static mut ebss: u64;
    }

    unsafe {
        (sbss as usize..ebss as usize).for_each(|ptr|{
                // use volatile to avoid compiler optimization
                (ptr as *mut u8).write_volatile(0);
            }
        );
    }
}

#[no_mangle] // avoid compiler confusion
fn rust_main() {
    clear_bss();
    logging::init();

    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    // mm tests
    mm::heap_test();
    mm::frame_allocator_test();
    mm::remap_test();

    trap::init();
    //trap::enable_interrupt();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

#[cfg(test)] // ensure this function only runs in test scenario
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    // use crate::board::QEMUExit;
    // crate::board::QEMU_EXIT_HANDLE.exit_success(); // CI autotest success
}