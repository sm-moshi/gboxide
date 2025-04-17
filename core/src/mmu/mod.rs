use crate::bus::MemoryBus;

pub struct MMU {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
}

impl MMU {
    pub fn new() -> Self {
        Self {
            rom: vec![0; 0x8000], // Default 32KB ROM
            ram: vec![0; 0x8000], // Up to 32KB external RAM
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
        }
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.rom = data;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => {
                let offset = self.rom_bank * 0x4000;
                self.rom[offset + (addr as usize - 0x4000)]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let offset = self.ram_bank * 0x2000;
                    self.ram[offset + (addr as usize - 0xA000)]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                let bank = (value as usize) & 0x1F;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => {
                self.ram_bank = (value as usize) & 0x03;
            }
            0x6000..=0x7FFF => {
                // Banking mode switch (ignored for now)
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let offset = self.ram_bank * 0x2000;
                    self.ram[offset + (addr as usize - 0xA000)] = value;
                }
            }
            _ => {}
        }
    }
}

// Correct trait implementation
impl MemoryBus for MMU {
    fn read8(&mut self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn write8(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }
}