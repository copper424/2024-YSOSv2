use alloc::format;
use x86_64::{
    structures::paging::{page::PageRange, *},
    VirtAddr,
};

use crate::memory::humanized_size;
use crate::memory::*;

pub mod stack;

use self::stack::*;

use super::{PageTableContext, ProcessId};

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
        self
    }

    pub fn init_proc_stack(&mut self, pid: ProcessId) -> VirtAddr {
        // FIXME: calculate the stack for pid
        let offset_per_pid = (pid.0 as u64 - 1) * STACK_MAX_SIZE;
        // [init_stack_low, init_statck_high)
        let init_stack_high = VirtAddr::new(STACK_MAX - offset_per_pid);
        let init_stack_low = VirtAddr::new(init_stack_high.as_u64() - STACK_DEF_SIZE);

        let mut page_mapper = self.page_table.mapper();
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        match elf::map_range(
            init_stack_low.as_u64(),
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
        let stack_top_addr = VirtAddr::new(STACK_INIT_TOP - offset_per_pid);
        self.stack = Stack::from_range(
            PageRange {
                start: Page::containing_address(init_stack_low),
                end: Page::containing_address(init_stack_high),
            },
            STACK_DEF_PAGE,
        );
        stack_top_addr
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        self.stack.memory_usage()
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
