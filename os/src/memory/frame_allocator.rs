use super::address::{PhysPageNum, PhysAddr};
use alloc::vec::Vec;
use crate::common::{MEMORY_END};
use crate::sync::UPSafeCell;
use lazy_static::*;

/// 物理页帧跟踪器
/// 用于封装一个已经分配出去的物理页帧
/// 该类会自动在资源释放时将物理页帧归还给FrameAllocator
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

// 物理页帧分配器
// 给定物理内存范围，将其划分成多个物理页帧，并管理和分配
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, page_num: PhysPageNum);
}

/// StackFrameAllocator描述了一个基于栈的物理页帧分配器
/// [current, end) 是当前尚未被分配出去的物理页号区间
/// recycled 是已经被回收的物理页号列表
/// 先复用recycled中的，如果recycled为空，才从[current, end)中分配
struct StackFrameAllocator {
    _start: usize,
    _end: usize,
    current: usize,
    end: usize,
    recycled: Vec<PhysPageNum>,
}

impl StackFrameAllocator {
    fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current = start.0;
        self.end = end.0;
        self._start = start.0;
        self._end = end.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            _start: 0,
            _end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(page_num) = self.recycled.pop() {
            Some(page_num)
        } else if self.current < self.end {
            let page_num = PhysPageNum(self.current);
            self.current += 1;
            Some(page_num)
        } else {
            None
        }
    }

    fn dealloc(&mut self, page_num: PhysPageNum) {
        let ppn = page_num.0;
        if ppn < self._start || ppn >= self._end {
            panic!("dealloc: invalid page number");
        }
        if ppn >= self.current {
            panic!("dealloc: page number not allocated");
        }
        if self.recycled.iter().any(|&p| p.0 == ppn) {
            panic!("dealloc: page number already deallocated");
        }
        
        self.recycled.push(page_num);
    }

}


type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = 
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator() {

    unsafe extern "C" {
        fn ekernel();
    }

    // 从内核结束地址 ~ 内存结束地址，都是帧分配器的分配范围
    // 注意需要按页面对齐
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil_page_num(),
        PhysAddr::from(MEMORY_END).floor_page_num(),
    )
}

pub fn frame_alloc() -> Option<FrameTracker> {
    let ppn = FRAME_ALLOCATOR.exclusive_access().alloc()?;
    Some(FrameTracker { ppn })
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

