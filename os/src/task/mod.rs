//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use lazy_static::*;
pub use task::{TaskControlBlock, TaskStatus};
pub use context::TaskContext;
pub use crate::timer::{get_time_ms, get_time_us};

pub static mut SWITCH_TASK_START: usize = 0;

pub unsafe fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext) {
    SWITCH_TASK_START = get_time_us();
    switch::__switch(current_task_cx_ptr, next_task_cx_ptr);
    // 记录除了第一次运行外的 switch cost
    crate::task::update_switch_cost(get_time_us() - SWITCH_TASK_START);
}

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
    /// the number of tasks that have not exit
    alive_task_num: usize,
    /// record time point
    checkpoint: usize,
}

/// ch3-pro2
impl TaskManagerInner {
    /// update checkpoint and return the diff time
    fn update_checkpoint(&mut self) -> usize {
        let prev_point = self.checkpoint;
        self.checkpoint = get_time_ms();
        return self.checkpoint - prev_point;
    }
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
            syscall_times: [0; MAX_SYSCALL_NUM],
            switch_time: 0,
            user_time: 0,
            kernel_time: 0,
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            task.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    alive_task_num: num_app,
                    checkpoint: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        inner.update_checkpoint();
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // ch3-pro1
        // if inner.alive_task_num > 1 {
            // println!("[kernel] task {} suspended", current);
        // }
        // ch3-pro2
        inner.tasks[current].kernel_time += inner.update_checkpoint();
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // ch3-pro1, 2
        inner.tasks[current].kernel_time += inner.update_checkpoint();
        println!("[kernel] task {} exited, total cost in kernel/user {}/{} ms, context switch cost {} us",
            current, inner.tasks[current].kernel_time, inner.tasks[current].user_time, inner.tasks[current].switch_time);
        inner.tasks[current].task_status = TaskStatus::Exited;
        inner.alive_task_num -= 1;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            // ch3-pro1
            if current != next {
                // println!("[kernel] task switch from {} to {}", current, next);
                unsafe {
                    __switch(current_task_cx_ptr, next_task_cx_ptr);
                }
            }
            // go back to user mode
        } else {
            println!("All applications completed!");
            shutdown(false);
        }
    }

    // ch3-pro2 start
    /// record the kernel time, now start to record the user time
    pub fn user_time_start(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].kernel_time += inner.update_checkpoint();
    }

    /// record the user time, now start to record the kernel time
    pub fn user_time_end(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].user_time += inner.update_checkpoint();
    }
    // ch3-pro2 end
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

// ch3-pro2
pub fn user_time_start() {
    TASK_MANAGER.user_time_start();
}

pub fn user_time_end() {
    TASK_MANAGER.user_time_end();
}

// lab3
pub fn get_current_task_block() -> TaskControlBlock {
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current]
}

pub fn update_task_syscall_times(syscall_id: usize) {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].syscall_times[syscall_id] += 1;
}

pub fn update_switch_cost(cost: usize) {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].switch_time += cost;
}

pub fn get_current_task() -> usize {
    let inner = TASK_MANAGER.inner.exclusive_access();
    inner.current_task
}