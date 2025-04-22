use super::{Mbc, MbcError};

/// MBC2: 256KB ROM, 512x4bit internal RAM (A000-A1FF)
/// - Up to 16 ROM banks (0x4000-0x7FFF)
/// - 512x4bit internal RAM (A000-A1FF, only lower 4 bits used)
/// - RAM enable: 0x0000-0x1FFF, only if (addr & 0x0100 == 0)
/// - ROM bank select: 0x2000-0x3FFF, only if (addr & 0x0100 == 0x0100)
/// - No external RAM
pub struct Mbc2 {
    rom: Vec<u8>,
    ram: [u8; 512], // 512 x 4 bits (lower nibble used)
    ram_enabled: bool,
    rom_bank: u8, // 4 bits
}

impl Mbc2 {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: [0x0F; 512], // Uninitialized RAM is 0x0F (all bits set)
            ram_enabled: false,
            rom_bank: 1, // Bank 1 is default after reset
        }
    }
}

impl Mbc for Mbc2 {
    fn read(&self, addr: u16) -> Result<u8, MbcError> {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => Ok(self.rom.get(addr as usize).copied().unwrap_or(0xFF)),
            // ROM Bank 1-15
            0x4000..=0x7FFF => {
                let bank = self.rom_bank & 0x0F;
                let idx = (bank as usize) * 0x4000 + (addr as usize - 0x4000);
                Ok(self.rom.get(idx).copied().unwrap_or(0xFF))
            }
            // Internal RAM (A000-A1FF)
            0xA000..=0xA1FF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let idx = (addr - 0xA000) as usize;
                Ok(self.ram[idx] | 0xF0) // upper nibble is open bus (usually 0xF0)
            }
            // Unused RAM area (A200-BFFF)
            0xA200..=0xBFFF => Ok(0xFF),
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MbcError> {
        match addr {
            // RAM Enable (only if addr & 0x0100 == 0)
            0x0000..=0x1FFF if addr & 0x0100 == 0 => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
                Ok(())
            }
            // ROM Bank Number (only if addr & 0x0100 == 0x0100)
            0x2000..=0x3FFF if addr & 0x0100 == 0x0100 => {
                let mut bank = value & 0x0F;
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank = bank;
                Ok(())
            }
            // Internal RAM (A000-A1FF)
            0xA000..=0xA1FF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let idx = (addr - 0xA000) as usize;
                self.ram[idx] = value & 0x0F; // Only lower 4 bits are stored
                Ok(())
            }
            // Unused RAM area (A200-BFFF)
            0xA200..=0xBFFF => Ok(()),
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn rom_bank(&self) -> usize {
        self.rom_bank as usize
    }
    fn ram_bank(&self) -> usize {
        0 // Only one RAM bank
    }
    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }
    fn save_ram(&self) -> Vec<u8> {
        self.ram.to_vec()
    }
    fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError> {
        if data.len() != 512 {
            return Err(MbcError::InvalidRamBank(data.len()));
        }
        self.ram.copy_from_slice(&data[..512]);
        Ok(())
    }
}
