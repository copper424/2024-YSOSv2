use super::*;
use crate::memory::{self, *};
use alloc::sync::Weak;
use spin::rwlock::RwLock;
use spin::*;
use x86_64::structures::paging::page::PageRange;
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
        debug!("Allocating initial stack for process {}", self.pid);
        // only allocate one page for the initial stack
        match elf::map_range(
            init_stack_low,
            STACK_DEF_PAGE,
            &mut page_mapper,
            frame_allocator,
            true,
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
        // self.parent.take();
    }

    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.context.init_stack_frame(entry, stack_top);
    }
    pub fn proc_page_fault_handler(&mut self, addr: VirtAddr) {
        let addr_at_page = Page::<Size4KiB>::containing_address(addr);
        let stack_segment = self.proc_data.as_ref().unwrap().stack_segment.unwrap();
        let start_page = stack_segment.start;
        let alloc_page_nums = start_page - addr_at_page;
        let original_page_size = stack_segment.end - start_page;

        let mut page_table_mapper = self
            .page_table
            .as_ref()
            .expect("page table is none")
            .mapper();

        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let user_access = processor::get_pid() != KERNEL_PID;
        match elf::map_range(
            addr_at_page.start_address().as_u64(),
            alloc_page_nums,
            &mut page_table_mapper,
            frame_allocator,
            user_access,
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
        self.proc_data.as_mut().unwrap().stack_pages = size as usize;
    }

    pub fn load_elf(&mut self, elf: &ElfFile) {
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let mut page_table_mapper = self
            .page_table
            .as_ref()
            .expect("page table did not exist!\n")
            .mapper();
        // map elf segments to new frames
        if let Err(e) = elf::load_elf(
            elf,
            PHYSICAL_OFFSET.get().cloned().unwrap(),
            &mut page_table_mapper,
            frame_allocator,
            true,
        ) {
            debug!("Failed to load ELF: {:?}", e);
        }
        // map and allocate stack
        if let Err(e) = elf::map_range(
            STACK_INIT_BOT,
            STACK_DEF_PAGE,
            &mut page_table_mapper,
            frame_allocator,
            true,
        ) {
            debug!("Failed to map stack: {:?}", e);
        }
        const STACK_INIT_END: u64 = STACK_INIT_BOT + STACK_DEF_SIZE;
        let stack_segment = PageRange {
            start: Page::containing_address(VirtAddr::new(STACK_INIT_BOT)),
            end: Page::containing_address(VirtAddr::new(STACK_INIT_END)),
        };
        let code_segment: Vec<PageRange> = elf
            .program_iter()
            .filter(|p| p.get_type().unwrap() == elf::program::Type::Load)
            .map(|segment_header| {
                let start = Page::containing_address(VirtAddr::new(segment_header.virtual_addr()));
                let end = Page::containing_address(VirtAddr::new(
                    segment_header.virtual_addr() + segment_header.mem_size(),
                ));
                PageRange { start, end }
            })
            .collect();
        self.proc_data.as_mut().unwrap().stack_pages = stack_segment.count();
        self.proc_data.as_mut().unwrap().code_pages = code_segment
            .iter()
            .map(|code_segment| code_segment.count())
            .sum();
        self.proc_data.as_mut().unwrap().stack_segment = Some(stack_segment);
        self.proc_data.as_mut().unwrap().code_segment = Some(code_segment);
    }

    pub fn fork(&mut self, parent: Weak<Process>) -> ProcessInner {
        // FIXME: get current process's stack info
        let mut child_context = self.context;
        let stack_seg = self.stack_segment.as_ref().unwrap();

        // FIXME: clone the process data struct
        let mut proc_data = self.proc_data.clone().unwrap();

        // FIXME: clone the page table context (see instructions)
        let page_table = self.page_table.as_ref().unwrap().fork();
        let mut page_mapper = page_table.mapper();

        // FIXME: alloc & map new stack for child (see instructions)
        // FIXME: copy the *entire stack* from parent to child
        let origin_stack_base = stack_seg.start.start_address().as_u64();
        let mut new_stack_base =
            origin_stack_base - (self.children.len() + 1) as u64 * STACK_MAX_SIZE;
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        while elf::map_range(
            new_stack_base,
            stack_seg.count() as u64,
            &mut page_mapper,
            frame_allocator,
            true,
        )
        .is_err()
        {
            trace!("Map thread stack to {:#x} failed.", new_stack_base);
            new_stack_base -= STACK_MAX_SIZE; // stack grow down
        }
        memory::clone_range(origin_stack_base, new_stack_base, stack_seg.count());
        // FIXME: update child's context with new *stack pointer*
        //          > update child's stack to new base
        //          > keep lower bits of *rsp*, update the higher bits
        //          > also update the stack record in process data
        let stack_offset = new_stack_base - origin_stack_base;
        child_context.as_mut().as_mut_ptr().update(|mut context| {
            context.stack_frame.stack_pointer += stack_offset;
            context
        });
        proc_data.set_stack(VirtAddr::new(new_stack_base), stack_seg.count() as u64);
        proc_data.code_pages = 0;
        proc_data.stack_pages = stack_seg.count();
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
            page_table: Some(page_table),
            proc_data: Some(proc_data),
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
        let (memory_usage, memory_unit) = inner
            .proc_data
            .as_ref()
            .map(|d| {
                let total_pages = d.stack_pages + d.code_pages;
                let total_size = total_pages as u64 * PAGE_SIZE;
                memory::humanized_size(total_size)
            })
            .unwrap_or((0f64, "B"));
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
