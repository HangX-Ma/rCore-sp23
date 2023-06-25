#![no_std] // tell rustc not use the standard library
#![no_main] // the simplest way to disable the 'start' program to initialize env
#![feature(panic_info_message)]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

pub fn test_runner(_test: &[&i32]) {
    loop {}
}