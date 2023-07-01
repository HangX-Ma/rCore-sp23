#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]


#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> i32 {
    println!("EMPTY MAIN");
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}