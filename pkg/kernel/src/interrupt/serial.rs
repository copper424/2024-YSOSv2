use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use super::consts::*;
use crate::drivers::input;
use crate::serial::get_serial_for_sure;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8].set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    let mut char_buf = alloc::vec::Vec::new();
    loop {
        if let Some(byte) = get_serial_for_sure().receive() {
            char_buf.push(byte);
            if let Ok(key) = core::str::from_utf8(&char_buf) {
                input::push_key(key.chars().next().unwrap());
                char_buf.clear();
            }
        } else {
            break;
        }
    }
}
