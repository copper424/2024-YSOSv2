#![no_std]
#![no_main]

use core::alloc::Layout;
use lib::*;
extern crate lib;
fn main() -> isize {
    let layout = Layout::new::<[usize; 4096]>();
    println!("{:#?}", layout);
    let ptr = lib::sys_allocate(&layout);
    println!("{:0x?}", ptr);
    for idx in 0..5 {
        unsafe {
            ptr.add(idx).write(idx as u8);
        }
    }
    for idx in 0..5 {
        unsafe {
            println!("{}", ptr.add(idx).read());
        }
    }
    lib::sys_deallocate(ptr, &layout);
    0
}

entry!(main);
