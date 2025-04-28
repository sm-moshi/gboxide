use super::{Mbc, MbcError};

/// MBC1: Most common MBC type
/// Supports up to 2MB ROM and 32KB RAM
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank_low: u8,
    rom_bank_high: u8,
    ram_bank: u8,
    mode: u8,
    rom_mask: usize,
    ram_base_offset: usize, // Precomputed RAM base offset for current bank
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Self {
        // Calculate ROM size mask based on actual ROM size
        let rom_bank_count = (rom.len() / 0x4000).next_power_of_two();
        let rom_addr_mask = rom_bank_count - 1;
        // Calculate RAM size (default to 32KB)
        let ram_total_size = 0x8000; // 32KB
        Self {
            rom,
            ram: vec![0; ram_total_size],
            ram_enabled: false,
            rom_bank_low: 1,
            rom_bank_high: 0,
            ram_bank: 0,
            mode: 0,
            rom_mask: rom_addr_mask,
            ram_base_offset: 0, // Start at bank 0
        }
    }
    const fn current_rom_bank(&self) -> usize {
        let mut bank = ((self.rom_bank_high as usize) << 5) | (self.rom_bank_low as usize);
        if bank.trailing_zeros() >= 5 {
            bank |= 1;
        }
        bank & self.rom_mask
    }
    const fn current_ram_bank(&self) -> usize {
        if self.mode == 0 {
            0
        } else {
            self.ram_bank as usize
        }
    }
    fn update_ram_base_offset(&mut self) {
        // Always called after ram_bank or mode changes
        let bank = self.current_ram_bank();
        debug_assert!(bank * 0x2000 < self.ram.len(), "RAM bank out of bounds");
        self.ram_base_offset = bank * 0x2000;
    }
}

impl Mbc for Mbc1 {
    fn read(&self, addr: u16) -> Result<u8, MbcError> {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => {
                let bank = if self.mode == 0 {
                    0
                } else {
                    self.rom_bank_high as usize
                };
                let idx = bank * 0x4000 + addr as usize;
                Ok(self.rom.get(idx).copied().unwrap_or(0xFF))
            }
            // ROM Bank 1-N
            0x4000..=0x7FFF => {
                let bank = self.current_rom_bank();
                let idx = bank * 0x4000 + (addr as usize - 0x4000);
                Ok(self.rom.get(idx).copied().unwrap_or(0xFF))
            }
            // RAM Banks
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let idx = self.ram_base_offset + (addr as usize - 0xA000);
                debug_assert!(idx < self.ram.len(), "RAM access out of bounds");
                // SAFETY: idx is always valid due to masking and precomputed offset
                Ok(unsafe { *self.ram.get_unchecked(idx) })
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
            // ROM Bank Number (Low 5 bits)
            0x2000..=0x3FFF => {
                let mut bank = value & 0x1F;
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank_low = bank;
                Ok(())
            }
            // ROM/RAM Bank Number (High 2 bits)
            0x4000..=0x5FFF => {
                let value = value & 0x03;
                if self.mode == 0 {
                    self.rom_bank_high = value;
                } else {
                    self.ram_bank = value;
                    self.update_ram_base_offset();
                }
                Ok(())
            }
            // Banking Mode Select
            0x6000..=0x7FFF => {
                self.mode = value & 0x01;
                self.update_ram_base_offset();
                Ok(())
            }
            // RAM Bank Write
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let idx = self.ram_base_offset + (addr as usize - 0xA000);
                debug_assert!(idx < self.ram.len(), "RAM write out of bounds");
                // SAFETY: idx is always valid due to masking and precomputed offset
                unsafe {
                    *self.ram.get_unchecked_mut(idx) = value;
                }
                Ok(())
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn rom_bank(&self) -> usize {
        self.current_rom_bank()
    }
    fn ram_bank(&self) -> usize {
        self.current_ram_bank()
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
        self.ram = data;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_rom(size: usize) -> Vec<u8> {
        let mut rom = vec![0u8; size];
        // Fill each bank with a unique value for identification
        for (i, chunk) in rom.chunks_mut(0x4000).enumerate() {
            for b in chunk.iter_mut() {
                *b = i as u8;
            }
        }
        rom
    }

    #[test]
    fn test_ram_disabled_error() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc1::new(rom);
        // RAM is disabled by default
        let result = mbc.write(0xA000, 0x42);
        assert!(matches!(result, Err(MbcError::RamDisabled)));
    }

    #[test]
    fn test_protection_violation_error() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc1::new(rom);
        // Access address outside valid range
        let result = mbc.write(0xF000, 0x12);
        assert!(matches!(result, Err(MbcError::ProtectionViolation(_))));
    }

    #[test]
    fn test_load_ram_wrong_size() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc1::new(rom);
        // Try to load RAM with wrong size
        let result = mbc.load_ram(vec![0u8; 0x1000]);
        assert!(matches!(result, Err(MbcError::InvalidRamBank(_))));
    }

    #[test]
    fn test_rom_bank_zero_maps_to_one() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc1::new(rom);
        // Set ROM bank to 0 (should map to 1)
        mbc.write(0x2000, 0x00).unwrap();
        assert_eq!(mbc.current_rom_bank(), 1);
    }

    #[test]
    fn test_mode_switching_and_ram_bank() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc1::new(rom);
        // Enable RAM
        mbc.write(0x0000, 0x0A).unwrap();
        // Set mode 1 (RAM banking)
        mbc.write(0x6000, 0x01).unwrap();
        // Set RAM bank to 1
        mbc.write(0x4000, 0x01).unwrap();
        assert_eq!(mbc.current_ram_bank(), 1);
        // Set mode 0 (ROM banking)
        mbc.write(0x6000, 0x00).unwrap();
        // RAM bank should be 0 in mode 0
        assert_eq!(mbc.current_ram_bank(), 0);
    }
}
