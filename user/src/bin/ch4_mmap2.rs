#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

#[macro_use]
extern crate user_lib;

use user_lib::mmap;

/*
理想结果：程序触发访存异常，被杀死。不输出 error 就算过。
*/

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 2;
    assert_eq!(0, mmap(start, len, prot));
    let addr: *mut u8 = start as *mut u8;
    unsafe {
        // *addr = start as u8; // can't write, R == 0 && W == 1 is illegal in riscv
        assert!(*addr != 0);
    }
    println!("Should cause error, Test 04_2 fail!");
    0
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}