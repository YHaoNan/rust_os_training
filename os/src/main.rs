#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

use core::arch::global_asm;

use crate::stack::{KERNEL_STACK, USER_STACK};

#[cfg(feature = "board_qemu")]
#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;
mod stack;
mod common;
mod multitask;
mod timer;
mod memory;
extern crate alloc;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

fn print_stack_info() {
    println!("[kernel] ==============Stack Info=============");
    println!("[kernel] User stack num: {}, Kernel stack num: {}", USER_STACK.len(), KERNEL_STACK.len());
    println!("[kernel] ==============Stack Info=============");
}

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");

    print_stack_info();
    memory::init();
    multitask::init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    multitask::run_first_task();

    loop {}
}
