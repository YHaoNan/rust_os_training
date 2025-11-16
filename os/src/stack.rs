use crate::common::*;
use core;
use crate::trap::TrapContext;

#[derive(Clone, Copy)]
pub struct KernelStack {
    pub data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, trap_cx: TrapContext) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}

#[derive(Clone, Copy)]
pub struct UserStack {
    pub data: [u8; USER_STACK_SIZE],
}


impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub static USER_STACK: [UserStack; APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE]
}; APP_NUM];
pub static KERNEL_STACK: [KernelStack; APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE]
}; APP_NUM];