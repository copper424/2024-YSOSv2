#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    for i in 0..10 {
        println!("Current process yield {} times", i);
        sys_sched_yield();
        sys_stat();
        println!("{}:Hello, world!!!", i);
    }
    let pid = sys_get_pid();
    pid as isize
}

entry!(main);
