use core::alloc::Layout;

use x86_64::VirtAddr;

use crate::proc::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    let name = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            args.arg0 as *const u8,
            args.arg1,
        ))
    };
    if let Some(pid) = spawn(name) {
        return pid.0 as usize;
    }
    0
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get handle by fd
    // FIXME: handle read from fd & return length
    //       - core::slice::from_raw_parts
    // FIXME: return 0 if failed
    let handle_num = args.arg0 as u8;
    let buf = unsafe { core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2) };
    curr_sys_write(handle_num, buf) as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let handle_num = args.arg0 as u8;
    let buf = unsafe { core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2) };
    curr_sys_read(handle_num, buf) as usize
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    crate::proc::exit(args.arg0 as isize, context);
}

pub fn list_process() {
    // FIXME: list all processes
    print_process_list();
}

pub fn waitpid(args: &SyscallArgs, context: &mut ProcessContext) {
    let pid = ProcessId(args.arg0 as u16);
    crate::proc::waitpid(pid, context);
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_kill(args: &SyscallArgs, context: &mut ProcessContext) {
    // kill process according to the given PID
    let pid = ProcessId(args.arg0 as u16);
    crate::proc::kill(pid, context);
}

pub fn sys_time() -> usize {
    // get current time
    let time = crate::utils::uefi_runtime::UEFI_RUNTIME
        .get()
        .unwrap()
        .lock()
        .get_time();
    let datetime =
        chrono::NaiveDate::from_ymd_opt(time.year() as i32, time.month() as u32, time.day() as u32)
            .unwrap()
            .and_hms_nano_opt(
                time.hour() as u32,
                time.minute() as u32,
                time.second() as u32,
                time.nanosecond() as u32,
            )
            .unwrap();
    datetime.and_utc().timestamp_millis() as usize
}

pub fn sys_fork(context: &mut ProcessContext) {
    fork(context);
}
/// op: `u8`, key: `u32`, val: `usize` -> ret: `any`
pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(remove_sem(args.arg1 as u32)),
        2 => sem_signal(args.arg1 as u32, context),
        3 => sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}

pub fn sys_brk(args: &SyscallArgs) -> usize {
    let new_heap_end = if args.arg0 == 0 {
        None
    } else {
        Some(VirtAddr::new(args.arg0 as u64))
    };
    match brk(new_heap_end) {
        Some(addr) => addr.as_u64() as usize,
        None => !0,
    }
}
