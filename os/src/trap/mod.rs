//! trap处理程序，可能来自用户程序的错误、或者系统调用等
//! trap处理程序首先要保存之前的现场，32个通用寄存器和sepc、sstauts
//! trap处理程序查看trap来源，做出相应反应

mod context;

use crate::multitask::{exit_and_run_next, suspend_and_run_next};
use crate::timer::set_next_trigger;
use core::arch::global_asm;
use crate::syscall::syscall;
use riscv::register::{
    scause,
    stval,
    sie,
};
use riscv::interrupt::{Trap};
use riscv::interrupt::supervisor::{Interrupt, Exception};
use riscv::register::stvec::{self, Stvec, TrapMode};


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
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
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

    cx
}


pub use context::TrapContext;