#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    
    sys_list_app();
    sys_stat();
    // sys_write(1, "Hello, world!!!\n".as_bytes());
    println!("Hello, world!!!");
    // panic!("panic");
    let pid = sys_get_pid();
    pid as isize
}

entry!(main);
