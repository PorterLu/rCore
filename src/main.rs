#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;
mod lang_item;
mod sbi;

use crate::sbi::shutdown;
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> !{
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
    }

    clear_bss();
    println!("Hello OS");
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(".boot_stack  top=bottom={:#x}, lower_bound={:#x}", 
            boot_stack_top as usize, boot_stack_lower_bound as usize);

    shutdown();
    //loop{}
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
