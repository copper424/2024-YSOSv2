use crate::Syscall;
use core::arch::asm;

#[doc(hidden)]
#[inline(always)]
pub fn syscall0(n: Syscall) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            lateout("rax") ret
        );
    }
    ret
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall1(n: Syscall, arg0: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            in("rdi") arg0,
            lateout("rax") ret
        );
    }
    ret
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall2(n: Syscall, arg0: usize, arg1: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            in("rdi") arg0, in("rsi") arg1,
            lateout("rax") ret
        );
    }
    ret
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall3(n: Syscall, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "int 0x80", in("rax") n as usize,
            in("rdi") arg0, in("rsi") arg1, in("rdx") arg2,
            lateout("rax") ret
        );
    }
    ret
}

#[macro_export]
macro_rules! syscall {
    ($n:expr) => {
        $crate::macros::syscall0($n)
    };
    ($n:expr, $a1:expr) => {
        $crate::macros::syscall1($n, $a1 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr) => {
        $crate::macros::syscall2($n, $a1 as usize, $a2 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::macros::syscall3($n, $a1 as usize, $a2 as usize, $a3 as usize)
    };
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall0_64(n: Syscall) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "syscall", inout("rax") (n as usize) => ret,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack),
        );
    }
    ret
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall1_64(n: Syscall, arg0: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "syscall", inout("rax") (n as usize) => ret,
            in("rdi") arg0,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack),
        );
    }
    ret
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall2_64(n: Syscall, arg0: usize, arg1: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "syscall", inout("rax") (n as usize) => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack),
        );
    }
    ret
}

#[doc(hidden)]
#[inline(always)]
pub fn syscall3_64(n: Syscall, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret: usize;
    unsafe {
        asm!(
            "syscall", inout("rax") (n as usize) => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            out("rcx") _, // rcx is used to store old rip
            out("r11") _, // r11 is used to store old rflags
            options(nostack),
        );
    }
    ret
}

#[macro_export]
macro_rules! syscall64 {
    ($n:expr) => {
        $crate::macros::syscall0_64($n)
    };
    ($n:expr, $a1:expr) => {
        $crate::macros::syscall1_64($n, $a1 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr) => {
        $crate::macros::syscall2_64($n, $a1 as usize, $a2 as usize)
    };
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => {
        $crate::macros::syscall3_64($n, $a1 as usize, $a2 as usize, $a3 as usize)
    };
}
