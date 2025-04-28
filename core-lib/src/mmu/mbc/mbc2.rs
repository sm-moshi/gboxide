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
    pub const fn new(rom: Vec<u8>) -> Self {
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
    fn test_rom_read_bank0_and_switchable() {
        let rom = dummy_rom(0x8000); // 2 banks
        let mbc = Mbc2::new(rom.clone());
        // Bank 0
        assert_eq!(mbc.read(0x0000).unwrap(), 0);
        assert_eq!(mbc.read(0x3FFF).unwrap(), 0);
        // Switchable bank (default 1)
        assert_eq!(mbc.read(0x4000).unwrap(), 1);
        assert_eq!(mbc.read(0x7FFF).unwrap(), 1);
        // Out of bounds
        let mbc = Mbc2::new(vec![0u8; 0x4000]); // Only 1 bank
        assert_eq!(mbc.read(0x4000).unwrap(), 0xFF);
    }

    #[test]
    fn test_rom_bank_switching() {
        let rom = dummy_rom(0x4000 * 4); // 4 banks
        let mut mbc = Mbc2::new(rom);
        // Write to ROM bank register (should select bank 2)
        mbc.write(0x2100, 0x02).unwrap();
        assert_eq!(mbc.rom_bank, 2);
        // Bank 0 is forbidden, should map to 1
        mbc.write(0x2100, 0x00).unwrap();
        assert_eq!(mbc.rom_bank, 1);
    }

    #[test]
    fn test_ram_enable_disable() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc2::new(rom);
        // RAM disabled by default
        assert!(!mbc.ram_enabled);
        // Enable RAM
        mbc.write(0x0000, 0x0A).unwrap();
        assert!(mbc.ram_enabled);
        // Disable RAM
        mbc.write(0x0000, 0x00).unwrap();
        assert!(!mbc.ram_enabled);
        // Write to non-RAM-enable address (should not change state, should error)
        let result = mbc.write(0x0100, 0x0A);
        assert!(matches!(result, Err(MbcError::ProtectionViolation(0x0100))));
        assert!(!mbc.ram_enabled);
    }

    #[test]
    fn test_ram_write_and_read_nibble_masking() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc2::new(rom);
        mbc.write(0x0000, 0x0A).unwrap(); // Enable RAM
                                          // Write value with upper nibble set
        mbc.write(0xA000, 0xAB).unwrap();
        // Only lower nibble should be stored, upper nibble should be open bus (0xF0)
        let val = mbc.read(0xA000).unwrap();
        assert_eq!(val & 0x0F, 0x0B);
        assert_eq!(val & 0xF0, 0xF0);
    }

    #[test]
    fn test_ram_disabled_error() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc2::new(rom);
        let result = mbc.write(0xA000, 0x01);
        assert!(matches!(result, Err(MbcError::RamDisabled)));
        let result = mbc.read(0xA000);
        assert!(matches!(result, Err(MbcError::RamDisabled)));
    }

    #[test]
    fn test_protection_violation_error() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc2::new(rom);
        let result = mbc.write(0xC000, 0x01);
        assert!(matches!(result, Err(MbcError::ProtectionViolation(_))));
        let result = mbc.read(0xC000);
        assert!(matches!(result, Err(MbcError::ProtectionViolation(_))));
    }

    #[test]
    fn test_load_ram_wrong_size() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc2::new(rom);
        let result = mbc.load_ram(vec![0u8; 256]);
        assert!(matches!(result, Err(MbcError::InvalidRamBank(_))));
    }

    #[test]
    fn test_save_and_load_ram_success() {
        let rom = dummy_rom(0x8000);
        let mut mbc = Mbc2::new(rom);
        mbc.write(0x0000, 0x0A).unwrap(); // Enable RAM
        mbc.write(0xA000, 0x05).unwrap();
        let saved = mbc.save_ram();
        assert_eq!(saved[0], 0x05);
        // Overwrite RAM and reload
        let mut mbc2 = Mbc2::new(dummy_rom(0x8000));
        mbc2.load_ram(saved.clone()).unwrap();
        mbc2.write(0x0000, 0x0A).unwrap(); // Enable RAM
        assert_eq!(mbc2.read(0xA000).unwrap() & 0x0F, 0x05);
    }
}
