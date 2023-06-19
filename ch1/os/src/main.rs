#![no_std] // tell rustc not use the standard library
#![no_main] // the simplest way to disable the 'start' program to initialize env

mod lang_items;

use core::arch::global_asm;

global_asm!(include_str!("entry.asm"));

fn main() {
    // do nothing
}
