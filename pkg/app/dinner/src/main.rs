#![no_std]
#![no_main]

use lib::*;
extern crate lib;
static CHOPSTICK: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
fn main() -> isize {
    for i in 0..CHOPSTICK.len() {
        CHOPSTICK[i].init(1);
    }
    println!("Scheme 1");
    philosopher(0);
    println!("Scheme 2");
    philosopher(1);
    0
}

fn philosopher(choice: i32) {
    let mut pids = [0; 5];

    match choice {
        0 => {
            for i in 0..CHOPSTICK.len() {
                let pid = sys_fork();
                if pid == 0 {
                    philosopher1(i);
                    sys_exit(0);
                } else {
                    pids[i] = pid as u16;
                }
            }
        }
        1 => {
            let avail = Semaphore::new(5);
            avail.init(CHOPSTICK.len() - 1);
            for i in 0..CHOPSTICK.len() {
                let pid = sys_fork();
                if pid == 0 {
                    philosopher2(i, &avail);
                    sys_exit(0);
                } else {
                    pids[i] = pid as u16;
                }
            }
        }
        _ => unreachable!("Invalid choice"),
    }

    sys_stat();
    for i in 0..CHOPSTICK.len() {
        sys_wait_pid(pids[i]);
    }
}

fn philosopher1(id: usize) {
    let left = id;
    let right = (id + 1) % CHOPSTICK.len();
    if id & 0x1 == 0 {
        CHOPSTICK[left].wait();
        CHOPSTICK[right].wait();

        println!("Philosopher {} is eating", id);
        CHOPSTICK[right].signal();
        CHOPSTICK[left].signal();
    } else {
        CHOPSTICK[right].wait();
        CHOPSTICK[left].wait();

        println!("Philosopher {} is eating", id);
        CHOPSTICK[left].signal();
        CHOPSTICK[right].signal();
    }
}

fn philosopher2(id: usize, avail: &Semaphore) {
    avail.wait();
    let left = id;
    let right = (id + 1) % CHOPSTICK.len();
    CHOPSTICK[left].wait();
    CHOPSTICK[right].wait();
    println!("Philosopher {} is eating", id);
    CHOPSTICK[right].signal();
    CHOPSTICK[left].signal();
    avail.signal();
}
entry!(main);
