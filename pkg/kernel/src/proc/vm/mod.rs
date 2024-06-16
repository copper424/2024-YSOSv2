use alloc::{borrow::ToOwned, format, vec::Vec};
use boot::KernelPages;
use elf::ElfFile;
use x86_64::{
    structures::paging::{
        mapper::{CleanUp, UnmapError},
        page::*,
        *,
    },
    VirtAddr,
};

use crate::memory::*;
use crate::memory::{self, humanized_size};
pub mod heap;
pub mod stack;

use self::{heap::Heap, stack::Stack};

use super::PageTableContext;

// See the documentation for the `KernelPages` type
// Ignore when you not reach this part
//
// use boot::KernelPages;

type MapperRef<'a> = &'a mut OffsetPageTable<'static>;
type FrameAllocatorRef<'a> = &'a mut BootInfoFrameAllocator;

pub struct ProcessVm {
    // page table is shared by parent and child
    pub(super) page_table: PageTableContext,

    // stack is pre-process allocated
    pub(super) stack: Stack,

    // heap is allocated by brk syscall
    pub(super) heap: Heap,

    // code is hold by the first process
    // these fields will be empty for other processes
    pub(super) code_segment: Vec<PageRangeInclusive>,
    // number of pages
    pub(super) code_segment_size: u64,
}

impl ProcessVm {
    pub fn new(page_table: PageTableContext) -> Self {
        Self {
            page_table,
            stack: Stack::empty(),
            heap: Heap::empty(),
            code_segment: Vec::new(),
            code_segment_size: 0,
        }
    }

    // See the documentation for the `KernelPages` type
    // Ignore when you not reach this part

    /// Initialize kernel vm
    ///
    /// NOTE: this function should only be called by the first process
    pub fn init_kernel_vm(mut self, pages: &KernelPages) -> Self {
        // FIXME: record kernel code usage
        self.code_segment = pages.to_owned();
        self.code_segment_size = pages.iter().map(|range| range.count() as u64).sum();
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
        let heap_usage = self.heap.memory_usage();
        let code_usage = self.code_segment_size * memory::PAGE_SIZE;
        stack_usage + heap_usage + code_usage
    }

    pub fn load_elf(&mut self, elf: &ElfFile) {
        let mapper = &mut self.page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();
        self.load_elf_code(elf, mapper, alloc);

        self.stack.init(mapper, alloc);
    }

    fn load_elf_code(&mut self, elf: &ElfFile, mapper: MapperRef, alloc: FrameAllocatorRef) {
        // FIXME: load elf to process pagetable
        // map elf segments to new frames
        self.code_segment = elf::load_elf(
            elf,
            PHYSICAL_OFFSET.get().cloned().unwrap(),
            mapper,
            alloc,
            true,
        )
        .expect("Failed to load ELF code segment");

        // FIXME: calculate code usage
        self.code_segment_size = self
            .code_segment
            .iter()
            .map(|code_segment| code_segment.count() as u64)
            .sum();
    }

    pub fn fork(&self, stack_offset_count: u64) -> Self {
        // clone the page table context (see instructions)
        let owned_page_table = self.page_table.fork();

        let mapper = &mut owned_page_table.mapper();
        let alloc = &mut *get_frame_alloc_for_sure();
        Self {
            page_table: owned_page_table,
            stack: self.stack.fork(mapper, alloc, stack_offset_count),
            heap: self.heap.fork(),
            // shared code segment
            code_segment: Vec::new(),
            code_segment_size: 0,
        }
    }
    pub fn brk(&self, addr: Option<VirtAddr>) -> Option<VirtAddr> {
        self.heap.brk(
            addr,
            &mut self.page_table.mapper(),
            &mut get_frame_alloc_for_sure(),
        )
    }

    pub(super) fn clean_up(&mut self) -> Result<(), UnmapError> {
        let mapper = &mut self.page_table.mapper();
        let dealloc = &mut *get_frame_alloc_for_sure();

        // FIXME: implement the `clean_up` function for `Stack`
        self.stack.clean_up(mapper, dealloc)?;

        if self.page_table.using_count() == 1 {
            // free heap
            // FIXME: implement the `clean_up` function for `Heap`
            self.heap.clean_up(mapper, dealloc)?;

            // free code
            for page_range in self.code_segment.iter() {
                let start_addr = page_range.start.start_address().as_u64();
                let page_count = page_range.count() as u64;
                elf::unmap_range(start_addr, page_count, mapper, dealloc, true)?;
            }

            unsafe {
                // free P1-P3
                mapper.clean_up(dealloc);

                // free P4
                dealloc.deallocate_frame(self.page_table.reg.addr);
            }
        }

        // NOTE: maybe print how many frames are recycled
        //       **you may need to add some functions to `BootInfoFrameAllocator`**

        Ok(())
    }
}

impl Drop for ProcessVm {
    fn drop(&mut self) {
        if let Err(err) = self.clean_up() {
            error!("Failed to clean up process memory: {:?}", err);
        }
    }
}

impl core::fmt::Debug for ProcessVm {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (size, unit) = humanized_size(self.memory_usage());

        f.debug_struct("ProcessVm")
            .field("stack", &self.stack)
            .field("heap", &self.heap)
            .field("memory_usage", &format!("{} {}", size, unit))
            .field("page_table", &self.page_table)
            .finish()
    }
}
