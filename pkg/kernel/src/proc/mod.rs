mod context;
mod data;
mod manager;
mod paging;
mod pid;
mod process;
mod processor;
mod sync;
mod vm;

use crate::proc::vm::ProcessVm;
pub use crate::resource::Resource;
use alloc::format;
use alloc::sync::Arc;
use alloc::vec::Vec;
use manager::*;
pub use process::ProcessPriority;
use process::*;
pub use processor::get_pid;

use alloc::string::{String, ToString};
pub use context::ProcessContext;
pub use data::ProcessData;
pub use paging::PageTableContext;
pub use pid::ProcessId;

use elf::ElfFile;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

use self::sync::SemaphoreResult;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init(boot_info: &'static boot::BootInfo) {
    // FIXME: set the kernel stack
    let proc_vm = ProcessVm::new(PageTableContext::new()).init_kernel_vm();

    trace!("Init kernel vm: {:#?}", proc_vm);

    let kproc_data = ProcessData::new();

    // kernel process
    let kproc = Process::new(
        String::from("kernel process"),
        None,
        Some(proc_vm),
        Some(kproc_data),
        ProcessPriority::Kernel as u8,
    );
    let app_list = boot_info.loaded_apps.as_ref();
    manager::init(kproc, app_list);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        get_process_manager().save_current(context);
        get_process_manager().switch_next(context);
        // debug!("already switch to next process\n");
    });
}

// pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let entry = VirtAddr::new(entry as usize as u64);
//         get_process_manager().spawn_kernel_thread(entry, name, data)
//     })
// }

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}

pub fn get_proc_exit_code(pid: ProcessId) -> Option<isize> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().get_proc_exit_code(pid)
    })
}

pub fn print_current_proc() -> String {
    format!("{:#?}", get_process_manager().current())
}

pub fn list_app() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().get_app_list();
        if app_list.is_none() {
            println!("[!] No app found in list!");
            return;
        }

        let apps = app_list
            .unwrap()
            .iter()
            .map(|app| app.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        // TODO: print more information like size, entry point, etc.

        println!("[+] App list: {}", apps);
    });
}

pub fn spawn(name: &str, priority: u8) -> Option<ProcessId> {
    let app = x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().get_app_list()?;
        app_list.iter().find(|&app| app.name.eq(name))
    })?;
    elf_spawn(name.to_string(), &app.elf, priority)
}

pub fn elf_spawn(name: String, elf: &ElfFile, priority: u8) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();
        let parent = Arc::downgrade(&manager.current());
        let pid = manager.spawn(elf, name, Some(parent), None, priority);

        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });

    Some(pid)
}

pub fn exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        manager.kill_current(ret); // FIXME: implement this for ProcessManager
        manager.switch_next(context);
    })
}

#[inline]
pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // check if the process is still alive
        get_process_manager().get_proc_status(pid) != ProgramStatus::Dead
    })
}

pub fn waitpid(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if let Some(ret) = manager.get_proc_exit_code(pid) {
            context.set_rax(ret as usize);
        } else {
            manager.waitpid(pid);
            manager.save_current(context);
            manager.current().write().block();
            manager.switch_next(context);
        }
    })
}

pub fn kill(pid: ProcessId, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        if pid == processor::get_pid() {
            manager.kill_current(-1);
            manager.switch_next(context);
        } else {
            manager.kill(pid, -1);
        }
    })
}

pub fn curr_sys_write(handle_num: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().curr_sys_write(handle_num, buf)
    })
}

pub fn curr_sys_read(handle_num: u8, buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().curr_sys_read(handle_num, buf)
    })
}

pub fn fork(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: save_current as parent
        manager.save_current(context);
        // FIXME: fork to get child
        manager.fork();
        // FIXME: switch to next process
        manager.switch_next(context);
    })
}

pub fn new_sem(key: u32, val: usize) -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let ret = manager.current().write().add_sem(key, val);
        ret as usize
    })
}

pub fn remove_sem(key: u32) -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let ret = manager.current().write().remove_sem(key);
        ret as usize
    })
}

pub fn sem_wait(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = processor::get_pid();
        let ret = manager.current().write().sem_wait(key, pid);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NotExist => context.set_rax(1),
            SemaphoreResult::Block(pid0) => {
                // FIXME: save, block it, then switch to next
                //        maybe use `save_current` and `switch_next`
                if pid0 == pid {
                    manager.save_current(context);
                    manager.block_current();
                    manager.switch_next(context);
                } else {
                    manager.block(pid0);
                }
            }
            _ => unreachable!(),
        }
    })
}

pub fn sem_signal(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let ret = manager.current().write().sem_signal(key);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NotExist => context.set_rax(1),
            SemaphoreResult::WakeUp(pid0) => {
                manager.wake_up(pid0, None);
            }
            _ => unreachable!(),
        }
    })
}

pub fn get_priority(pid: ProcessId) -> u8 {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().get_priority(pid))
}

pub fn set_priority(pid: ProcessId, priority: u8) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().set_priority(pid, priority);
    })
}
