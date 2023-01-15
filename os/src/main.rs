#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;
pub mod batch;
mod lang_item;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;

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
    batch::init();
    batch::run_next_app();
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
