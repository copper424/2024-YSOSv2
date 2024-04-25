use core::alloc::Layout;

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
    if let Some(resource) = crate::proc::handle(handle_num) {
        let buf = unsafe { core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2) };
        if let Some(len) = resource.write(buf) {
            return len;
        }
    }
    0
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let handle_num = args.arg0 as u8;
    if let Some(resource) = crate::proc::handle(handle_num) {
        let buf = unsafe { core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2) };
        if let Some(len) = resource.read(buf) {
            return len;
        }
    }
    0
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    crate::proc::exit(args.arg0 as isize, context);
}

pub fn list_process() {
    // FIXME: list all processes
    print_process_list();
}

pub fn waitpid(args: &SyscallArgs) -> isize {
    let pid = ProcessId(args.arg0 as u16);
    crate::proc::waitpid(pid)
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
