use crate::common::*;

#[derive(Clone, Copy)]
pub struct KernelStack {
    pub data: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {

}

#[derive(Clone, Copy)]
pub struct UserStack {
    pub data: [u8; USER_STACK_SIZE],
}

impl UserStack {
}

pub static USER_STACK: [UserStack; APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE]
}; APP_NUM];
pub static KERNEL_STACK: [KernelStack; APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE]
}; APP_NUM];