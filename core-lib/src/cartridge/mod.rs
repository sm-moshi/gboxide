/// core-lib/src/cartridge/mod.rs
use crate::mmu::mbc::{Mbc, Mbc1, NoMbc};
use thiserror::Error;

/// Errors that can occur when working with cartridges
#[derive(Debug, Error)]
pub enum CartridgeError {
    #[error("Invalid ROM size")]
    InvalidRomSize,
    #[error("Invalid cartridge type: {0:#04X}")]
    InvalidCartridgeType(u8),
    #[error("Unsupported cartridge type: {0:#04X}")]
    UnsupportedCartridgeType(u8),
    #[error("Invalid size: {0}")]
    InvalidSize(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    // TODO: Add other MBC types as we implement them
}

impl TryFrom<u8> for CartridgeType {
    type Error = CartridgeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::RomOnly),
            0x01 => Ok(Self::Mbc1),
            0x02 => Ok(Self::Mbc1Ram),
            0x03 => Ok(Self::Mbc1RamBattery),
            0x05..=0xFF => Err(CartridgeError::UnsupportedCartridgeType(value)),
            _ => Err(CartridgeError::InvalidCartridgeType(value)),
        }
    }
}

/// Represents a Game Boy cartridge with its ROM and RAM
/// core/src/cartridge/mod.rs
pub struct Cartridge {
    pub data: Vec<u8>,
    cart_type: u8,
    rom_size: usize,
    ram_size: usize,
    rom_bank: usize,
    ram_bank: usize,
    ram_enabled: bool,
    has_battery: bool,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        if rom.len() < 0x150 {
            return Err(CartridgeError::InvalidSize(rom.len()));
        }

        let cart_type = rom[0x147];
        let rom_size_code = Self::get_rom_size(rom[0x148]);
        let ram_size_bytes = Self::get_ram_size(rom[0x149]);
        let has_battery = matches!(cart_type, 0x03 | 0x06);

        Ok(Self {
            data: rom,
            cart_type,
            rom_size: rom_size_code,
            ram_size: ram_size_bytes,
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            has_battery,
        })
    }

    const fn get_rom_size(value: u8) -> usize {
        match value {
            0x00 => 32 * 1024,   // 32KB (2 banks)
            0x01 => 64 * 1024,   // 64KB (4 banks)
            0x02 => 128 * 1024,  // 128KB (8 banks)
            0x03 => 256 * 1024,  // 256KB (16 banks)
            0x04 => 512 * 1024,  // 512KB (32 banks)
            0x05 => 1024 * 1024, // 1MB (64 banks)
            0x06 => 2048 * 1024, // 2MB (128 banks)
            0x07 => 4096 * 1024, // 4MB (256 banks)
            0x08 => 8192 * 1024, // 8MB (512 banks)
            _ => 32 * 1024,      // Default to 32KB
        }
    }

    const fn get_ram_size(value: u8) -> usize {
        match value {
            0x00 => 0,          // No RAM
            0x01 => 2 * 1024,   // 2KB
            0x02 => 8 * 1024,   // 8KB
            0x03 => 32 * 1024,  // 32KB (4 banks of 8KB each)
            0x04 => 128 * 1024, // 128KB (16 banks of 8KB each)
            0x05 => 64 * 1024,  // 64KB (8 banks of 8KB each)
            _ => 0,             // Default to no RAM
        }
    }

    /// Create an appropriate MBC instance based on the cartridge type

    pub fn create_mbc(self) -> Box<dyn Mbc> {
        match self.cart_type {
            0x00 => Box::new(NoMbc::new(self.data)),
            0x01 => Box::new(Mbc1::new(self.data)),
            0x02 => Box::new(Mbc1::new(self.data)),
            0x03 => Box::new(Mbc1::new(self.data)),
            0x05 => Box::new(Mbc1::new(self.data)),
            0x06 => Box::new(Mbc1::new(self.data)),
            _ => panic!("Unsupported cartridge type: {:#04X}", self.cart_type),
        }
    }

    /// Get the cartridge title from the ROM header

    pub fn title(&self) -> String {
        let title_bytes = &self.data[0x134..=0x143];
        let end = title_bytes.iter().position(|&b| b == 0).unwrap_or(16);
        String::from_utf8_lossy(&title_bytes[..end]).into_owned()
    }

    /// Get the cartridge type

    pub fn cartridge_type(&self) -> CartridgeType {
        CartridgeType::try_from(self.cart_type).unwrap()
    }

    /// Get the ROM size in bytes

    pub const fn rom_size(&self) -> usize {
        self.rom_size
    }

    /// Get the RAM size in bytes

    pub const fn ram_size(&self) -> usize {
        self.ram_size
    }

    /// Check if the cartridge has battery-backed RAM

    pub const fn has_battery(&self) -> bool {
        self.has_battery
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => self.data[addr as usize],

            // ROM Bank 1-N
            0x4000..=0x7FFF => {
                let bank_offset = self.rom_bank * 0x4000;
                let addr_offset = (addr - 0x4000) as usize;
                self.data[bank_offset + addr_offset]
            }

            // External RAM
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank_offset = self.ram_bank * 0x2000;
                    let addr_offset = (addr - 0xA000) as usize;
                    self.data[bank_offset + addr_offset]
                } else {
                    0xFF
                }
            }

            _ => panic!("Invalid cartridge address: {addr:#06X}"),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM Enable
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }

            // ROM Bank Number
            0x2000..=0x3FFF => {
                let bank = value & 0x1F;
                self.rom_bank = if bank == 0 { 1 } else { bank as usize };
            }

            // RAM Bank Number
            0x4000..=0x5FFF => {
                self.ram_bank = (value & 0x03) as usize;
            }

            // External RAM
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank_offset = self.ram_bank * 0x2000;
                    let addr_offset = (addr - 0xA000) as usize;
                    self.data[bank_offset + addr_offset] = value;
                }
            }

            _ => panic!("Invalid cartridge write address: {addr:#06X}"),
        }
    }
}
