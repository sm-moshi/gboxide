use super::{Mbc, MbcError};

/// MBC5: Up to 8MB ROM (512 banks), 128KB RAM (16 banks), optional rumble
/// - ROM banking: 9 bits (0x2000–0x3FFF, low 8 bits; 0x3000–0x3FFF, 9th bit)
/// - RAM banking: 4 bits (0x4000–0x5FFF)
/// - RAM enable: 0x0000–0x1FFF, 0x0A enables
/// - Rumble: 0x4000–0x5FFF, bit 3 (stub only)
pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>, // up to 128KB (16 x 8KB)
    ram_enabled: bool,
    rom_bank: u16, // 9 bits
    ram_bank: u8,  // 4 bits
                   // rumble: bool, // stub, not implemented
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>) -> Self {
        // RAM size: up to 128KB (16 banks)
        let ram_buf = vec![0; 0x20000];
        Self {
            rom,
            ram: ram_buf,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            // rumble: false,
        }
    }
}

impl Mbc for Mbc5 {
    fn read(&self, addr: u16) -> Result<u8, MbcError> {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => Ok(self.rom.get(addr as usize).copied().unwrap_or(0xFF)),
            // ROM Bank 1–511
            0x4000..=0x7FFF => {
                let bank = self.rom_bank & 0x1FF;
                let idx = (bank as usize) * 0x4000 + (addr as usize - 0x4000);
                Ok(self.rom.get(idx).copied().unwrap_or(0xFF))
            }
            // RAM
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let bank = self.ram_bank & 0x0F;
                let idx = (bank as usize) * 0x2000 + (addr as usize - 0xA000);
                Ok(*self.ram.get(idx).unwrap_or(&0xFF))
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MbcError> {
        match addr {
            // RAM Enable
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
                Ok(())
            }
            // ROM Bank Number (lower 8 bits)
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x100) | u16::from(value);
                Ok(())
            }
            // ROM Bank Number (9th bit)
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0xFF) | (u16::from(value & 0x01) << 8);
                Ok(())
            }
            // RAM Bank Number / Rumble (bit 3)
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x0F;
                // self.rumble = (value & 0x08) != 0; // stub
                Ok(())
            }
            // RAM
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let bank = self.ram_bank & 0x0F;
                let idx = (bank as usize) * 0x2000 + (addr as usize - 0xA000);
                if idx < self.ram.len() {
                    self.ram[idx] = value;
                    Ok(())
                } else {
                    Err(MbcError::InvalidRamBank(bank as usize))
                }
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn rom_bank(&self) -> usize {
        self.rom_bank as usize
    }
    fn ram_bank(&self) -> usize {
        self.ram_bank as usize
    }
    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }
    fn save_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }
    fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError> {
        if data.len() != self.ram.len() {
            return Err(MbcError::InvalidRamBank(data.len() / 0x2000));
        }
        self.ram.copy_from_slice(&data);
        Ok(())
    }
}
