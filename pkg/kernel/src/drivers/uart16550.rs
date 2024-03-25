use core::fmt;

use bitflags::bitflags;
use x86_64::instructions::port::{
    Port, PortGeneric, PortReadOnly, PortWriteOnly, ReadOnlyAccess, ReadWriteAccess,
    WriteOnlyAccess,
};

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    port: u16,
    offset_zero: PortGeneric<u8, ReadWriteAccess>,
    offset_one: PortGeneric<u8, WriteOnlyAccess>,
    int_fifo: PortGeneric<u8, ReadWriteAccess>,
    line_control: PortGeneric<u8, ReadWriteAccess>,
    modem_control: PortGeneric<u8, ReadWriteAccess>,
    line_status: PortGeneric<u8, ReadOnlyAccess>,
}
bitflags! {
    pub struct InitVal:u8{
        const disable_interrupts = 0x00;
        const enable_DLAB = 0x80;
        const divisor_lo = 0x03;
        const divisor_hi = 0x00;
        const no_parity = 0x03;
        const int_fifo = 0xC7;
        const irq_rts = 0x0B;
        const lo_mod = 0x1E;
        const test_chip = 0xAE;
    }
}
impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            offset_zero: Port::new(port),
            offset_one: PortWriteOnly::new(port + 1),
            int_fifo: Port::new(port + 2),
            line_control: Port::new(port + 3),
            modem_control: Port::new(port + 4),
            line_status: PortReadOnly::new(port + 5),
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) {
        // FIXME: Initialize the serial port
        unsafe {
            self.offset_one.write(InitVal::disable_interrupts.bits()); // Disable all interrupts
            self.line_control.write(InitVal::enable_DLAB.bits()); // Enable DLAB (set baud rate divisor)
            self.offset_zero.write(InitVal::divisor_lo.bits()); // Set divisor to 3 (lo byte) 38400 baud
            self.offset_one.write(InitVal::divisor_hi.bits()); //                   (hi byte)
            self.line_control.write(InitVal::no_parity.bits()); // 8 bits, no parity, one stop bit
            self.int_fifo.write(InitVal::int_fifo.bits()); // Enable FIFO, clear them, with 14-byte threshold
            self.modem_control.write(InitVal::irq_rts.bits()); // IRQs enabled, RTS/DSR set
            self.modem_control.write(InitVal::lo_mod.bits()); // Set in loopback mode, test the serial chip
            self.offset_zero.write(InitVal::test_chip.bits()); // Test serial chip (send byte 0xAE and check if serial returns same byte)
        }
        // Check if serial is faulty (i.e: not same byte as sent)
        if unsafe { self.offset_zero.read() } != 0xAE {
            return;
        }
        // If serial is not faulty set it in normal operation mode
        // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
        unsafe {
            self.modem_control.write(0x0F);
        }
        // Enable all interrupts
        unsafe {
            self.offset_one.write(0x01);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        while (unsafe { self.line_status.read() } & 0x20) == 0 {}
        unsafe {
            self.offset_zero.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        if (unsafe { self.line_status.read() } & 1) == 0 {
            return None;
        } else {
            return Some(unsafe { self.offset_zero.read() });
        }
    }

    pub fn backspace(&mut self) {
        self.send(0x08);
        self.send(0x20);
        self.send(0x08);
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
