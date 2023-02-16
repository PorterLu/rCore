#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod config;
mod lang_item;
mod loader;
mod sbi;
mod sync;
pub mod syscall;
mod timer;
pub mod trap;
pub mod task;

#[path = "boards/qemu.rs"]
mod board;

//use crate::sbi::shutdown;
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> !{
    clear_bss();
    println!("[kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    println!("apps loading over");
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    println!("start running");
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize ..ebss as usize).for_each(|x| {
        unsafe{ (x as *mut u8).write_volatile(0) }
    });
}
