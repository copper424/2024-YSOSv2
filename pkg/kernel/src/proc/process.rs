use self::vm::stack::STACK_MAX_SIZE;
use self::vm::ProcessVm;

use super::*;
use crate::memory::*;
use alloc::sync::Weak;
use spin::rwlock::RwLock;
use spin::*;

#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    proc_vm: Option<ProcessVm>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        proc_vm: Option<ProcessVm>,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            proc_vm,
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
    }

    // pub fn alloc_init_stack(&self) -> VirtAddr {
    //     self.write().vm_mut().init_proc_stack(self.pid)
    // }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // FIXME: lock inner as write
        let mut inner = self.inner.write();
        // FIXME: inner fork with parent weak ref
        let child_inner = inner.fork(Arc::downgrade(self));
        let child_pid = ProcessId::new();
        let child = Process {
            pid: child_pid,
            inner: Arc::new(RwLock::new(child_inner)),
        };
        // FOR DBG: maybe print the child process info
        //          e.g. parent, name, pid, etc.
        debug!("The forked child process information:{:#?}", child);
        // FIXME: make the arc of child
        let child = Arc::new(child);
        // FIXME: add child to current process's children list
        inner.children.push(child.clone());
        // FIXME: set fork ret value for parent with `context.set_rax`
        inner.context.set_rax(child.pid().0 as usize);
        // FIXME: mark the child as ready & return it
        // child.write().pause();
        child
    }
}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn block(&mut self) {
        self.status = ProgramStatus::Blocked;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn set_rax(&mut self, value: usize) {
        self.context.set_rax(value);
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.vm().page_table.clone_l4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }
    pub fn is_running(&self) -> bool {
        self.status == ProgramStatus::Running
    }
    pub fn is_dead(&self) -> bool {
        self.status == ProgramStatus::Dead
    }
    pub fn vm(&self) -> &ProcessVm {
        self.proc_vm.as_ref().unwrap()
    }

    pub fn vm_mut(&mut self) -> &mut ProcessVm {
        self.proc_vm.as_mut().unwrap()
    }
    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context.save(context);
        if self.is_running() {
            self.pause();
        }
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        // FIXME: restore the process's page table
        self.vm().page_table.load();
        self.resume();
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // FIXME: take and drop unused resources
        self.proc_vm.take();
        self.proc_data.take();
        // self.parent.take();
    }

    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.context.init_stack_frame(entry, stack_top);
    }
    pub fn load_elf(&mut self, elf: &ElfFile) {
        self.vm_mut().load_elf(elf);
    }

    pub fn fork(&self, parent: Weak<Process>) -> ProcessInner {
        // FIXME: calculate the initial stack offset in bytes
        let stack_offset_count = (self.children.len() + 1) as u64 * STACK_MAX_SIZE;
        // FIXME: fork the process virtual memory struct
        let proc_vm = self.vm().fork(stack_offset_count);

        // the real stack offset
        let real_stack_offset = self.vm().stack.cal_offset(&proc_vm.stack);
        let mut child_context = self.context;
        // FIXME: update `rsp` in interrupt stack frame
        child_context.as_mut().as_mut_ptr().update(|mut context| {
            context.stack_frame.stack_pointer -= real_stack_offset;
            context
        });
        // FIXME: set the return value 0 for child with `context.set_rax`
        child_context.set_rax(0);

        // FIXME: construct the child process inner
        let child_inner = ProcessInner {
            name: self.name.clone(),
            children: Vec::new(),
            parent: Some(parent),
            ticks_passed: 0,
            status: ProgramStatus::Ready,
            exit_code: self.exit_code,
            context: child_context,
            proc_vm: Some(proc_vm),
            // FIXME: clone the process data struct
            proc_data: self.proc_data.clone(),
        };
        // NOTE: return inner because there's no pid record in inner
        child_inner
    }

    pub fn add_sem(&self, key: u32, value: usize) -> bool {
        let mut sems_guard = self.semaphores.write();
        sems_guard.insert(key, value)
    }
    pub fn remove_sem(&self, key: u32) -> bool {
        let mut sems_guard = self.semaphores.write();
        sems_guard.remove(key)
    }
    pub fn sem_wait(&self, key: u32, pid: ProcessId) -> SemaphoreResult {
        self.semaphores.read().wait(key, pid)
    }
    pub fn sem_signal(&self, key: u32) -> SemaphoreResult {
        self.semaphores.read().signal(key)
    }

    pub fn brk(&self, addr: Option<VirtAddr>) -> Option<VirtAddr> {
        self.vm().brk(addr)
    }
}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("vm", &inner.proc_vm);
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        let (memory_usage, memory_unit) = humanized_size(inner.vm().memory_usage());
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?} | {:<} {}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status,
            memory_usage,
            memory_unit,
        )?;
        Ok(())
    }
}
