mod error;
mod types;

use crate::mmu::mbc::{Mbc, Mbc1, Mbc2, Mbc3, Mbc5, NoMbc};
pub use error::CartridgeError;
pub use types::{CartridgeType, RamSize, RomSize};

/// Represents a Game Boy cartridge with its ROM and RAM.
///
/// This struct encapsulates all header and banking logic for a loaded cartridge,
/// including type, size, and battery/feature flags. It provides safe, error-aware
/// access to ROM and RAM, and is robust to malformed or unsupported ROMs.
pub struct Cartridge {
    pub data: Vec<u8>,
    pub cart_type: CartridgeType,
    pub rom_size: RomSize,
    pub ram_size: RamSize,
    pub rom_bank: usize,
    pub ram_bank: usize,
    pub ram_enabled: bool,
    pub has_battery: bool,
    /// Dedicated RAM buffer for RAM-supporting cartridges
    pub ram: Option<Vec<u8>>,
}

impl Cartridge {
    /// Create a new Cartridge from ROM data, validating all header fields.
    ///
    /// This ensures that only supported cartridge types and sizes are accepted,
    /// and that all feature flags are parsed. Returns a detailed error if the
    /// ROM is malformed or unsupported.
    pub fn new(rom: Vec<u8>) -> Result<Self, CartridgeError> {
        if rom.len() < 0x150 {
            return Err(CartridgeError::InvalidSize(rom.len()));
        }
        let cart_type = CartridgeType::from_u8(rom[0x147])
            .ok_or(CartridgeError::UnsupportedCartridgeType(rom[0x147]))?;
        let rom_size =
            RomSize::from_u8(rom[0x148]).ok_or(CartridgeError::InvalidSize(rom[0x148] as usize))?;
        let ram_size =
            RamSize::from_u8(rom[0x149]).ok_or(CartridgeError::InvalidSize(rom[0x149] as usize))?;
        let has_battery = matches!(
            cart_type,
            CartridgeType::Mbc1 { battery: true, .. }
                | CartridgeType::Mbc2 { battery: true }
                | CartridgeType::Mbc3 { battery: true, .. }
                | CartridgeType::Mbc5 { battery: true, .. }
        );
        // Allocate RAM buffer if needed
        let ram = if ram_size.size() > 0 {
            Some(vec![0xFF; ram_size.size()])
        } else {
            None
        };
        Ok(Self {
            data: rom,
            cart_type,
            rom_size,
            ram_size,
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,
            has_battery,
            ram,
        })
    }

    /// Create an appropriate MBC instance based on the cartridge type
    ///
    /// # Errors
    /// Returns `CartridgeError::UnsupportedCartridgeType` if the type is not supported.
    pub fn create_mbc(self) -> Result<Box<dyn Mbc>, CartridgeError> {
        match self.cart_type {
            CartridgeType::RomOnly => Ok(Box::new(NoMbc::new(self.data))),
            CartridgeType::Mbc1 { .. } => Ok(Box::new(Mbc1::new(self.data))),
            CartridgeType::Mbc2 { .. } => Ok(Box::new(Mbc2::new(self.data))),
            CartridgeType::Mbc3 { .. } => Ok(Box::new(Mbc3::new(self.data))),
            CartridgeType::Mbc5 { .. } => Ok(Box::new(Mbc5::new(self.data))),
            CartridgeType::HuC1 | CartridgeType::HuC3 => {
                Err(CartridgeError::UnsupportedCartridgeType(0xFF))
            }
        }
    }

    /// Get the cartridge title from the ROM header (old and new style).
    ///
    /// This supports both legacy and modern title encodings, and always returns
    /// a valid UTF-8 string (lossy if needed).
    pub fn title(&self) -> String {
        let is_new = self.data.get(0x14B).copied() == Some(0x33);
        let slice = if is_new {
            &self.data[0x134..0x13F]
        } else {
            &self.data[0x134..=0x143]
        };
        let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        String::from_utf8_lossy(&slice[..end]).into_owned()
    }

    /// Get the ROM size in bytes
    pub fn rom_size(&self) -> usize {
        self.rom_size.size()
    }

    /// Get the RAM size in bytes
    pub fn ram_size(&self) -> usize {
        self.ram_size.size()
    }

    /// Check if the cartridge has battery-backed RAM
    pub fn has_battery(&self) -> bool {
        self.has_battery
    }

    /// Reads a byte from the cartridge at the given address.
    ///
    /// Returns an error if the address is out of bounds or not mapped by the
    /// cartridge type. This prevents panics and ensures robust emulation.
    pub fn read(&self, addr: u16) -> Result<u8, CartridgeError> {
        match addr {
            0x0000..=0x3FFF => self
                .data
                .get(addr as usize)
                .copied()
                .ok_or(CartridgeError::InvalidAddress(addr)),
            0x4000..=0x7FFF => {
                let bank_offset = self.rom_bank * 0x4000;
                let idx = bank_offset + (addr as usize - 0x4000);
                self.data
                    .get(idx)
                    .copied()
                    .ok_or(CartridgeError::InvalidAddress(addr))
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if let Some(ram) = &self.ram {
                        let bank_offset = self.ram_bank * 0x2000;
                        let idx = bank_offset + (addr as usize - 0xA000);
                        ram.get(idx)
                            .copied()
                            .ok_or(CartridgeError::InvalidAddress(addr))
                    } else {
                        Err(CartridgeError::InvalidAddress(addr))
                    }
                } else {
                    Ok(0xFF)
                }
            }
            _ => Err(CartridgeError::InvalidAddress(addr)),
        }
    }

    /// Writes a byte to the cartridge at the given address.
    ///
    /// Returns an error if the address is out of bounds or not mapped by the
    /// cartridge type. This prevents panics and ensures robust emulation.
    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), CartridgeError> {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
                Ok(())
            }
            0x2000..=0x3FFF => {
                let bank = value & 0x1F;
                self.rom_bank = if bank == 0 { 1 } else { bank as usize };
                Ok(())
            }
            0x4000..=0x5FFF => {
                self.ram_bank = (value & 0x03) as usize;
                Ok(())
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if let Some(ram) = &mut self.ram {
                        let bank_offset = self.ram_bank * 0x2000;
                        let idx = bank_offset + (addr as usize - 0xA000);
                        if let Some(cell) = ram.get_mut(idx) {
                            *cell = value;
                            Ok(())
                        } else {
                            Err(CartridgeError::InvalidAddress(addr))
                        }
                    } else {
                        Err(CartridgeError::InvalidAddress(addr))
                    }
                } else {
                    Ok(())
                }
            }
            _ => Err(CartridgeError::InvalidAddress(addr)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mmu::mbc::Mbc;
    #[test]
    fn test_valid_romonly_cartridge_construction() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x00;
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let cart = Cartridge::new(rom);
        assert!(cart.is_ok());
        let cart = cart.unwrap();
        assert_eq!(cart.cart_type, CartridgeType::RomOnly);
        assert_eq!(cart.rom_size, RomSize::Size32KB);
        assert_eq!(cart.ram_size, RamSize::None);
        assert!(!cart.has_battery);
        assert_eq!(cart.rom_size(), 32 * 1024);
        assert_eq!(cart.ram_size(), 0);
        assert!(!cart.has_battery());
    }
    #[test]
    fn test_invalid_rom_too_small() {
        let rom = vec![0; 0x100];
        let cart = Cartridge::new(rom);
        assert!(matches!(cart, Err(CartridgeError::InvalidSize(0x100))));
    }
    #[test]
    fn test_unsupported_cartridge_type() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0xAA;
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let cart = Cartridge::new(rom);
        assert!(matches!(
            cart,
            Err(CartridgeError::UnsupportedCartridgeType(0xAA))
        ));
    }
    #[test]
    fn test_invalid_rom_size_header() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x00;
        rom[0x148] = 0xFF;
        rom[0x149] = 0x00;
        let cart = Cartridge::new(rom);
        assert!(matches!(cart, Err(CartridgeError::InvalidSize(0xFF))));
    }
    #[test]
    fn test_invalid_ram_size_header() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x00;
        rom[0x148] = 0x00;
        rom[0x149] = 0xFF;
        let cart = Cartridge::new(rom);
        assert!(matches!(cart, Err(CartridgeError::InvalidSize(0xFF))));
    }
    #[test]
    fn test_title_parsing_old_and_new() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x00;
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let title = b"TESTTITLE";
        rom[0x134..0x134 + title.len()].copy_from_slice(title);
        let cart = Cartridge::new(rom.clone()).unwrap();
        assert_eq!(cart.title(), "TESTTITLE");
        let mut rom_new = rom.clone();
        rom_new[0x14B] = 0x33;
        let cart_new = Cartridge::new(rom_new).unwrap();
        assert_eq!(cart_new.title(), "TESTTITLE");
    }
    #[test]
    fn test_invalid_address_read_write() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x00;
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let mut cart = Cartridge::new(rom).unwrap();
        assert!(matches!(
            cart.read(0xC000),
            Err(CartridgeError::InvalidAddress(0xC000))
        ));
        assert!(matches!(
            cart.write(0xC000, 0x12),
            Err(CartridgeError::InvalidAddress(0xC000))
        ));
    }
    #[test]
    fn test_create_mbc_variants() {
        // ROM ONLY
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x00;
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let cart = Cartridge::new(rom).unwrap();
        let mbc = cart.create_mbc();
        assert!(mbc.is_ok());
        // HuC1/3 unsupported
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0xFE;
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let cart = Cartridge::new(rom).unwrap();
        let mbc = cart.create_mbc();
        assert!(matches!(
            mbc,
            Err(CartridgeError::UnsupportedCartridgeType(0xFF))
        ));
    }
    #[test]
    fn test_ram_enable_disable_and_write_error() {
        // Use a cartridge type that supports RAM
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x03; // MBC1 + RAM + BATTERY (supports RAM)
        rom[0x148] = 0x00; // 32KB ROM
        rom[0x149] = 0x02; // 8KB RAM
        let mut cart = Cartridge::new(rom).unwrap();
        // RAM not enabled: write should be Ok (ignored)
        assert_eq!(cart.read(0xA000).unwrap(), 0xFF);
        assert!(cart.write(0xA000, 0x42).is_ok());
        // Enable RAM
        cart.write(0x0000, 0x0A).unwrap();
        // Write to out-of-range RAM address (should error)
        assert!(cart.write(0xBFFF + 1, 0x42).is_err());
    }
    #[test]
    fn test_ram_enable_disable_and_write_success() {
        // Use a cartridge type that supports RAM
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x03; // MBC1 + RAM + BATTERY (supports RAM)
        rom[0x148] = 0x00; // 32KB ROM
        rom[0x149] = 0x02; // 8KB RAM
        let mut cart = Cartridge::new(rom).unwrap();
        // Enable RAM
        cart.write(0x0000, 0x0A).unwrap();
        // Write to RAM (should succeed)
        assert!(cart.write(0xA000, 0x42).is_ok());
        // Read back the value
        assert_eq!(cart.read(0xA000).unwrap(), 0x42);
    }
    #[test]
    fn test_rom_and_ram_banking() {
        let mut rom = vec![0; 0x4000 * 4]; // 4 ROM banks
        rom.resize(0x150, 0);
        rom[0x147] = 0x00;
        rom[0x148] = 0x01; // 64KB ROM
        rom[0x149] = 0x01; // 2KB RAM
        let mut cart = Cartridge::new(rom).unwrap();
        // Switch ROM bank
        cart.write(0x2000, 0x02).unwrap();
        assert_eq!(cart.rom_bank, 2);
        // Switch RAM bank
        cart.write(0x4000, 0x01).unwrap();
        assert_eq!(cart.ram_bank, 1);
    }
    #[test]
    fn test_battery_flag_for_mbc_types() {
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x03; // MBC1 + RAM + BATTERY
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let cart = Cartridge::new(rom).unwrap();
        assert!(cart.has_battery);
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x06; // MBC2 + BATTERY
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let mut cart = Cartridge::new(rom).unwrap();
        assert!(cart.has_battery);
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x13; // MBC3 + RAM + BATTERY
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let mut cart = Cartridge::new(rom).unwrap();
        assert!(cart.has_battery);
        let mut rom = vec![0; 0x150];
        rom[0x147] = 0x1B; // MBC5 + RAM + BATTERY
        rom[0x148] = 0x00;
        rom[0x149] = 0x00;
        let mut cart = Cartridge::new(rom).unwrap();
        assert!(cart.has_battery);
    }
}
