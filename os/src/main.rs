#![no_std]
#![no_main]

use core::arch::global_asm;

#[cfg(feature = "board_qemu")]
#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
pub mod batch;
mod lang_items;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;

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

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    trap::init();
    batch::init();
    batch::run_next_app();
}

use core::arch::asm;
use core::ptr;

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