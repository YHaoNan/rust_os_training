use crate::stack::{KernelStack, UserStack};


#[derive(Copy, Clone)]
pub struct TCB {
    pub id: usize, // task unique id
    pub entry_ptr: usize, // pointing to it's entry point
    pub status: TaskStatus, 
    pub ctx: TaskContext
}

impl TCB {
    pub fn empty() -> Self {
        Self {
            id: 0,
            entry_ptr: 0,
            status: TaskStatus::UnInit,
            ctx: TaskContext {  }
        }
    }
}

#[derive(Clone, Copy)]
pub struct TaskContext {
    // 1. callee saved register
    // 2. 
}

#[derive(Clone, Copy)]
pub enum TaskStatus {
    UnInit,   // 任务状态，未初始化
    Ready,    // 任务状态，随时准备执行
    Running,  // 任务状态，运行中
    Exited,   // 任务状态，已退出
}
