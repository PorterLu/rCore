#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_time, yield_, get_task_info};
use user_lib::task::*;

#[no_mangle]
fn main() -> i32 {
    let current_timer = get_time();
    let wait_for = current_timer + 3000;
    while get_time() < wait_for {
        yield_();
    }
    
    for i in 0..4 {
        let task_info = &mut TaskInfo::new() as  *mut TaskInfo;
        get_task_info(i as usize, task_info);
        unsafe {
            println!("app_id:{}", (*task_info).id);
            println!("status:{:?}", (*task_info).status);
            println!("total time:{}", (*task_info).time);
            for j in 0..MAX_SYSCALL_NUM {
                if (*task_info).call[j].id != usize::MAX {
                    println!("syscall_id:{}, times:{}", (*task_info).call[j].id, (*task_info).call[j].times);
                }
            }
        }
    }
    println!("Test sleep OK!");
    0
}
