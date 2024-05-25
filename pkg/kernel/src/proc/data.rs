use alloc::collections::BTreeMap;
use spin::RwLock;

use crate::resource::ResourceSet;

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,
    pub(super) resource: Arc<RwLock<ResourceSet>>,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            resource: Arc::new(RwLock::new(ResourceSet::default())),
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

    pub fn sys_read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resource.read().read(fd, buf)
    }

    pub fn sys_write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resource.read().write(fd, buf)
    }
}
