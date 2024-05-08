use boot::{BootInfo, RuntimeServices, Time};

once_mutex!(pub UEFI_RUNTIME:UefiRuntime);

pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        init_UEFI_RUNTIME(UefiRuntime::new(boot_info));
    }
}

pub struct UefiRuntime {
    runtime_service: &'static RuntimeServices,
}

impl UefiRuntime {
    pub unsafe fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            runtime_service: boot_info.system_table.runtime_services(),
        }
    }

    pub fn get_time(&self) -> Time {
        self.runtime_service.get_time().unwrap()
    }
}
