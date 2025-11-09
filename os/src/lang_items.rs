//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
        
        print_backtrace();
        
    } else {
        println!("[kernel] Panicked: {}", info.message());
    }
    shutdown()
}

#[inline(never)]
fn print_backtrace() {
    println!("Stack backtrace:");
    
    let mut frame_pointer: usize;
    unsafe {
        core::arch::asm!("mv {}, s0", out(reg) frame_pointer); // s0 通常用作帧指针
    }
    
    let mut depth = 0;
    const MAX_DEPTH: usize = 50;
    
    while frame_pointer != 0 && depth < MAX_DEPTH {
        // 在 RISC-V 调用约定中，返回地址通常保存在帧指针-8的位置
        let return_addr = unsafe { *(frame_pointer as *const usize).sub(1) };
        
        if return_addr == 0 {
            break;
        }
        
        println!("  #{:02}: {:#016x}", depth, return_addr);
        
        // 下一个帧指针保存在当前帧指针的位置
        frame_pointer = unsafe { *(frame_pointer as *const usize) };
        depth += 1;
    }
    
    if depth == MAX_DEPTH {
        println!("  (backtrace truncated)");
    }
}