use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, get_task_info};
use crate::timer::get_time_ms;
use crate::task::task::{TaskInfo};

pub fn sys_exit(exit_code: i32) -> !{
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_task_info(id: usize, task_info: *mut TaskInfo) -> isize {
    get_task_info(id, task_info)
}