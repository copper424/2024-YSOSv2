mod apic;
pub mod clock;
mod consts;
mod exceptions;
mod serial;
mod syscall;
use crate::memory::physical_to_virtual;
use apic::*;
use x86::cpuid::CpuId;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
            syscall::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();

    // FIXME: check and init APIC
    warn!("XApic support status is :{}",XApic::support() );
    if XApic::support() {
        let mut apic0 = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
        apic0.cpu_init();
    }
    // FIXME: enable serial irq with IO APIC (use enable_irq)
    // According to comments in qemu source code,
    // Serial port in the micro virtual machine is connected to IRQ 4,
    // which is mark as Serial0 in personal computer.
    enable_irq(
        consts::Irq::Serial0 as u8,
        CpuId::new()
            .get_feature_info()
            .unwrap()
            .initial_local_apic_id() as u8,
    );
    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
