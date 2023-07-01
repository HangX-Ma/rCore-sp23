#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[macro_use]
extern crate user_lib;

use user_lib::yield_;

const WIDTH: usize = 10;
const HEIGHT: usize = 5;

/*
理想结果：三个程序交替输出 ABC
*/

#[no_mangle]
fn main() -> i32 {
    for i in 0..HEIGHT {
        let buf = ['A' as u8; WIDTH];
        println!(
            "{} [{}/{}]",
            core::str::from_utf8(&buf).unwrap(),
            i + 1,
            HEIGHT
        );
        yield_();
    }
    println!("Test write A OK!");
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}