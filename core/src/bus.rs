/// Memory interface abstraction between CPU and memory/IO
use crate::mmu::MMU;

pub trait MemoryBusTrait {
    fn read8(&mut self, addr: u16) -> u8;
    fn write8(&mut self, addr: u16, value: u8);
    fn read16(&mut self, addr: u16) -> u16 {
        let lo = self.read8(addr) as u16;
        let hi = self.read8(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }
    fn write16(&mut self, addr: u16, value: u16) {
        let lo = value as u8;
        let hi = (value >> 8) as u8;
        self.write8(addr, lo);
        self.write8(addr.wrapping_add(1), hi);
    }
}

pub struct MemoryBus<'a> {
    pub mmu: &'a mut MMU,
}

impl<'a> MemoryBus<'a> {
    pub fn new(mmu: &'a mut MMU) -> Self {
        Self { mmu }
    }
}

impl<'a> MemoryBusTrait for MemoryBus<'a> {
    fn read8(&mut self, addr: u16) -> u8 {
        self.mmu.read(addr)
    }

    fn write8(&mut self, addr: u16, value: u8) {
        self.mmu.write(addr, value);
    }
}
