#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

extern crate user_lib;

use user_lib::{
    // println
    get_time, println, sleep, task_info, TaskInfo, TaskStatus, SYSCALL_EXIT, SYSCALL_GET_TIME,
    SYSCALL_TASK_INFO, SYSCALL_WRITE, SYSCALL_YIELD,
};

use user_lib::{fork, wait};

#[no_mangle]
pub fn main() -> usize {
    let pid = fork();
    if pid == 0 {
        // child process
        println!("taskinfo child process!");
        let t1 = get_time() as usize;
        let info = TaskInfo::new();
        get_time();
        sleep(500);
        let t2 = get_time() as usize;
        // 注意本次 task info 调用也计入
        assert_eq!(0, task_info(&info));
        let t3 = get_time() as usize;
        assert!(3 <= info.syscall_times[SYSCALL_GET_TIME]);
        assert_eq!(1, info.syscall_times[SYSCALL_TASK_INFO]);
        assert_eq!(1, info.syscall_times[SYSCALL_WRITE]);
        assert!(0 < info.syscall_times[SYSCALL_YIELD]);
        assert_eq!(0, info.syscall_times[SYSCALL_EXIT]);
        assert!(t2 - t1 <= info.time + 1);
        assert!(info.time < t3 - t1 + 100);
        assert!(info.status == TaskStatus::Running);

        // 想想为什么 write 调用是两次
        // BUG: 这里的测例是从 2023S Test 中拷贝的， 原来的测试环境里实现了 flush，
        // 并且 console 的实现方式和当前的不一致， 所以在现在的 OS 环境下仅有一次 write 调用
        // BUG: 调用 ch3_taskinfo.rs 需要保证其他的 test 程序不运行， 否则计时会出现很大偏差。
        println!("string from task info test\n");
        let t4 = get_time() as usize;
        assert_eq!(0, task_info(&info));
        let t5 = get_time() as usize;
        assert!(5 <= info.syscall_times[SYSCALL_GET_TIME]);
        assert_eq!(2, info.syscall_times[SYSCALL_TASK_INFO]);
        assert_eq!(2, info.syscall_times[SYSCALL_WRITE]);
        assert!(0 < info.syscall_times[SYSCALL_YIELD]);
        assert_eq!(0, info.syscall_times[SYSCALL_EXIT]);
        assert!(t4 - t1 <= info.time + 1);
        assert!(info.time < t5 - t1 + 100);
        assert!(info.status == TaskStatus::Running);

        println!("Test task info OK!");
        100
    } else {
        // parent process
        let mut exit_code: i32 = 0;
        println!("ready waiting on parent process!");
        assert_eq!(pid, wait(&mut exit_code));
        assert_eq!(exit_code, 100);
        println!("child process pid = {}, exit code = {}", pid, exit_code);
        0
    }
}

pub fn test_runner(_test: &[&dyn Fn()]) {
    loop {}
}