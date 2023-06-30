//! App management syscalls
use crate::batch::{run_next_app, app_time_elapse};
// use super::stats::*; // lab2-pro3

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    // stats_clear_and_print(); // lab2-pro3
    app_time_elapse();
    run_next_app()
}
