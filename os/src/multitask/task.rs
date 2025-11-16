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
            ctx: TaskContext::empty()
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

impl TaskContext {
    pub fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12]
        }
    }
    
    /// set task context {__restore ASM funciton, kernel stack, s_0..12 }
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        unsafe extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize, // set ra to __restore
            sp: kstack_ptr,         // set sp to kstack_ptr
            s: [0; 12],             // set reg files to 0
        }
        // when `__switch` was called.
        // 1. restore reg files
        // 2. change stack to task kernel stack
        // 3. run `__restore`(and it'll be trap into user stack and run user code)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit,   // 任务状态，未初始化
    Ready,    // 任务状态，随时准备执行
    Running,  // 任务状态，运行中
    Exited,   // 任务状态，已退出
}
