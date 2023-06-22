#![feature(panic_info_message)]
#![no_std] // tell rustc not use the standard library
#![no_main] // the simplest way to disable the 'start' program to initialize env

mod lang_items;
mod sbi;
#[macro_use]
mod console;

use core::arch::global_asm;

use crate::sbi::shutdown;

global_asm!(include_str!("entry.asm"));

#[no_mangle] // avoid compiler confusion
fn rust_main() {
    // do nothing
    clear_bss();
    println!("Hello, world!");
    shutdown();
}

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
