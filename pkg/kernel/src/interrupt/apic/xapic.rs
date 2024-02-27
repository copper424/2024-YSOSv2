use crate::interrupt::consts::{Interrupts, Irq};

use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

pub struct XApic {
    addr: u64,
}

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.addr + reg as u64) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        write_volatile((self.addr + reg as u64) as *mut u32, value);
        self.read(0x20);
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXME: Check CPUID to see if xAPIC is supported.
        CpuId::new()
            .get_feature_info()
            .map(|finfo| finfo.has_apic())
            .unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // FIXME: Enable local APIC; set spurious interrupt vector.
            const SPIV: u32 = 0xF0;
            let mut spiv_value = self.read(SPIV);
            spiv_value |= 1 << 8;
            spiv_value &= !(0xFF);
            spiv_value |= Interrupts::IrqBase as u32 + Irq::Spurious as u32;
            self.write(SPIV, spiv_value);

            // FIXME: The timer repeatedly counts down at bus frequency
            const LVT_TIMER: u32 = 0x320;
            let mut lvt_timer = self.read(LVT_TIMER);
            lvt_timer &= !(0xFF);
            lvt_timer |= Interrupts::IrqBase as u32 + Irq::Timer as u32;
            lvt_timer &= !(1 << 16);
            lvt_timer |= 1 << 17;
            self.write(LVT_TIMER, lvt_timer);

            // Initialization configuration register for timer
            const TICR: u32 = 0x380;
            // Divide configuration register for timer
            const TDCR: u32 = 0x3E0;
            self.write(TDCR, 0b1011);
            self.write(TICR, 0x20000);

            // FIXME: Disable logical interrupt lines (LINT0, LINT1)
            const LINT0: u32 = 0x350;
            const LINT1: u32 = 0x360;
            self.write(LINT0, 1 << 16);
            self.write(LINT1, 1 << 16);

            // FIXME: Disable performance counter overflow interrupts (PCINT)
            const PCINT: u32 = 0x340;
            self.write(PCINT, 1 << 16);

            // FIXME: Map error interrupt to IRQ_ERROR.
            const LVT_ERR: u32 = 0x370;
            let mut lvt_error = self.read(LVT_ERR);
            // ?
            lvt_error &= !(0xFF);
            lvt_error |= Interrupts::IrqBase as u32 + Irq::Error as u32;
            self.write(LVT_ERR, lvt_error);

            // FIXME: Clear error status register (requires back-to-back writes).
            // Error status register
            const ESR: u32 = 0x280;
            self.write(ESR, 0);
            self.write(ESR, 0);

            // FIXME: Ack any outstanding interrupts.
            // const EOI: u32 = 0xB0;
            // self.write(EOI, 0);
            self.eoi();

            // FIXME: Send an Init Level De-Assert to synchronise arbitration ID's.
            const ICR_0: u32 = 0x300;
            const ICR_1: u32 = 0x310;
            self.write(ICR_1, 0);
            const BCAST_INIT: u32 = 1 << 19;
            const INIT_DE_ASSERT_MODE: u32 = 5 << 8;
            const TRIG_MODE_LEVEL: u32 = 1 << 15;
            self.write(ICR_0, BCAST_INIT | INIT_DE_ASSERT_MODE | TRIG_MODE_LEVEL);
            const DS: u32 = 1 << 12;
            while self.read(ICR_0) & DS != 0 {}

            // FIXME: Enable interrupts on the APIC (but not on the processor).
            const TPR: u32 = 0x80;
            self.write(TPR, 0);
        }

        // NOTE: Try to use bitflags! macro to set the flags.
    }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
