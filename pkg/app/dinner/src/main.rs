#![no_std]
#![no_main]

use lib::*;
extern crate lib;
static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
fn main() -> isize {
    for i in 0..CHOPSTICK.len() {
        CHOPSTICK[i].init(1);
    }
    let mut pids = [0; 5];
    for i in 0..CHOPSTICK.len() {
        let pid = sys_fork();
        if pid == 0 {
            philosopher(i, &CHOPSTICK);
            sys_exit(0);
        } else {
            pids[i] = pid as u16;
        }
    }
    sys_stat();
    for i in 0..CHOPSTICK.len() {
        sys_wait_pid(pids[i]);
    }
    0
}

fn philosopher(id: usize, chopsticks: &[Semaphore; 5]) {
    let left = id;
    let right = (id + 1) % 5;
    if id & 0x1 == 0 {
        chopsticks[left].wait();
        chopsticks[right].wait();

        println!("Philosopher {} is eating", id);
        chopsticks[right].signal();
        chopsticks[left].signal();
    } else {
        chopsticks[right].wait();
        chopsticks[left].wait();

        println!("Philosopher {} is eating", id);
        chopsticks[left].signal();
        chopsticks[right].signal();
    }
}

entry!(main);
