/// core-lib/src/bus.rs
use crate::mmu::MMU;

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
        let lo = u8::try_from(value).unwrap_or(0);
        let hi = u8::try_from(value >> 8).unwrap_or(0);
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }
}

/// Implementation of `MemoryBus` that wraps around the MMU
pub struct Bus<'a> {
    mmu: &'a mut MMU,
}

impl<'a> Bus<'a> {
    pub fn new(mmu: &'a mut MMU) -> Bus<'a> {
        Bus { mmu }
    }
}

impl<'a> MemoryBus for Bus<'a> {
    fn read_word(&self, addr: u16) -> u16 {
        let lo = u16::from(self.read(addr));
        let hi = u16::from(self.read(addr.wrapping_add(1)));
        (hi << 8) | lo
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        let lo = u8::try_from(value).unwrap_or(0);
        let hi = u8::try_from(value >> 8).unwrap_or(0);
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }

    fn read(&self, addr: u16) -> u8 {
        self.mmu.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.mmu.write(addr, value);
    }
}
