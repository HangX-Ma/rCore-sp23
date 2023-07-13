#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, wait};

#[no_mangle]
pub fn main() -> i32 {
    for i in 0..1000 {
        if fork() == 0 {
            exec("ch7b_pipe_large_test\0", &[0 as *const u8]);
        } else {
            let mut _unused: i32 = 0;
            wait(&mut _unused);
            println!("Iter {} OK.", i);
        }
    }
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}
