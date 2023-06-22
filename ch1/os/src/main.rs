#![feature(panic_info_message)]
#![no_std] // tell rustc not use the standard library
#![no_main] // the simplest way to disable the 'start' program to initialize env

mod lang_items;
mod sbi;
#[macro_use]
mod console;
mod logging;
use log::*;

use core::arch::global_asm;

use crate::sbi::shutdown;

global_asm!(include_str!("entry.asm"));

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

    extern "C" {
        fn skernel();
        fn ekernel();
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn _start();
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }

    info!("=> .text [{:#x}, {:#x})", stext as usize, etext as usize);

    info!("=> .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);

    info!("=> .data [{:#x}, {:#x})", sdata as usize, edata as usize);

    info!("=> .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    info!("kernel load range: [{:#x}, {:#x}] start={:#x}",
        skernel as usize, ekernel as usize, _start as usize);

    info!(
        "boot_stack top/bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize);

    println!("Hello, world!");
    shutdown();
}
