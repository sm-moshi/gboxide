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

mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
pub use mbc1::Mbc1;
pub use mbc2::Mbc2;
pub use mbc3::Mbc3;
pub use mbc5::Mbc5;

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
