#![no_std]
#![no_main]

use core::ptr::addr_of_mut;

use lib::{collections::VecDeque, sync::Semaphore, *};

extern crate lib;
const THREAD_COUNT: usize = 16;
fn main() -> isize {
    let mut pids = [0; THREAD_COUNT];
    let key1 = 1;
    sys_new_sem(key1, 1);
    static mut QUEUE: VecDeque<Message> = VecDeque::new();
    static MUTEX: Semaphore = Semaphore::new(1);
    static AVAIL: Semaphore = Semaphore::new(2);
    static EMPTY: Semaphore = Semaphore::new(3);
    MUTEX.init(1);
    AVAIL.init(0);
    EMPTY.init(8);
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            let pid = sys_get_pid();
            if i % 2 == 0 {
                unsafe {
                    producer(pid, &MUTEX, &AVAIL, &EMPTY, addr_of_mut!(QUEUE));
                }
            } else {
                unsafe {
                    consumer(pid, &MUTEX, &AVAIL, &EMPTY, addr_of_mut!(QUEUE));
                }
            }
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }
    sys_stat();
    for i in 0..THREAD_COUNT {
        sys_wait_pid(pids[i]);
    }
    println!("Message queue is empty: {:?}", unsafe { QUEUE.is_empty() });
    0
}
#[derive(Debug, Clone, Copy)]
struct Message {
    pid: u16,
    value: usize,
}

impl Message {
    fn new(pid: u16, value: usize) -> Self {
        Message { pid, value }
    }
}

fn producer(
    pid: u16,
    mutex: &Semaphore,
    avail: &Semaphore,
    empty: &Semaphore,
    queue: *mut VecDeque<Message>,
) {
    for i in 0..10 {
        empty.wait();
        mutex.wait();
        unsafe {
            println!(
                "Producer {}: send message with content {}. Queue length: {:?}",
                pid,
                i,
                (*queue).len()
            );
            (*queue).push_back(Message::new(pid, i));
        }
        mutex.signal();
        avail.signal();
    }
}

fn consumer(
    pid: u16,
    mutex: &Semaphore,
    avail: &Semaphore,
    empty: &Semaphore,
    queue: *mut VecDeque<Message>,
) {
    for _ in 0..10 {
        avail.wait();
        mutex.wait();
        if let Some(mes) = unsafe { (*queue).pop_front() } {
            println!(
                "Consumer {}: message from {} with content {}. Queue length: {:?}",
                pid,
                mes.pid,
                mes.value,
                unsafe { (*queue).len() }
            );
        }
        mutex.signal();
        empty.signal();
    }
}

entry!(main);
