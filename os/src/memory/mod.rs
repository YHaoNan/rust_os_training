use crate::memory::memory_set::KERNEL_SPACE;

pub mod heap_allocator;
pub mod address;
mod frame_allocator;
pub mod page_table;
pub mod memory_set;

pub fn init() {
    // 初始化内核堆
    heap_allocator::init_heap();
    // 初始化物理页帧分配器
    frame_allocator::init_frame_allocator();
    // 激活内核地址空间（因为都是恒等映射，所以激活后没任何影响）
    KERNEL_SPACE.exclusive_access().activate();
}