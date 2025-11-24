//! trap处理程序，可能来自用户程序的错误、或者系统调用等
//! trap处理程序首先要保存之前的现场，32个通用寄存器和sepc、sstauts
//! trap处理程序查看trap来源，做出相应反应

mod context;

use crate::common::{TRAMPOLINE, TRAP_CONTEXT};
use crate::multitask::{exit_and_run_next, suspend_and_run_next, current_user_token};
use crate::timer::set_next_trigger;
use core::arch::{global_asm, asm};
use crate::syscall::syscall;
use riscv::register::{
    scause,
    stval,
    sie,
    stvec,
};
use riscv::interrupt::{Trap};
use riscv::interrupt::supervisor::{Interrupt, Exception};
use riscv::register::stvec::{Stvec, TrapMode};


global_asm!(include_str!("trap.S"));


pub fn init() {
    println!("[kernel] ==============Trap Init=============");
    unsafe extern "C" {
        fn __alltraps();
    }

    let val = Stvec::new(__alltraps as usize, TrapMode::Direct);
    unsafe {
        stvec::write(val); // 将__alltraps写入中断向量表
    }
    println!("[kernel] ==============Trap Init=============");
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
    println!("[kernel] ==============Timer Interrupt Enabled=============");
} 

#[unsafe(no_mangle)]
/// handle an interrupt, exception, or system call from user space
pub fn trap_handler(cx: &mut TrapContext) -> ! {
    set_kernel_trap_entry(); // 设置 S -> S时的trap入口
    let scause_reg = scause::read();
    let cause_raw = scause_reg.cause();
    let stval = stval::read();
        // 转换成目标特定的 Trap<Exception, Interrupt>
    let cause: Trap<Interrupt, Exception> = cause_raw.try_into().unwrap_or_else(|_| {
        panic!("Unknown trap cause {:?}", cause_raw);
    });

    match cause {
        Trap::Exception(e) => match e {
            Exception::UserEnvCall => {
                cx.sepc += 4;
                cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            }
            Exception::StoreFault | Exception::StorePageFault => {
                println!("[kernel] PageFault in application, kernel killed it.");
                exit_and_run_next();
            }
            Exception::IllegalInstruction => {
                println!("[kernel] IllegalInstruction in application, kernel killed it.");
                exit_and_run_next();
            }
            Exception::InstructionFault => {
                println!("[kernel] InstructionFault in application, kernel killed it.");
                exit_and_run_next();
            }
            _ => {
                panic!(
                    "Unsupported trap {:?}, stval = {:#x}!",
                    scause_reg.code(),
                    stval
                );
            }
        }, 
        Trap::Interrupt(i) => match i {
            Interrupt::SupervisorTimer => {
                // println!("[kernel] timer interrupt");
                set_next_trigger();
                suspend_and_run_next();
            },
            _ => {
                panic!(
                    "Unsupported interrupt {:?}, stval = {:#x}!",
                    scause_reg.code(),
                    stval
                );
            }
        }
        _ => {
            panic!(
                "Unsupported scause {:?}, stval = {:#x}!",
                scause_reg.code(),
                stval
            );
        }
    }

    trap_return()
}


pub use context::TrapContext;

#[unsafe(no_mangle)]
/// 内核处理完一次陷入后，要返回到用户任务的入口函数
pub fn trap_return() -> ! {
    set_user_trap_entry(); // 设置U -> S时 trap 入口为TRAMPOLINE（注意，一旦使用分页，trap入口必须是源态地址空间中的，如U -> S时，必须是U的）
    let trap_cx_ptr = TRAP_CONTEXT; // 每一个应用固定的TrapContext地址
    let user_satp = current_user_token(); // 当前用户程序的根页表地址

    unsafe extern "C" {
        fn __alltraps();
        fn __restore();
    }


    // restore在trampoline中的相对偏移量
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;

    // 调用restore，将用户的trap_context和user_satp传入
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
fn trap_from_kernel() {
    panic!("trap from kernel! (S->S)")
}

fn set_user_trap_entry() {
    unsafe {
        let val = Stvec::new(TRAMPOLINE as usize, TrapMode::Direct);
        stvec::write(val);
    }
}

fn set_kernel_trap_entry() {
    unsafe {
        let val = Stvec::new(trap_from_kernel as usize, TrapMode::Direct);
        stvec::write(val);
    }
}