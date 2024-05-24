#![no_std]
#![no_main]

use lib::*;

extern crate lib;

const THREAD_COUNT: usize = 8;
static mut COUNTER: isize = 0;

fn main() -> isize {
    test_without_protection();
    test_with_spin();
    test_with_spinguard();
    test_semaphores();
    0
}

fn do_counter_inc() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        inc_counter();
    }
}
fn do_counter_inc_with_spinlock_guard(lock: &sync::SpinLock1<isize>){
    for _ in 0..100{
        let mut lock_guard = lock.acquire();
        *lock_guard += 1;
    }
}
fn do_counter_inc_with_spinlock(lock: &sync::SpinLock) {
    for _ in 0..100 {
        lock.acquire();
        inc_counter();
        lock.release();
    }
}

fn do_counter_inc_with_semaphore(key: u32){
    for _ in 0..100{
        sys_sem_wait(key);
        inc_counter();
        sys_sem_signal(key);
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter() {
    unsafe {
        delay();
        let mut val = COUNTER;
        delay();
        val += 1;
        delay();
        COUNTER = val;
    }
}
fn test_with_spinguard() {
    unsafe {
        COUNTER = 0;
    }
    println!("\x1b[32mTest with spinlock_guard...\x1b[0m");
    let mut pids = [0u16; THREAD_COUNT];
    static LOCK: sync::SpinLock1<isize> = sync::SpinLock1::new(0);
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_with_spinlock_guard(&LOCK);
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }
    let lock_guard = LOCK.acquire();
    
    println!("COUNTER result: {}", *lock_guard);
}
fn test_with_spin() {
    unsafe {
        COUNTER = 0;
    }
    println!("\x1b[32mTest with spinlock...\x1b[0m");
    let mut pids = [0u16; THREAD_COUNT];
    static LOCK: sync::SpinLock = sync::SpinLock::new();
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_with_spinlock(&LOCK);
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}

fn test_semaphores() {
    unsafe {
        COUNTER = 0;
    }
    println!("\x1b[34mTest with semaphores...\x1b[0m");
    let mut pids = [0u16; THREAD_COUNT];
    let key = 1;
    if !sys_new_sem(1, 1){
        println!("Semaphore already exists");
        sys_remove_sem(key);
        sys_new_sem(key, 1);
    }
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_with_semaphore(key);
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}

fn test_without_protection() {
    unsafe {
        COUNTER = 0;
    }
    println!("\x1b[33mTest without protection...\x1b[0m");
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });
}
#[inline(never)]
#[no_mangle]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
