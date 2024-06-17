#![no_std]
#![no_main]
extern crate lib;

use lib::*;

fn main() -> isize {
    lib::println!("Priority Test");
    sys_set_priority(0, 11);
    lib::println!("Current priority: {}", sys_get_priority(0));
    let child = sys_spawn("hello", 12);
    println!("Child PID: {}", child);
    sys_stat();
    0
}

entry!(main);
