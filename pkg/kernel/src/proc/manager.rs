use super::*;

use alloc::{
    collections::*,
    format,
    sync::{Arc, Weak},
};
use boot::AppListRef;
use manager::vm::{stack::STACK_INIT_TOP, ProcessVm};
use spin::{Mutex, RwLock};

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, boot_info: AppListRef) {
    // FIXME: set init process as Running
    init.write().resume();
    // init.write().pause();
    // FIXME: set processor's current pid to init's pid
    processor::set_pid(init.pid());
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init, boot_info));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef<'static>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>, boot_info: AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: boot_info,
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        let curr = self.current();
        // FIXME: update current process's tick count
        let mut curr_guard = curr.write();
        curr_guard.tick();
        // FIXME: update current process's context
        curr_guard.save(context);
        // FIXME: push current process to ready queue if still alive
        if !curr_guard.is_dead() {
            self.push_ready(curr.pid());
        }
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {
        let mut front_id = processor::get_pid();
        // FIXME: fetch the next process from ready queue
        while let Some(front_id1) = self.ready_queue.lock().pop_front() {
            // extend the lifetime of processes_guard
            let processes_guard = self.processes.read();
            let proc1 = processes_guard.get(&front_id1).expect("process not found");
            // FIXME: check if the next process is ready,
            //        continue to fetch if not ready
            if !proc1.read().is_ready() {
                continue;
            }
            // avoid duplicate process in the ready queue
            if front_id1 != front_id {
                front_id = front_id1;
                break;
            }
        }
        let processes_guard = self.processes.read();
        let proc1 = processes_guard.get(&front_id).expect("process not found");
        // FIXME: restore next process's context
        proc1.write().restore(context);
        // FIXME: update processor's current pid
        processor::set_pid(front_id);
        front_id
        // KERNEL_PID
    }

    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let proc_vm = Some(ProcessVm::new(page_table));
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), proc_vm, proc_data);

    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();

    //     // FIXME: set the stack frame
    //     proc.write().init_stack_frame(entry, stack_top);
    //     debug!("process status: {:#?}", proc);
    //     let proc_pid = proc.pid();
    //     // FIXME: add to process map
    //     self.add_proc(proc_pid, proc);
    //     // FIXME: push to ready queue
    //     self.push_ready(proc_pid);
    //     proc_pid
    // }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        let curr_proc = get_process_manager().current();
        if !err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION)
            && !err_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE)
        {
            return false;
        }
        // handle page fault in current process
        let ret = curr_proc.write().vm_mut().handle_page_fault(addr);
        ret
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output =
            String::from("  PID | PPID | Process Name |  Ticks  | Status | Memory Usage\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
    pub fn get_proc_exit_code(&self, pid: ProcessId) -> Option<isize> {
        if let Some(proc) = self.processes.read().get(&pid) {
            let proc = proc.read();
            if proc.status() == ProgramStatus::Dead {
                return proc.exit_code();
            }
        }
        None
    }
    pub fn get_app_list(&self) -> boot::AppListRef {
        self.app_list
    }
    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc_vm = Some(ProcessVm::new(page_table));
        let proc = Process::new(name, parent, proc_vm, proc_data);
        let pid = proc.pid();

        // FIXME: load elf to process pagetable
        proc.write().load_elf(elf);
        // debug!("loading elf to process pagetable");
        // FIXME: alloc new stack for process
        let entry = VirtAddr::new(elf.header.pt2.entry_point());
        proc.write()
            .init_stack_frame(entry, VirtAddr::new(STACK_INIT_TOP));
        // FIXME: mark process as ready
        proc.write().pause();

        trace!("New {:#?}", &proc);

        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);
        pid
    }

    pub fn get_proc_status(&self, pid: ProcessId) -> ProgramStatus {
        if let Some(proc) = self.get_proc(&pid) {
            proc.read().status()
        } else {
            ProgramStatus::Dead
        }
    }

    pub fn waitpid(&self, pid: ProcessId) -> isize {
        self.get_proc_exit_code(pid).unwrap_or(-1)
    }

    pub fn curr_sys_write(&self, fd: u8, buf: &[u8]) -> isize {
        self.current().read().sys_write(fd, buf)
    }

    pub fn curr_sys_read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.current().read().sys_read(fd, buf)
    }
}
