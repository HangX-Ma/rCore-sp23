#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[macro_use]
extern crate user_lib;
use user_lib::set_priority;

/// 正确输出：（无报错信息）
/// Test set_priority OK!

#[no_mangle]
pub fn main() -> i32 {
    assert_eq!(set_priority(10), 10);
    assert_eq!(set_priority(isize::MAX), isize::MAX);
    assert_eq!(set_priority(0), -1);
    assert_eq!(set_priority(1), -1);
    assert_eq!(set_priority(-10), -1);
    println!("Test set_priority OK!");
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}