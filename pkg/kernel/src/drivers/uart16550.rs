use core::fmt;

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
            self.offset_one.write(0x00); // Disable all interrupts
            self.line_control.write(0x80); // Enable DLAB (set baud rate divisor)
            self.offset_zero.write(0x03); // Set divisor to 3 (lo byte) 38400 baud
            self.offset_one.write(0x00); //                   (hi byte)
            self.line_control.write(0x03); // 8 bits, no parity, one stop bit
            self.int_fifo.write(0xC7); // Enable FIFO, clear them, with 14-byte threshold
            self.modem_control.write(0x0B); // IRQs enabled, RTS/DSR set
            self.modem_control.write(0x1E); // Set in loopback mode, test the serial chip
            self.offset_zero.write(0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)
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
        unsafe{
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
        while (unsafe { self.line_status.read() } & 1) == 0 {}
        unsafe { Some(self.offset_zero.read()) }
    }
    
    pub fn backspace(&mut self){
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
