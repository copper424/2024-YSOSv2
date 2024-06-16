use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;
use x86_64::{
    structures::paging::{mapper::UnmapError, Page, Size4KiB},
    VirtAddr,
};

use crate::proc::{processor, KERNEL_PID};

use super::{FrameAllocatorRef, MapperRef};

// user process runtime heap
// 0x100000000 bytes -> 4GiB
// from 0x0000_2000_0000_0000 to 0x0000_2000_ffff_fff8
pub const HEAP_START: u64 = 0x2000_0000_0000;
pub const HEAP_PAGES: u64 = 0x100000;
pub const HEAP_SIZE: u64 = HEAP_PAGES * crate::memory::PAGE_SIZE;
pub const HEAP_END: u64 = HEAP_START + HEAP_SIZE - 8;

/// User process runtime heap
///
/// always page aligned, the range is [base, end)
pub struct Heap {
    /// the base address of the heap
    ///
    /// immutable after initialization
    base: VirtAddr,

    /// the current end address of the heap
    ///
    /// use atomic to allow multiple threads to access the heap
    end: Arc<AtomicU64>,
}

impl Heap {
    pub fn empty() -> Self {
        Self {
            base: VirtAddr::new(HEAP_START),
            end: Arc::new(AtomicU64::new(HEAP_START)),
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            base: self.base,
            end: self.end.clone(),
        }
    }

    pub fn brk(
        &self,
        new_end: Option<VirtAddr>,
        mapper: MapperRef,
        alloc: FrameAllocatorRef,
    ) -> Option<VirtAddr> {
        // FIXME: if new_end is None, return the current end address
        if new_end.is_none() {
            return Some(VirtAddr::new(self.end.load(Ordering::SeqCst)));
        }
        // FIXME: check if the new_end is valid (in range [base, base + HEAP_SIZE])
        let new_end = new_end.unwrap();
        if new_end.as_u64() < HEAP_START || new_end.as_u64() > HEAP_END {
            return None;
        }
        // FIXME: calculate the difference between the current end and the new end
        let diff = new_end.as_u64() - self.end.load(Ordering::SeqCst);
        // NOTE: print the heap difference for debugging
        debug!("Heap diff: {:#x}", diff);
        // FIXME: do the actual mapping or unmapping
        let user_access = processor::get_pid() != KERNEL_PID;
        if diff > 0 {
            // map the new pages
            let start: Page<Size4KiB> =
                Page::containing_address(VirtAddr::new(self.end.load(Ordering::SeqCst)));
            let end: Page<Size4KiB> = Page::containing_address(new_end);
            let count = end - start;

            match elf::map_range(
                start.start_address().as_u64(),
                count,
                mapper,
                alloc,
                user_access,
            ) {
                Ok(range) => {
                    debug!(
                        "map heap ranging from {:#?} to {:#?}",
                        range.start, range.end
                    );
                }
                Err(e) => {
                    debug!("Failed to map heap: {:?}", e);
                    return None;
                }
            }
        } else {
            // unmap the pages
            let start: Page<Size4KiB> = Page::containing_address(new_end);
            let end: Page<Size4KiB> =
                Page::containing_address(VirtAddr::new(self.end.load(Ordering::SeqCst)));
            let count = end - start;
            if let Err(e) =
                elf::unmap_range(start.start_address().as_u64(), count, mapper, alloc, true)
            {
                debug!("Failed to unmap heap: {:?}", e);
                return None;
            }
        }
        // FIXME: update the end address
        self.end.store(new_end.as_u64(), Ordering::SeqCst);
        Some(new_end)
    }

    pub(super) fn clean_up(
        &self,
        mapper: MapperRef,
        dealloc: FrameAllocatorRef,
    ) -> Result<(), UnmapError> {
        if self.memory_usage() == 0 {
            return Ok(());
        }

        // FIXME: load the current end address and **reset it to base** (use `swap`)
        let end = self.end.swap(HEAP_START, Ordering::Relaxed);
        // FIXME: unmap the heap pages
        let start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(HEAP_START));
        let end_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(end));
        let count = end_page - start_page;
        elf::unmap_range(HEAP_START, count, mapper, dealloc, true)?;
        Ok(())
    }

    pub fn memory_usage(&self) -> u64 {
        self.end.load(Ordering::Relaxed) - self.base.as_u64()
    }
}

impl core::fmt::Debug for Heap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Heap")
            .field("base", &format_args!("{:#x}", self.base.as_u64()))
            .field(
                "end",
                &format_args!("{:#x}", self.end.load(Ordering::Relaxed)),
            )
            .finish()
    }
}
