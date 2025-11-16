use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;



fn sys_call(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret, // 首个参数 -> x10，并且执行流返回后，x10的内容会被写回ret
            in("x11") args[1], // 第二个参数 -> x11
            in("x12") args[2], // 第三个参数 -> x12
            in("x17") id       // 系统调用id -> x17
        );
    }
    ret
}

pub fn sys_exit(code: i32) -> isize {
    sys_call(SYSCALL_EXIT, [code as usize, 0, 0])
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    sys_call(SYSCALL_WRITE, [fd, buf.as_ptr() as usize, buf.len()])
}

pub fn sys_yield() -> isize {
    sys_call(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_gettime() -> isize {
    sys_call(SYSCALL_GET_TIME, [0, 0, 0])
}