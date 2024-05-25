use alloc::{format, vec::Vec};
use elf::ElfFile;
use x86_64::{
    structures::paging::{page::PageRange, *},
    VirtAddr,
};

use crate::memory::*;
use crate::memory::{self, humanized_size};

pub mod stack;

use self::stack::*;

use super::PageTableContext;

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,

    pub(super) code_segment: Vec<PageRange>,
    // number of pages
    pub(super) code_segment_size: u64,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
            code_segment: Vec::new(),
            code_segment_size: 0,
        }
    }

    pub fn init_kernel_vm(mut self) -> Self {
        // TODO: record kernel code usage
        self.stack = Stack::kstack();
        self
    }

    pub fn handle_page_fault(&mut self, addr: VirtAddr) -> bool {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.handle_page_fault(addr, mapper, alloc)
    }

    pub(super) fn memory_usage(&self) -> u64 {
        let stack_usage = self.stack.memory_usage();
        let code_usage = self.code_segment_size * memory::PAGE_SIZE;
        stack_usage + code_usage
    }

    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();

        self.stack.init(mapper, alloc);

        // FIXME: load elf to process pagetable
        // map elf segments to new frames
        if let Err(e) = elf::load_elf(
            elf,
            PHYSICAL_OFFSET.get().cloned().unwrap(),
            mapper,
            alloc,
            true,
        ) {
            debug!("Failed to load ELF: {:?}", e);
        }
        // code segment information
        let code_segment = elf
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
        self.code_segment = code_segment;

        let code_page_size: usize = self
            .code_segment
            .iter()
            .map(|code_segment| code_segment.count())
            .sum();
        self.code_segment_size = code_page_size as u64;
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
