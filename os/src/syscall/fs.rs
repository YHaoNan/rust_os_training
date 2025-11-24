use crate::memory::page_table::{translate_byte_buffer};
use crate::multitask::current_user_token;


const FD_STDOUT: usize = 1;

/// sys_write需要改写，buf是一个用户空间的地址，而非内核空间地址
/// 
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translate_byte_buffer(current_user_token(), buf, len);

            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            
            len as isize
        },

        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}