
use super::*;
use crate::memory::*;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::*;

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
    page_table: Option<PageTableContext>,
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
        page_table: PageTableContext,
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
            page_table: Some(page_table),
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

    pub fn alloc_init_stack(&self) -> VirtAddr {
        // FIXME: alloc init stack base on self pid
        let offset_per_pid = (self.pid().0 as u64 - 1) * STACK_MAX_SIZE;
        // [init_stack_low, init_statck_high)
        let init_stack_high = STACK_MAX - offset_per_pid;
        let init_stack_low = init_stack_high - STACK_DEF_SIZE;

        let mut process_inner_guard = self.write();
        let mut page_mapper = process_inner_guard.page_table.as_ref().unwrap().mapper();
        // debug!("page mapper status:{:#?}", page_mapper);

        let frame_allocator = &mut *get_frame_alloc_for_sure();
        // only allocate one page for the initial stack
        match elf::map_range(
            init_stack_low,
            STACK_DEF_PAGE,
            &mut page_mapper,
            frame_allocator,
        ) {
            Ok(res) => {
                debug!("Allocated initial stack: {:?}", res);
            }
            Err(e) => {
                warn!("Failed to allocate initial stack: {:?}", e);
            }
        }
        process_inner_guard.set_stack(VirtAddr::new(init_stack_low), STACK_DEF_PAGE);
        let init_stack_top = STACK_INIT_TOP - offset_per_pid;
        VirtAddr::new(init_stack_top)
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

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.page_table.as_ref().unwrap().clone_l4()
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
        self.page_table.as_ref().unwrap().load();
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
        self.page_table.take();
        self.proc_data.take();
        self.parent.take();
    }

    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.context.init_stack_frame(entry, stack_top);
    }
    pub fn proc_page_fault_handler(&mut self, addr: VirtAddr) {
        let addr_at_page = Page::<Size4KiB>::containing_address(addr);
        let stack_segment = self
            .proc_data
            .as_ref()
            .unwrap()
            .stack_segment
            .as_ref()
            .unwrap();
        let start_page = stack_segment.start;
        let alloc_page_nums = start_page - addr_at_page;
        let original_page_size = stack_segment.end - start_page;

        let mut page_table_mapper = self
            .page_table
            .as_ref()
            .expect("page table is none")
            .mapper();

        let frame_allocator = &mut *get_frame_alloc_for_sure();
        match elf::map_range(
            addr_at_page.start_address().as_u64(),
            alloc_page_nums,
            &mut page_table_mapper,
            frame_allocator,
        ) {
            Ok(res) => {
                debug!("Allocated stack for page fault exception: {:?}", res);
            }
            Err(e) => {
                warn!("Failed to allocate stack for page fault exception: {:?}", e);
            }
        }
        let size = original_page_size + alloc_page_nums;
        self.set_stack(addr_at_page.start_address(), size);
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
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status
        )?;
        Ok(())
    }
}
