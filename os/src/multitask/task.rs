use crate::memory::memory_set::{KERNEL_SPACE, MapPermission, MemorySpace};
use crate::memory::address::{PhysPageNum, VirtAddr};
use crate::common::{TRAP_CONTEXT, kernel_stack_position};
use crate::trap::{trap_return, TrapContext, trap_handler};

pub struct TCB {
    pub status: TaskStatus, 
    pub ctx: TaskContext,
    pub memory_space: MemorySpace,
    pub trap_cx_ppn: PhysPageNum, 
    pub base_size: usize,
}

impl TCB {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {

        // 1. 从elf构建任务的内存空间
        println!("[kernel] create memory space for task {}", app_id);
        let (memory_space, user_sp, entry_point) = MemorySpace::from_elf(elf_data);

        let trap_cx_ppn = memory_space // 获取trap_cx的物理页号
                .translate(VirtAddr::from(TRAP_CONTEXT).into())
                .unwrap()
                .ppn();

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id); // 计算内核栈位置
        KERNEL_SPACE.exclusive_access() // 向地址空间中插入内核栈
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(), 
                MapPermission::R | MapPermission::W
            );

        // 2. 创建任务上下文
        let task_status = TaskStatus::Ready;

        let tcb = Self {
            status: task_status, 
            ctx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_space,
            trap_cx_ppn,
            base_size: user_sp,
        };

        // 3. 初始化用户空间中的trap_context
        //    在当前设计中TrapContext不再存储在内核栈中，而是存储在用户空间的一个页中
        let trap_cx = tcb.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point, // 首次从trap返回时返回到应用程序入口
            user_sp,     // 用户栈地址
            // 下面三个字段是进入trap时的辅助字段，帮助asm代码切换到内核空间和内核栈，它们只会在第一次创建任务的TrapContext时被填充，不会修改
            KERNEL_SPACE.exclusive_access().token(),  // 内核页表
            kernel_stack_top,  // 内核栈地址
            trap_handler as usize // trap_handler地址
        );

        // 映射
        tcb


    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
}

/// TaskContext用于任务切换时返回到用户程序
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize, // function call return address
    pub sp: usize, // user stack pointer
    pub s: [usize; 12], // callee saved registers
}

impl TaskContext {
    pub fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12]
        }
    }
    
    /// 该方法在用户任务初始化时被调用，用于进入任务的kernel stack，并通过trap_return返回至用户空间
    pub fn goto_trap_return(kernel_stack_top: usize) -> Self {
        Self {
            ra: trap_return as usize, 
            sp: kernel_stack_top,
            s: [0; 12], // set reg files to 0
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
