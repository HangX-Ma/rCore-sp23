#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[macro_use]
extern crate user_lib;

use user_lib::{get_time, sleep};

#[no_mangle]
pub fn main() -> i32 {
    let start = get_time();
    println!("current time_msec = {}", start);
    sleep(100);
    let end = get_time();
    println!(
        "time_msec = {} after sleeping 100 ticks, delta = {}ms!",
        end,
        end - start
    );
    println!("Test sleep1 passed!");
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}