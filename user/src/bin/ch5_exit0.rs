#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

extern crate user_lib;
use user_lib::exit;

/*
辅助测例，正常退出，不输出 FAIL 即可。
*/

#[allow(unreachable_code)]
#[no_mangle]
pub fn main() -> i32 {
    exit(66778);
    panic!("FAIL: T.T\n");
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}