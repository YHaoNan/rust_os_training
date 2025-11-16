use core::ptr;

use lazy_static::*;
use crate::{common::*, multitask::{switch::__switch, task::{TCB, TaskContext, TaskStatus}}, sync::UPSafeCell, trap::TrapContext, stack::{KERNEL_STACK, USER_STACK}};
use core::arch::asm;
mod task;
mod sched;
mod switch;

pub struct TaskManager {
    task_num: usize,
    inner: UPSafeCell<TaskManagerInner>
}

pub struct TaskManagerInner {
    tcbs: [TCB; APP_NUM],
    current_task_idx: usize
}

impl TaskManager {

    fn run_first_task(&self) {
        let mut inner = TASK_MANAGER.inner.exclusive_access();
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
        let inner = TASK_MANAGER.inner.exclusive_access();

        for i in (inner.current_task_idx)..(inner.current_task_idx + TASK_MANAGER.task_num) {
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
            let mut inner = TASK_MANAGER.inner.exclusive_access();
            let current_idx = inner.current_task_idx;

            // 2. save the context of current task
            let current_task_ctx_ptr = &mut inner.tcbs[current_idx].ctx as *mut TaskContext;
            let next_task_ctx_ptr = &inner.tcbs[next_idx].ctx as *const TaskContext;

            inner.tcbs[current_idx].status = TaskStatus::Ready;
            inner.tcbs[next_idx].status = TaskStatus::Running;

            drop(inner);

            // 3. restore context of the next task
            // 4. ret
            unsafe {
                // we need handle step 3 and 4 in assembly code(__switch)
                __switch(current_task_ctx_ptr, next_task_ctx_ptr)
            }
            
        } else {
            panic!("[kernel] all task has been completed!");
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
    
}

// load all task to memory and prepare thier TCB.
pub fn init() {
    println!("[kernel] ==============Task Loader=============");
    println!("[kernel] task manager loaded {} task.", TASK_MANAGER.task_num);
    println!("[kernel] ==============Task Loader=============");
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager =  { 
        unsafe extern "C" {
            fn _num_app();
        }

        let mut tcbs: [TCB; APP_NUM] = [
            TCB::empty(); APP_NUM
        ];

        let mut manager = TaskManager {
            task_num: 0,
            inner: unsafe {
                UPSafeCell::new(
                    TaskManagerInner { tcbs: tcbs, current_task_idx: 0 }
                )
            }
        };

        // load tasks
        unsafe {
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; APP_NUM + 1] = [0; APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);

            asm!("fence.i");

            let mut inner = manager.inner.exclusive_access();
            for idx in 0..num_app {
                println!("[kernel] load task {}", idx);
                // prepare tcb
                manager.task_num = num_app;
                inner.tcbs[idx].entry_ptr = app_start[idx];
                inner.tcbs[idx].id = idx;
                inner.tcbs[idx].status = TaskStatus::Ready;
                // make initial context
                //  1. ra pointing to kernel __restore function
                //  2. all register is zero
                //  3. sp pointing to it's kernel stack
                //  4. kernel stack return to user stack and set pc to user program address
                inner.tcbs[idx].ctx = TaskContext::goto_restore(init_app_cx(idx));
                println!("[kernel] task {} tcb is ready", idx);

                println!("[kernel] loading task {} from {:#x} to {:#x}", idx, app_start[idx], APP_ENTRY_POINT + idx * APP_SIZE);

                // load task code
                let task_start_src = app_start[idx] as *const u8;
                let task_start_dst = (APP_ENTRY_POINT + idx * APP_SIZE) as *mut u8;
                let task_len = APP_SIZE;

                ptr::copy(task_start_src, task_start_dst, task_len);
                println!("[kernel] task {} loaded from {:#x} to {:#x}", idx, app_start[idx], APP_ENTRY_POINT + idx * APP_SIZE);

            }
        }

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

/// get app info with entry and sp and save `TrapContext` in kernel stack
pub fn init_app_cx(app_id: usize) -> usize {
    let task_start_dst = APP_ENTRY_POINT + app_id * APP_SIZE;
    println!("[kernel] init_app_cx. entry {:#x}", task_start_dst);
    let result = KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        task_start_dst,
        USER_STACK[app_id].get_sp(),
    ));
    // for i in 0..24 {
    //     println!("{:#x} {:#x} {:#x} {:#x} {:#x} {:#x}", 
    //         KERNEL_STACK[app_id].data[KERNEL_STACK[app_id].data.len()-(i * 6 + 6)],
    //         KERNEL_STACK[app_id].data[KERNEL_STACK[app_id].data.len()-(i * 6 + 5)],
    //         KERNEL_STACK[app_id].data[KERNEL_STACK[app_id].data.len()-(i * 6 + 4)],
    //         KERNEL_STACK[app_id].data[KERNEL_STACK[app_id].data.len()-(i * 6 + 3)],
    //         KERNEL_STACK[app_id].data[KERNEL_STACK[app_id].data.len()-(i * 6 + 2)],
    //         KERNEL_STACK[app_id].data[KERNEL_STACK[app_id].data.len()-(i * 6 + 1)],
    //     );
    // }
    result
}
