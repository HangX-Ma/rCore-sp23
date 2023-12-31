#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[macro_use]
extern crate user_lib;

use user_lib::getpid;

/*
辅助测例 打印子进程 pid
*/

#[no_mangle]
pub fn main() -> i32 {
    let pid = getpid();
    println!("Test getpid OK! pid = {}", pid);
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}