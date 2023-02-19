#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{getpid, spawn, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    println!("pid {}: Hello world from user mode program!", getpid());
    let ret = spawn("matrix\0");
    if ret == -1 {
        println!("fail!");
    } else {
        println!("pid: {} success!", ret);
    }
    let mut exit_code: i32 = Default::default();
    waitpid(ret as usize, &mut exit_code);
    0
}
