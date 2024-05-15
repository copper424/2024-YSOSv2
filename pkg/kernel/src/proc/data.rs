use alloc::collections::BTreeMap;
use spin::rwlock::RwLock;
use x86_64::structures::paging::{page::PageRange, Page};

use crate::resource::StdIO;

use self::sync::SemaphoreSet;

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,
    pub(super) file_handles: Arc<RwLock<BTreeMap<u8, Resource>>>,
    pub(super) semaphores: Arc<RwLock<SemaphoreSet>>,
    // process specific data
    pub(super) stack_segment: Option<PageRange>,
    pub(super) code_segment: Option<Vec<PageRange>>,
    pub(super) stack_pages: usize,
    pub(super) code_pages: usize,
}

impl Default for ProcessData {
    fn default() -> Self {
        let mut file_handles = BTreeMap::new();

        // stdin, stdout, stderr
        file_handles.insert(0, Resource::Console(StdIO::Stdin));
        file_handles.insert(1, Resource::Console(StdIO::Stdout));
        file_handles.insert(2, Resource::Console(StdIO::Stderr));
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            file_handles: Arc::new(RwLock::new(file_handles)),
            semaphores: Arc::new(RwLock::new(SemaphoreSet::default())),
            stack_segment: None,
            code_segment: None,
            stack_pages: 0,
            code_pages: 0,
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        self.stack_segment = Some(Page::range(start, start + size));
    }

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        // FIXME: check if the address is on the stack
        if let Some(segment) = self.stack_segment {
            let stack_top = segment.start.start_address();
            // debug!("stack: {:#x?} - {:#x?}\n", front, last);
            // debug!("addr: {:#x?}\n", addr);
            if addr.as_u64() & STACK_START_MASK == stack_top.as_u64() & STACK_START_MASK {
                return true;
            }
        }
        false
    }
    pub fn handle(&self, fd: u8) -> Option<Resource> {
        self.file_handles.read().get(&fd).cloned()
    }
}
