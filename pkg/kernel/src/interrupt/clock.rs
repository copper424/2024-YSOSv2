use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{memory::gdt, proc::ProcessContext};

use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8].set_handler_fn(clock_handler);
}

pub extern "C" fn clock(mut context: ProcessContext) {
    // debug!("begin to switch context\n");
    crate::proc::switch(&mut context);
    super::ack();
}

as_handler!(clock);
