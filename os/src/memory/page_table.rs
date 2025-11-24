
use crate::common::{PAGE_SIZE};
use crate::memory::address::{PhysAddr, StepByOne, VirtAddr};

use super::address::{PhysPageNum, VirtPageNum};
use super::frame_allocator::{frame_alloc, FrameTracker};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;

bitflags! {

    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PTE {
    pub bits: usize
}

impl PTE {

    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }
    
    pub fn writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }

}

pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    /// 新建一个页表
    /// 1. 分配根物理页
    /// 2. 记录到frames
    pub fn new() -> Self {
        // 分配失败时直接crash
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame] // 使FrameTracker跟随页表的生命周期，当页表drop，自动释放所有物理页帧
        }
    }

    /// 查找页表，通过VPN找到PTE
    /// 若过程中遇到未映射的页表项会自动创建（最后一级页表项不会自动创建）
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PTE> {
        let idx = vpn.indexes();
        let mut ppn: PhysPageNum = self.root_ppn;
        for (i, id) in idx.iter().enumerate() {

            let pte = &mut (get_pte_array(&ppn)[*id]);

            if i == 2 { // 最后一级，命中，返回
                return Some(pte);
            }

            if !pte.is_valid() { // 未映射，创建
                let frame = frame_alloc().unwrap();
                *pte = PTE::new(ppn, PTEFlags::V);
                self.frames.push(frame);
            }

            // 下探下一级页表
            ppn = pte.ppn();
        }
        None
    }

    /// 查找页表，通过VPN找到PTE
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PTE> {
        // 1. 从vpn中找到三级页表中每一级的索引
        let idx = vpn.indexes();

        // 2. 依次访问三级页表
        let mut ppn: PhysPageNum = self.root_ppn;
        for (i, id) in idx.iter().enumerate() {
            let pte = &mut (get_pte_array(&ppn)[*id]);
            if i == 2 { // 最后一级，命中，返回
                return Some(pte);
            }

            if !pte.is_valid() { // 未映射，返回
                return None;
            }

            // 下探下一级页表
            ppn = pte.ppn();
        }
        None
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is already mapped", vpn);
        *pte = PTE::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is not mapped", vpn);
        *pte = PTE::empty();
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PTE> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    /// Temporarily used to get arguments from user space.
    /// 拿satp的内容，返回临时页表？？
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        0b1000usize << 60 | self.root_ppn.0
    }

}

pub fn get_pte_array(ppn: &PhysPageNum) -> &'static mut [PTE] {
    let pa: PhysAddr = (*ppn).into(); // 获取根页表的物理地址
    let pte_array: &mut [PTE] = unsafe {
        // core::slice::from_raw_parts_mut(pa.0 as *mut PTE, 512)
        core::slice::from_raw_parts_mut(pa.0 as *mut PTE, PAGE_SIZE / core::mem::size_of::<PTE>()) // 512 pte for 1 page
    };
    return pte_array;
}

pub fn get_bytes_array(ppn: &mut PhysPageNum) -> &'static mut [u8] {
    let pa: PhysAddr = (*ppn).into();
    unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
}

pub fn translate_byte_buffer(
    token: usize, // 页表根
    ptr: *const u8, // 页表中的buffer VA
    len: usize      // buffer长度
) -> Vec<&'static [u8]>{
    let page_table = PageTable::from_token(token);

    let mut start = ptr as usize;
    let end = start + len;

    let mut v = Vec::new();

    while start < end {

        let start_va = VirtAddr::from(start); // 转虚拟地址

        let mut start_vpn: VirtPageNum = start_va.floor_page_num(); // 转虚拟页号

        let mut start_ppn = page_table.translate(start_vpn) // translate，获取物理页号
            .unwrap()
            .ppn();

        start_vpn.step();
        let mut end_va: VirtAddr = start_vpn.into();
        end_va = end_va.min(VirtAddr::from(end)); // 取本页和end中最小的

        if end_va.aligned() {
            v.push(&get_bytes_array(&mut start_ppn)[start_va.page_offset()..]);
        } else {
            v.push(&get_bytes_array(&mut start_ppn)[start_va.page_offset()..end_va.page_offset()]);
        }

        start = end_va.0;
    }

    v

}