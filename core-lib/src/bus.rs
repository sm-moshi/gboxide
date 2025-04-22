/// core-lib/src/bus.rs
use crate::interrupts::InterruptFlag;
use crate::mmu::{MemoryBusTrait, MMU};

/// Trait for memory access abstraction across the system
pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    /// Read a 16-bit value from memory in little-endian format
    fn read_word(&self, addr: u16) -> u16 {
        let lo = u16::from(self.read(addr));
        let hi = u16::from(self.read(addr.wrapping_add(1)));
        (hi << 8) | lo
    }

    /// Write a 16-bit value to memory in little-endian format
    fn write_word(&mut self, addr: u16, value: u16) {
        let lo = (value & 0xFF) as u8;
        let hi = (value >> 8) as u8;
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }

    fn get_interrupt(&self) -> Option<InterruptFlag>;
    fn clear_interrupt(&mut self, flag: InterruptFlag);
    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16;
}

/// Implementation of `MemoryBus` that wraps around the MMU
pub struct Bus<'a> {
    mmu: &'a mut MMU,
}

impl<'a> Bus<'a> {
    pub fn new(mmu: &'a mut MMU) -> Self {
        Self { mmu }
    }
}

impl MemoryBus for Bus<'_> {
    fn read(&self, addr: u16) -> u8 {
        self.mmu.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        let _ = self.mmu.write(addr, value);
    }

    fn get_interrupt(&self) -> Option<InterruptFlag> {
        self.mmu.get_interrupt()
    }

    fn clear_interrupt(&mut self, flag: InterruptFlag) {
        self.mmu.clear_interrupt(flag);
    }

    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16 {
        self.mmu.get_interrupt_vector(flag)
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = u16::from(self.read(addr));
        let hi = u16::from(self.read(addr.wrapping_add(1)));
        (hi << 8) | lo
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        let lo = (value & 0xFF) as u8;
        let hi = (value >> 8) as u8;
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }
}
