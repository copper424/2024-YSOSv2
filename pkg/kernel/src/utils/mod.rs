#[macro_use]
mod macros;
#[macro_use]
mod regs;

// pub mod clock;
pub mod func;
pub mod logger;

pub use macros::*;
pub use regs::*;
pub mod resource;
pub mod uefi_runtime;
use crate::proc::*;

pub const fn get_ascii_header() -> &'static str {
    concat!(
        r"
__  __      __  _____            ____  _____
\ \/ /___ _/ /_/ ___/___  ____  / __ \/ ___/
 \  / __ `/ __/\__ \/ _ \/ __ \/ / / /\__ \
 / / /_/ / /_ ___/ /  __/ / / / /_/ /___/ /
/_/\__,_/\__//____/\___/_/ /_/\____//____/

                                       v",
        env!("CARGO_PKG_VERSION")
    )
}

// pub fn new_test_thread(id: &str) -> ProcessId {
//     let mut proc_data = ProcessData::new();
//     proc_data.set_env("id", id);

//     spawn_kernel_thread(
//         crate::utils::func::test,
//         format!("#{}_test", id),
//         Some(proc_data),
//     )
// }

// pub fn new_stack_test_thread() {
//     let pid = spawn_kernel_thread(
//         crate::utils::func::stack_test,
//         alloc::string::String::from("stack"),
//         None,
//     );

//     // wait for progress exit
//     wait(pid);
// }

fn wait(pid: ProcessId) {
    loop {
        // FIXME: try to get the status of the process
        let exit_code = crate::proc::get_proc_exit_code(pid);
        // HINT: it's better to use the exit code

        if exit_code.is_none() {
            x86_64::instructions::hlt();
        } else {
            debug!("Process #{} exit with code: {}", pid, exit_code.unwrap());
            break;
        }
    }
}
