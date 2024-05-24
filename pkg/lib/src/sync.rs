use core::{
    cell::UnsafeCell,
    hint::spin_loop,
    ops::{Deref, DerefMut, Drop},
    result::Result,
    sync::atomic::{AtomicBool, Ordering},
};
use core::hint;

use crate::*;

pub struct SpinLock {
    bolt: AtomicBool,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            bolt: AtomicBool::new(false),
        }
    }

    pub fn acquire(&self) {
        // FIXME: acquire the lock, spin if the lock is not available
        while self
            .bolt
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            == Err(true)
        {
            hint::spin_loop();
        }
    }

    pub fn release(&self) {
        // FIXME: release the lock
        self.bolt.store(false, Ordering::SeqCst);
    }
}

unsafe impl Sync for SpinLock {} // Why? Check reflection question 5

pub struct SpinLock1<T: ?Sized> {
    bolt: AtomicBool,
    data: UnsafeCell<T>,
}
impl<T> SpinLock1<T> {
    pub const fn new(data: T) -> Self {
        Self {
            bolt: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn acquire(&self) -> SpinLock1Guard<T> {
        while self
            .bolt
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            hint::spin_loop();
        }
        SpinLock1Guard {
            bolt: &self.bolt,
            data: self.data.get(),
        }
    }

    pub fn try_acquire(&self) -> Result<SpinLock1Guard<T>, ()> {
        if self
            .bolt
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return Ok(SpinLock1Guard {
                bolt: &self.bolt,
                data: self.data.get(),
            });
        }
        Err(())
    }
}
unsafe impl<T: ?Sized> Sync for SpinLock1<T> {}

pub struct SpinLock1Guard<'a, T: ?Sized + 'a> {
    bolt: &'a AtomicBool,
    data: *mut T,
}

impl<'a, T:?Sized> Deref for SpinLock1Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<'a, T:?Sized> DerefMut for SpinLock1Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<'a, T: ?Sized> Drop for SpinLock1Guard<'a, T> {
    fn drop(&mut self) {
        self.bolt.store(false, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semaphore {
    /* FIXME: record the sem key */
    key: u32,
}

impl Semaphore {
    pub const fn new(key: u32) -> Self {
        Semaphore { key }
    }

    #[inline(always)]
    pub fn init(&self, value: usize) -> bool {
        sys_new_sem(self.key, value)
    }

    /* FIXME: other functions with syscall... */
    #[inline(always)]
    pub fn wait(&self) {
        sys_sem_wait(self.key);
    }

    #[inline(always)]
    pub fn signal(&self) {
        sys_sem_signal(self.key);
    }

    #[inline(always)]
    pub fn destroy(&self) -> bool {
        sys_remove_sem(self.key)
    }
}

unsafe impl Sync for Semaphore {}

#[macro_export]
macro_rules! semaphore_array {
    [$($x:expr),+ $(,)?] => {
        [ $($crate::Semaphore::new($x),)* ]
    }
}
