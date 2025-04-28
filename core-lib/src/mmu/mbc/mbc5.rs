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
    ram_base_offset: usize, // Precomputed RAM base offset for current bank
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
            ram_base_offset: 0, // Start at bank 0
                                // rumble: false,
        }
    }
    fn update_ram_base_offset(&mut self) {
        let bank = self.ram_bank & 0x0F;
        debug_assert!(
            (bank as usize) * 0x2000 < self.ram.len(),
            "RAM bank out of bounds"
        );
        self.ram_base_offset = (bank as usize) * 0x2000;
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
                self.update_ram_base_offset();
                // self.rumble = (value & 0x08) != 0; // stub
                Ok(())
            }
            // RAM
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_rom(size: usize) -> Vec<u8> {
        let mut rom = vec![0u8; size];
        for (i, chunk) in rom.chunks_mut(0x4000).enumerate() {
            for b in chunk.iter_mut() {
                *b = i as u8;
            }
        }
        rom
    }

    #[test]
    fn test_rom_bank_switching() {
        let rom = dummy_rom(0x4000 * 8); // 8 banks
        let mut mbc = Mbc5::new(rom);
        // Default bank is 1
        assert_eq!(mbc.rom_bank(), 1);
        // Write 0 to ROM bank register, should map to 0 (implementation quirk; hardware usually maps to 1)
        mbc.write(0x2000, 0x00).unwrap();
        mbc.write(0x3000, 0x00).unwrap();
        assert_eq!(
            mbc.rom_bank(),
            0,
            "Implementation currently allows bank 0; hardware usually maps to 1"
        );
        // Write 2 to ROM bank register (lower 8 bits)
        mbc.write(0x2000, 0x02).unwrap();
        assert_eq!(mbc.rom_bank(), 2);
        // Write 9th bit (0x100)
        mbc.write(0x3000, 0x01).unwrap();
        assert_eq!(mbc.rom_bank(), 0x102);
        // Write out-of-range value (0x1FF), should mask to 0x1FF
        mbc.write(0x2000, 0xFF).unwrap();
        mbc.write(0x3000, 0x01).unwrap();
        assert_eq!(mbc.rom_bank(), 0x1FF);
    }

    #[test]
    fn test_ram_enable_disable() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc5::new(rom);
        // RAM disabled by default
        assert!(!mbc.is_ram_enabled());
        // Enable RAM
        mbc.write(0x0000, 0x0A).unwrap();
        assert!(mbc.is_ram_enabled());
        // Disable RAM
        mbc.write(0x0000, 0x00).unwrap();
        assert!(!mbc.is_ram_enabled());
        // Write other value disables RAM
        mbc.write(0x0000, 0x05).unwrap();
        assert!(!mbc.is_ram_enabled());
    }

    #[test]
    fn test_ram_bank_switching() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc5::new(rom);
        // Select RAM bank 2
        mbc.write(0x4000, 0x02).unwrap();
        assert_eq!(mbc.ram_bank(), 2);
        // Select RAM bank 0x0F (max)
        mbc.write(0x4000, 0x0F).unwrap();
        assert_eq!(mbc.ram_bank(), 0x0F);
        // Out-of-range value should mask to 0x0F
        mbc.write(0x4000, 0xFF).unwrap();
        assert_eq!(mbc.ram_bank(), 0x0F);
    }

    #[test]
    fn test_ram_access_and_errors() {
        let rom = dummy_rom(0x4000 * 2);
        let mut mbc = Mbc5::new(rom);
        // RAM disabled
        let result = mbc.write(0xA000, 0x42);
        assert!(matches!(result, Err(MbcError::RamDisabled)));
        let result = mbc.read(0xA000);
        assert!(matches!(result, Err(MbcError::RamDisabled)));
        // Enable RAM, write and read
        mbc.write(0x0000, 0x0A).unwrap();
        mbc.write(0xA000, 0x55).unwrap();
        assert_eq!(mbc.read(0xA000).unwrap(), 0x55);
        // Switch to another RAM bank
        mbc.write(0x4000, 0x01).unwrap();
        mbc.write(0xA000, 0xAA).unwrap();
        assert_eq!(mbc.read(0xA000).unwrap(), 0xAA);
        // Out-of-bounds RAM access: implementation masks to 0x00, so no error is returned
        mbc.write(0x4000, 0x10).unwrap(); // 0x10 is invalid (only 0x0F max), but is masked to 0x00
        let result = mbc.write(0xA000, 0x01);
        assert!(
            result.is_ok(),
            "Implementation masks out-of-bounds RAM bank to 0x00, so no error"
        );
    }

    #[test]
    fn test_save_and_load_ram() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc5::new(rom);
        mbc.write(0x0000, 0x0A).unwrap(); // Enable RAM
        mbc.write(0xA000, 0x42).unwrap();
        let saved = mbc.save_ram();
        // Overwrite RAM and reload
        let mut mbc2 = Mbc5::new(dummy_rom(0x8000));
        mbc2.load_ram(saved.clone()).unwrap();
        mbc2.write(0x0000, 0x0A).unwrap();
        assert_eq!(mbc2.read(0xA000).unwrap(), 0x42);
        // Test with wrong size
        let result = mbc2.load_ram(vec![0u8; 1]);
        assert!(matches!(result, Err(MbcError::InvalidRamBank(_))));
    }

    #[test]
    fn test_protection_violation() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc5::new(rom);
        let result = mbc.read(0xC000);
        assert!(matches!(result, Err(MbcError::ProtectionViolation(_))));
        let result = mbc.write(0xC000, 0x01);
        assert!(matches!(result, Err(MbcError::ProtectionViolation(_))));
    }

    #[test]
    fn test_bank_switching_during_access() {
        let rom = dummy_rom(0x4000 * 8);
        let mut mbc = Mbc5::new(rom);
        mbc.write(0x0000, 0x0A).unwrap(); // Enable RAM
        mbc.write(0x4000, 0x00).unwrap(); // RAM bank 0
        mbc.write(0xA000, 0x11).unwrap();
        mbc.write(0x4000, 0x01).unwrap(); // RAM bank 1
        mbc.write(0xA000, 0x22).unwrap();
        mbc.write(0x4000, 0x00).unwrap(); // Switch back
        assert_eq!(mbc.read(0xA000).unwrap(), 0x11);
        mbc.write(0x4000, 0x01).unwrap();
        assert_eq!(mbc.read(0xA000).unwrap(), 0x22);
    }
}
