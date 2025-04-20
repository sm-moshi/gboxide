/// core-lib/src/mmu/mbc.rs
use thiserror::Error;

/// Errors that can occur in Memory Bank Controllers
#[derive(Debug, Error)]
pub enum MbcError {
    #[error("Invalid ROM bank number: {0}")]
    InvalidRomBank(usize),
    #[error("Invalid RAM bank number: {0}")]
    InvalidRamBank(usize),
    #[error("RAM access while disabled")]
    RamDisabled,
    #[error("Memory protection violation at address: {0:#06X}")]
    ProtectionViolation(u16),
}

/// Memory Bank Controller trait defining common functionality
pub trait Mbc {
    /// Read a byte from the specified address
    fn read(&self, addr: u16) -> Result<u8, MbcError>;

    /// Write a byte to the specified address
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MbcError>;

    /// Get the current ROM bank number
    fn rom_bank(&self) -> usize;

    /// Get the current RAM bank number
    fn ram_bank(&self) -> usize;

    /// Check if RAM is enabled
    fn is_ram_enabled(&self) -> bool;

    /// Save the current RAM state
    fn save_ram(&self) -> Vec<u8>;

    /// Load a RAM state
    fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError>;
}

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
                let bank = self.current_ram_bank();
                let idx = bank * 0x2000 + (addr as usize - 0xA000);
                Ok(self.ram.get(idx).copied().unwrap_or(0xFF))
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
                }
                Ok(())
            }

            // Banking Mode Select
            0x6000..=0x7FFF => {
                self.mode = value & 0x01;
                Ok(())
            }

            // RAM Bank Write
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                let bank = self.current_ram_bank();
                let idx = bank * 0x2000 + (addr as usize - 0xA000);
                if idx < self.ram.len() {
                    self.ram[idx] = value;
                    Ok(())
                } else {
                    Err(MbcError::InvalidRamBank(bank))
                }
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

/// No MBC (32KB ROM only)
pub struct NoMbc {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
}

impl NoMbc {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            ram: vec![0; 0x2000], // 8KB RAM
            ram_enabled: false,
        }
    }
}

impl Mbc for NoMbc {
    fn read(&self, addr: u16) -> Result<u8, MbcError> {
        match addr {
            0x0000..=0x7FFF => Ok(self.rom.get(addr as usize).copied().unwrap_or(0xFF)),
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                Ok(self.ram[(addr - 0xA000) as usize])
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> Result<(), MbcError> {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
                Ok(())
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                self.ram[(addr - 0xA000) as usize] = value;
                Ok(())
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }

    fn rom_bank(&self) -> usize {
        0
    }

    fn ram_bank(&self) -> usize {
        0
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
