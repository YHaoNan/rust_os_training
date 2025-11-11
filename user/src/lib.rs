#![no_std]
#![feature(linkage)]
// #![feature(panic_info_message)]

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! { // system level entry
    clear_bss();
    exit(main()); // user level entry
    panic!("unreachable after sys_exit!");
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 { // this function provided by user program.
    panic!("Cannot find main!"); // default behavior.
}


use core::arch::asm;
use core::ptr;
fn clear_bss() {
    unsafe extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

use syscall::*;


pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}


// 获取当前堆栈情况，以从顶到下的顺序放置到result中
pub fn stack_trace() {

    let mut fp: *const usize;
    let mut curr = 0;

    unsafe {
        core::arch::asm!(
            "mv {}, fp",
            out(reg) fp
        );
    }

    println!("======START OF STACK======");

    while fp != ptr::null() {
        unsafe {

            let ra = *fp.sub(1);
            let saved_fp = *fp.sub(2);

            println!("Stack fp = {:016x}", ra);

            fp = saved_fp as *const usize;
            curr += 1;
        }
    }

    println!("======END OF STACK======");

}