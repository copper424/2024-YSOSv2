use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use super::consts::*;
use crate::drivers::input;
use crate::serial::get_serial;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as usize + Irq::Serial0 as usize].set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    if let Some(key) = get_serial().unwrap().receive() {
        debug!("Receive key: {:?}\n", key);
        input::push_key(key as char);
    }else{
        debug!("Do not receive key\n");
    }
}
