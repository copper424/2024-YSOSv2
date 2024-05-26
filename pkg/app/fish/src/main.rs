#![no_std]
#![no_main]

use lib::*;

extern crate lib;

static ASSISTOR: Semaphore = Semaphore::new(3);
static SEM: [Semaphore; 3] = semaphore_array![0, 1, 2];
fn main() -> isize {
    let mut pids = [0; 3];
    for i in 0..3 {
        SEM[i].init(0);
    }
    ASSISTOR.init(1);
    for i in 0..3 {
        let pid = sys_fork();
        if pid == 0 {
            print_child(i);
            return 0;
        } else {
            pids[i] = pid;
        }
    }
    SEM[0].signal();
    for i in 0..3 {
        sys_wait_pid(pids[i]);
    }
    0
}

fn print_child(id: usize) {
    let mut idx = 0;
    let length = if id != 0 { 5 } else { 10 };
    while idx < length {
        match id {
            0 => {
                SEM[0].wait();
                ASSISTOR.wait();

                print!(">");
                sleep(500);

                SEM[1].signal();

                SEM[0].wait();

                print!(">");
                sleep(500);

                ASSISTOR.wait();
                SEM[2].signal();
                idx += 1;
            }
            1 => {
                SEM[1].wait();

                print!("<");
                sleep(500);

                SEM[0].signal();
                ASSISTOR.signal();
            }
            2 => {
                SEM[2].wait();

                print!("_");
                sleep(500);

                ASSISTOR.signal();
                SEM[0].signal();
            }
            _ => unreachable!(),
        }
        idx += 1;
        // SEM[id].signal();
    }
}

entry!(main);
