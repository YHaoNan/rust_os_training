use core::ptr;

use lazy_static::*;
use crate::{common::*, loader::task::TCB};
use core::arch::asm;
mod task;

pub struct TaskManager {
    task_num: usize,
    tcbs: [TCB; APP_NUM],
    current_task_idx: usize
}

// loader加载所有app到正确的位置。构建它们的TCB。
pub fn init() {
    println!("[kernel] ==============Task Loader=============");
    println!("[kernel] task manager loaded {} task.", TASK_MANAGER.task_num);
    println!("[kernel] ==============Task Loader=============");
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        unsafe extern "C" {
            fn _num_app();
        }

        let mut tcbs: [TCB; APP_NUM] = [
            TCB::empty(); APP_NUM
        ];

        let mut manager = TaskManager {
            task_num: 0,
            tcbs: tcbs,
            current_task_idx: 0
        };

        unsafe {
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; APP_NUM + 1] = [0; APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);

            asm!("fence.i");

            for idx in 0..num_app {
                println!("[kernel] load task {}", idx);
                // prepare tcb
                manager.task_num = num_app;
                tcbs[idx].entry_ptr = app_start[idx];
                tcbs[idx].id = idx;
                tcbs[idx].status = task::TaskStatus::UnInit;
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

        println!("[kernel] task manager initialized!");
        manager
    };
}