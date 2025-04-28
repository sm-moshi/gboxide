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
    pub const fn new(mmu: &'a mut MMU) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interrupts::InterruptFlag;

    fn create_test_mmu() -> MMU {
        // 32KB dummy ROM, MBC1 type, 8KB RAM
        let mut rom = vec![0; 0x8000];
        rom[0x147] = 0x01; // MBC1
        rom[0x148] = 0x00; // 32KB ROM
        rom[0x149] = 0x02; // 8KB RAM
        MMU::new(rom).unwrap()
    }

    #[test]
    fn test_bus_new_and_read_write() {
        let mut mmu = create_test_mmu();
        let mut bus = Bus::new(&mut mmu);
        // Write and read a value
        bus.write(0xC000, 0x42);
        assert_eq!(bus.read(0xC000), 0x42);
    }

    #[test]
    fn test_read_write_word() {
        let mut mmu = create_test_mmu();
        let mut bus = Bus::new(&mut mmu);
        bus.write_word(0xC000, 0xBEEF);
        assert_eq!(bus.read_word(0xC000), 0xBEEF);
    }

    #[test]
    fn test_get_and_clear_interrupt() {
        let mut mmu = create_test_mmu();
        let mut bus = Bus::new(&mut mmu);
        bus.write(0xFF0F, 0x01);
        bus.write(0xFFFF, 0x01);
        drop(bus); // Release borrow on mmu before mutably borrowing interrupts
        mmu.interrupts.borrow_mut().ime = true;
        let mut bus = Bus::new(&mut mmu);
        assert_eq!(bus.get_interrupt(), Some(InterruptFlag::VBlank));
        // Clear the interrupt
        bus.clear_interrupt(InterruptFlag::VBlank);
        assert_eq!(bus.get_interrupt(), None);
    }

    #[test]
    fn test_get_interrupt_vector() {
        let mut mmu = create_test_mmu();
        let bus = Bus::new(&mut mmu);
        // VBlank interrupt vector is 0x40
        assert_eq!(bus.get_interrupt_vector(InterruptFlag::VBlank), 0x40);
    }

    #[test]
    fn test_bus_new_constructor() {
        let mut mmu = create_test_mmu();
        let _bus = Bus::new(&mut mmu);
    }

    #[test]
    fn test_memorybus_trait_methods() {
        let mut mmu = create_test_mmu();
        let mut bus = Bus::new(&mut mmu);
        // Use trait object to call default methods
        let bus_trait: &mut dyn MemoryBus = &mut bus;
        bus_trait.write_word(0xC100, 0xCAFE);
        assert_eq!(bus_trait.read_word(0xC100), 0xCAFE);
    }

    struct DummyBus {
        mem: [u8; 4],
    }
    impl MemoryBus for DummyBus {
        fn read(&self, addr: u16) -> u8 {
            self.mem[addr as usize % 4]
        }
        fn write(&mut self, addr: u16, value: u8) {
            self.mem[addr as usize % 4] = value;
        }
        fn get_interrupt(&self) -> Option<InterruptFlag> {
            None
        }
        fn clear_interrupt(&mut self, _flag: InterruptFlag) {}
        fn get_interrupt_vector(&self, _flag: InterruptFlag) -> u16 {
            0
        }
    }

    #[test]
    fn test_memorybus_trait_default_methods() {
        let mut bus = DummyBus { mem: [0; 4] };
        // Test default read_word and write_word
        bus.write_word(0, 0xBEEF);
        assert_eq!(bus.read_word(0), 0xBEEF);
    }
}
