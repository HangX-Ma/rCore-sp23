//! App management syscalls
use crate::batch::{run_next_app, time_elapse};

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    time_elapse();
    run_next_app()
}
