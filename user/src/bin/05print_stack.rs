#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::stack_trace;

#[unsafe(no_mangle)]
fn main() -> i32 {
    print_stack_trace();
    0
}

fn print_stack_trace() {

    stack_trace();

}

