use crate::multitask::{exit_and_run_next, suspend_and_run_next};
use crate::timer::get_time_ms;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_and_run_next();
    panic!("Unreachable in sys_exit!")
}

pub fn sys_yield() -> isize {
    suspend_and_run_next();
    0
}

pub fn sys_gettime() -> isize {
    get_time_ms() as isize
}