use core::{num, ptr};

use lazy_static::*;
use crate::{common::*, multitask::{switch::__switch, task::{TCB, TaskContext, TaskStatus}}, sync::UPSafeCell, trap::TrapContext, stack::{KERNEL_STACK, USER_STACK}};
use core::arch::asm;
use alloc::vec::Vec;
use loader::{get_app_data, get_num_app};
mod task;
mod sched;
mod switch;
mod loader;

pub struct TaskManager {
    task_num: usize,
    inner: UPSafeCell<TaskManagerInner>
}

pub struct TaskManagerInner {
    tcbs: Vec<TCB>,
    current_task_idx: usize
}

impl TaskManager {

    fn run_first_task(&self) {
        let mut inner = self.inner.exclusive_access();
        // 1. pick the first task
        inner.tcbs[0].status = TaskStatus::Running;
        // for i in 0..24 {
        //     println!("{:#x} {:#x} {:#x} {:#x} {:#x} {:#x}", 
        //         KERNEL_STACK[0].data[KERNEL_STACK[0].data.len()-(i * 6 + 6)],
        //         KERNEL_STACK[0].data[KERNEL_STACK[0].data.len()-(i * 6 + 5)],
        //         KERNEL_STACK[0].data[KERNEL_STACK[0].data.len()-(i * 6 + 4)],
        //         KERNEL_STACK[0].data[KERNEL_STACK[0].data.len()-(i * 6 + 3)],
        //         KERNEL_STACK[0].data[KERNEL_STACK[0].data.len()-(i * 6 + 2)],
        //         KERNEL_STACK[0].data[KERNEL_STACK[0].data.len()-(i * 6 + 1)],
        //     );
        // }
        let next_task_ctx_ptr = &inner.tcbs[0].ctx as *const TaskContext;
        drop(inner);

        // 2. make pesudo current context
        let mut _unused = TaskContext::empty();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_ctx_ptr);
        }

        panic!("Unreachable in run_first_task!")
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();

        for i in (inner.current_task_idx + 1)..(inner.current_task_idx + self.task_num + 1) {
            let idx = i % TASK_MANAGER.task_num;
            let tcb = &inner.tcbs[idx];
            if TaskStatus::Ready == tcb.status {
                return Some(idx)
            }
        }

        None
    }

    fn run_next_task(&self) {
        // 1. pick the next task
        if let Some(next_idx) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current_idx = inner.current_task_idx;

            // 2. save the context of current task
            let current_task_ctx_ptr = &mut inner.tcbs[current_idx].ctx as *mut TaskContext;
            let next_task_ctx_ptr = &inner.tcbs[next_idx].ctx as *const TaskContext;

            inner.tcbs[next_idx].status = TaskStatus::Running;
            inner.current_task_idx = next_idx;

            // println!("[kernel] run next {} curr {}", next_idx, current_idx);

            drop(inner);

            // 3. restore context of the next task
            // 4. ret
            unsafe {
                // we need handle step 3 and 4 in assembly code(__switch)
                __switch(current_task_ctx_ptr, next_task_ctx_ptr)
            }
            
        } else {
            println!("[kernel] all task has been completed!");
            loop {}
        }
    }

    fn mark_current_status(&self, status: TaskStatus) {
        // todo: state machine validation
        let mut inner = TASK_MANAGER.inner.exclusive_access();
        let curr_idx = inner.current_task_idx;
        inner.tcbs[curr_idx].status = status;
    }

    pub fn mark_current_ready_and_run_next(&self) {
        self.mark_current_status(TaskStatus::Ready);
        self.run_next_task();
    }

    pub fn mark_current_exit_and_run_next(&self) {
        self.mark_current_status(TaskStatus::Exited);
        self.run_next_task();
    }

    pub fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tcbs[inner.current_task_idx].memory_space.token()
    }
    
}

// load all task to memory and prepare thier TCB.
pub fn init() {
    println!("[kernel] ==============Task Loader=============");
    println!("[kernel] task manager loaded {} task.", TASK_MANAGER.task_num);
    println!("[kernel] ==============Task Loader=============");
}

lazy_static! {
    /// 在当前版本的TaskManager中，不需要再有复杂的加载代码
    /// 而是将标准的elf格式应用交给TCB处理，创建对应的MemorySpace

    pub static ref TASK_MANAGER: TaskManager =  { 

        unsafe extern "C" {
            fn _num_app();
        }

        let mut tasks: Vec<TCB> = Vec::new();

        let num_app = get_num_app();
        for i in 0..num_app {
            println!("[kernel] load task {}", i);
            tasks.push(TCB::new(get_app_data(i), i));
            println!("[kernel] task {} loaded", i);
        }

        let manager = TaskManager {
            task_num: 0,
            inner: unsafe {
                UPSafeCell::new(
                    TaskManagerInner { tcbs: tasks, current_task_idx: 0 }
                )
            }
        };
        
        //  make tasks initial context and status
        println!("[kernel] task manager initialized!");
        manager
    };
}

pub fn run_first_task() {
    println!("[kernel] run first task!");
    TASK_MANAGER.run_first_task();
}

pub fn suspend_and_run_next() {
    TASK_MANAGER.mark_current_ready_and_run_next();
}

pub fn exit_and_run_next() {
    TASK_MANAGER.mark_current_exit_and_run_next();
}

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}