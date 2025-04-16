use crate::bus::MemoryBus;

/// Temporary flat 32K memory for testing
pub struct MMU {
    wram: [u8; 0x8000],
}

impl Default for MMU {
    fn default() -> Self {
        Self::new()
    }
}

impl MMU {
    pub fn new() -> Self {
        MMU { wram: [0; 0x8000] }
    }
}

impl MemoryBus for MMU {
    fn read(&mut self, addr: u16) -> u8 {
        self.wram[addr as usize % 0x8000]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.wram[addr as usize % 0x8000] = value;
    }
}
