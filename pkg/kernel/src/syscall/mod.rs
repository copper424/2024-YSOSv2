pub mod service;
use crate::memory::gdt;
use crate::proc::{list_app, ProcessContext};
use alloc::format;
use service::*;
use syscall_def::Syscall;
use x86_64::{registers::rflags::RFlags, VirtAddr};

static mut PRIVILEGE_RSP: u64 = 0;
static mut USER_RSP: u64 = 0;

pub fn init() {
    info!("Initializing syscall module.");
    use x86_64::registers::model_specific::*;

    unsafe {
        let mut efer_flags = Efer::read();
        efer_flags.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
        Efer::write(efer_flags);
        info!("EferFlags: {:#?}", Efer::read());
    }

    // kernel code segment
    let syscall_val = gdt::get_selector().code_selector.0;
    // user code segment
    let sysret_val = gdt::get_user_selector().code_selector.0 - 16;

    unsafe {
        // segment selector in GDT table should conform the boundary here
        Star::write_raw(sysret_val, syscall_val);
    }
    // warn!("Star reg:{:#?}", Star::read());
    // warn!("Raw Star reg:{:#?}", Star::read_raw());

    LStar::write(VirtAddr::new(syscall_handler as u64));
    SFMask::write(
        RFlags::INTERRUPT_FLAG
            | RFlags::TRAP_FLAG
            | RFlags::DIRECTION_FLAG
            | RFlags::IOPL_LOW
            | RFlags::IOPL_HIGH
            | RFlags::ALIGNMENT_CHECK
            | RFlags::NESTED_TASK,
    );
    unsafe {
        PRIVILEGE_RSP = gdt::get_syscall_stack().as_u64() - 8;
    }
    GsBase::write(gdt::get_syscall_stack());
}

#[derive(Clone, Debug)]
pub struct SyscallArgs {
    pub syscall: Syscall,
    pub arg0: usize,
    pub arg1: usize,
    pub arg2: usize,
}

pub fn dispatcher(context: &mut ProcessContext) {
    let args = super::syscall::SyscallArgs::new(
        Syscall::from(context.regs.rax),
        context.regs.rdi,
        context.regs.rsi,
        context.regs.rdx,
    );

    match args.syscall {
        // fd: arg0 as u8, buf: &[u8] (ptr: arg1 as *const u8, len: arg2)
        Syscall::Read => {
            context.set_rax(sys_read(&args));
        }
        // fd: arg0 as u8, buf: &[u8] (ptr: arg1 as *const u8, len: arg2)
        Syscall::Write => {
            context.set_rax(sys_write(&args));
        }

        // None -> pid: u16
        Syscall::GetPid => {
            context.set_rax(crate::proc::get_pid().0 as usize);
        }
        Syscall::Time => {
            context.set_rax(sys_time() as usize);
        }
        Syscall::Sem => {
            sys_sem(&args, context);
        }
        Syscall::Fork => {
            sys_fork(context);
        }
        // path: &str (ptr: arg0 as *const u8, len: arg1) -> pid: u16
        Syscall::Spawn => {
            context.set_rax(spawn_process(&args));
        }
        // ret: arg0 as isize
        Syscall::Exit => {
            exit_process(&args, context);
        }
        // pid: arg0 as u16 -> status: isize
        Syscall::WaitPid => {
            service::waitpid(&args, context);
        }
        // pid: arg0 as u16
        Syscall::Kill => {
            sys_kill(&args, context);
        }

        // None
        Syscall::Stat => {
            list_process();
        }
        // None
        Syscall::ListApp => {
            list_app();
        }

        // ----------------------------------------------------
        // NOTE: following syscall examples are implemented
        // ----------------------------------------------------

        // layout: arg0 as *const Layout -> ptr: *mut u8
        Syscall::Allocate => context.set_rax(sys_allocate(&args)),
        // ptr: arg0 as *mut u8
        Syscall::Deallocate => sys_deallocate(&args),
        // Unknown
        Syscall::Unknown => warn!("Unhandled syscall: {:x?}", context.regs.rax),
    }
}

impl SyscallArgs {
    pub fn new(syscall: Syscall, arg0: usize, arg1: usize, arg2: usize) -> Self {
        Self {
            syscall,
            arg0,
            arg1,
            arg2,
        }
    }
}

impl core::fmt::Display for SyscallArgs {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "SYSCALL: {:<10} (0x{:016x}, 0x{:016x}, 0x{:016x})",
            format!("{:?}", self.syscall),
            self.arg0,
            self.arg1,
            self.arg2
        )
    }
}

pub extern "C" fn syscall_handler_inner(mut context: ProcessContext) {
    // x86_64::instructions::interrupts::disable();
    // info!("Syscall context: {:#?}", context);
    dispatcher(&mut context);
    // x86_64::instructions::interrupts::enable();
}

pub extern "C" fn syscall_handler() {
    unsafe {
        core::arch::asm!(
            "
            // swapgs
            mov {0}[rip], rsp
            mov rsp, {1}[rip]
            push 0x2B
            push {0}[rip]
            push r11
            push 0x33
            push rcx
            push rbp
            push rax
            push rbx
            push rcx
            push rdx
            push rsi
            push rdi
            push r8
            push r9
            push r10
            push r11
            push r12
            push r13
            push r14
            push r15
            call {syscall_handler_inner}
            pop r15
            pop r14
            pop r13
            pop r12
            pop r11
            pop r10
            pop r9
            pop r8
            pop rdi
            pop rsi
            pop rdx
            pop rcx
            pop rbx
            pop rax
            pop rbp
            pop rcx
            add rsp, 0x8
            pop r11
            add rsp, 0x10
            mov {1}[rip], rsp
            mov rsp, {0}[rip]
            // swapgs
            // enable interrupt
            sti
            sysretq",
            sym USER_RSP,
            sym PRIVILEGE_RSP,
            syscall_handler_inner=sym syscall_handler_inner,
            // without nostack option, the compiler will add `pushq rax` 
            // at the front of the assembly block
            options(noreturn,nostack)
        );
    }
}
