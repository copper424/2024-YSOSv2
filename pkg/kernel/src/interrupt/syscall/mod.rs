use crate::{memory::gdt::SYSCALL_IST_INDEX, proc::*};

use x86_64::{
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
    PrivilegeLevel,
};

use crate::syscall::dispatcher;
pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    // FIXME: register syscall handler to IDT
    //        - standalone syscall stack
    //        - ring 3
    idt[super::consts::Interrupts::Syscall as u8]
        .set_handler_fn(syscall_handler)
        .set_stack_index(SYSCALL_IST_INDEX as u16)
        .set_privilege_level(PrivilegeLevel::Ring3);
}

pub extern "C" fn syscall(mut context: ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        super::syscall::dispatcher(&mut context);
    });
}

as_handler!(syscall);
