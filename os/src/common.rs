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