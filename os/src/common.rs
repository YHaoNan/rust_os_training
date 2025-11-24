// 用户栈大小
pub const USER_STACK_SIZE: usize = 4096 * 2;

// 内核栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;

// 用户程序数量
pub const APP_NUM: usize = 16;

// 用户程序起始点(第一个用户程序被加载的位置)
pub const APP_ENTRY_POINT: usize = 0x80400000;

// 用户程序大小
pub const APP_SIZE: usize = 0x20000;

// #[cfg(feature = "board_k210")]
// pub const CLOCK_FREQ: usize = 403000000 / 62;

pub const CLOCK_FREQ: usize = 12500000;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];

// 内核堆大小
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; 

// 内存末尾
pub const MEMORY_END: usize = 0x80800000;

// 页面大小占用的位（4k=12位）
pub const PAGE_SIZE_BITS: usize = 0xC;

// 页面大小（4k）
pub const PAGE_SIZE: usize = 0x1000;

// 跳板页起始地址（最后一页起始地址）
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

// Trap上下文
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;


/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
