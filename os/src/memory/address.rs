use core::fmt::{self, Debug, Formatter};
use crate::common::{PAGE_SIZE_BITS, PAGE_SIZE};


// SV39物理地址宽度
const PA_WIDTH_SV39: usize = 56;
// SV39虚拟地址宽度
const VA_WIDTH_SV39: usize = 39;

// SV39物理页号宽度
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
// SV39虚拟页号宽度
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;



// 物理地址
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

// 物理页号(4k对齐)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

// 虚拟地址
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

// 虚拟页号(4k对齐)
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "PA: ({:#x})", self.0)
    }
}

impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "PPN: ({:#x})", self.0)
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "VA: ({:#x})", self.0)
    }
}

impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "VPN: ({:#x})", self.0)
    }
}

impl From<usize> for PhysAddr {
    // 只保留低56位
    fn from(addr: usize) -> Self {
        Self(addr & ((1 << PA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for PhysPageNum {
    // 只保留低44位
    fn from(addr: usize) -> Self {
        Self(addr & ((1 << PPN_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtAddr {
    // 只保留低39位
    fn from(addr: usize) -> Self {
        Self(addr & ((1 << VA_WIDTH_SV39) - 1))
    }
}

impl From<usize> for VirtPageNum {
    // 只保留低28位
    fn from(addr: usize) -> Self {
        Self(addr & ((1 << VPN_WIDTH_SV39) - 1))
    }
}

impl From<PhysAddr> for usize {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl From<PhysPageNum> for usize {
    fn from(addr: PhysPageNum) -> Self {
        addr.0
    }
}

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> Self {
        addr.0
    }
}

impl From<VirtPageNum> for usize {
    fn from(addr: VirtPageNum) -> Self {
        addr.0
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(addr: PhysAddr) -> Self {
        assert_eq!(addr.page_offset(), 0);
        addr.floor_page_num()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(addr: PhysPageNum) -> Self {
        Self(addr.0 << PAGE_SIZE_BITS)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(addr: VirtAddr) -> Self {
        addr.floor_page_num()
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(addr: VirtPageNum) -> Self {
        Self(addr.0 << PAGE_SIZE_BITS)
    }
}

impl PhysAddr {
    pub fn floor_page_num(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil_page_num(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

}

impl VirtAddr {
    pub fn floor_page_num(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }

    pub fn ceil_page_num(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }

    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }

}

impl VirtPageNum {

    pub fn indexes(&self) -> [usize; 3] {
        let vpn = self.0;
        [
            (vpn >> 9 >> 9) & 0b111111111,
            (vpn >> 9) & 0b111111111,
            (vpn) & 0b111111111,
        ]
    }

}

impl PhysPageNum {
    
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        unsafe { (pa.0 as *mut T).as_mut().unwrap() }
    }

}

pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Clone, Copy)]
pub struct SimpleRange<T> 
where T: StepByOne + Copy + PartialOrd + PartialEq + Debug {
    l: T,
    r: T,
}

impl<T> SimpleRange<T> 
where T: StepByOne + Copy + PartialOrd + PartialEq + Debug {

    pub fn new(start: T, end: T) -> Self {
        Self { l: start, r: end }
    }

    pub fn get_start(&self) -> T {
        self.l
    }

    pub fn get_end(&self) -> T {
        self.r
    }

    pub fn contains(&self, x: T) -> bool {
        self.l <= x && x < self.r
    }
}

impl<T> IntoIterator for SimpleRange<T> 
where T: StepByOne + Copy + PartialOrd + PartialEq + Debug {
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator {
            current: self.l,
            end: self.r,
        }
    }
}

pub struct SimpleRangeIterator<T> 
where T: StepByOne + Copy + PartialOrd + PartialEq + Debug {
    current: T,
    end: T,
}

impl<T> Iterator for SimpleRangeIterator<T> 
where T: StepByOne + Copy + PartialOrd + PartialEq + Debug {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }
        let ret = self.current;
        self.current.step();
        Some(ret)
    }
}

pub type VPNRange = SimpleRange<VirtPageNum>;
