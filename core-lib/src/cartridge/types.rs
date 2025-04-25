/// Cartridge type enum, covering all supported MBCs and features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartridgeType {
    /// ROM ONLY (no MBC)
    RomOnly,
    /// MBC1 (with optional RAM and battery)
    Mbc1 {
        ram: bool,
        battery: bool,
        multicart: bool,
    },
    /// MBC2 (with optional battery)
    Mbc2 { battery: bool },
    /// MBC3 (with optional RAM, battery, RTC)
    Mbc3 { ram: bool, battery: bool, rtc: bool },
    /// MBC5 (with optional RAM, battery, rumble)
    Mbc5 {
        ram: bool,
        battery: bool,
        rumble: bool,
    },
    /// `HuC1` (Hudson Soft, rare)
    HuC1,
    /// `HuC3` (Hudson Soft, rare)
    HuC3,
}

impl CartridgeType {
    /// Attempts to create a `CartridgeType` from its header byte (0x147).
    /// Returns `None` if the byte doesn't correspond to a known type.
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 | 0x08 | 0x09 => Some(Self::RomOnly), // Merged identical arms
            0x01 => Some(Self::Mbc1 {
                ram: false,
                battery: false,
                multicart: false,
            }),
            0x02 => Some(Self::Mbc1 {
                ram: true,
                battery: false,
                multicart: false,
            }),
            0x03 => Some(Self::Mbc1 {
                ram: true,
                battery: true,
                multicart: false,
            }),
            0x05 => Some(Self::Mbc2 { battery: false }),
            0x06 => Some(Self::Mbc2 { battery: true }),
            0x0F => Some(Self::Mbc3 {
                ram: false,
                battery: true,
                rtc: true,
            }),
            0x10 => Some(Self::Mbc3 {
                ram: true,
                battery: true,
                rtc: true,
            }),
            0x11 => Some(Self::Mbc3 {
                ram: false,
                battery: false,
                rtc: false,
            }),
            0x12 => Some(Self::Mbc3 {
                ram: true,
                battery: false,
                rtc: false,
            }),
            0x13 => Some(Self::Mbc3 {
                ram: true,
                battery: true,
                rtc: false,
            }),
            0x19 => Some(Self::Mbc5 {
                ram: false,
                battery: false,
                rumble: false,
            }),
            0x1A => Some(Self::Mbc5 {
                ram: true,
                battery: false,
                rumble: false,
            }),
            0x1B => Some(Self::Mbc5 {
                ram: true,
                battery: true,
                rumble: false,
            }),
            0x1C => Some(Self::Mbc5 {
                ram: false,
                battery: false,
                rumble: true,
            }),
            0x1D => Some(Self::Mbc5 {
                ram: true,
                battery: false,
                rumble: true,
            }),
            0x1E => Some(Self::Mbc5 {
                ram: true,
                battery: true,
                rumble: true,
            }),
            0xFE => Some(Self::HuC3),
            0xFF => Some(Self::HuC1),
            _ => None,
        }
    }
}

/// ROM size enum (from header)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomSize {
    Size32KB,
    Size64KB,
    Size128KB,
    Size256KB,
    Size512KB,
    Size1MB,
    Size2MB,
    Size4MB,
    Size8MB,
}

impl RomSize {
    /// Attempts to create a `RomSize` from its header byte (0x148).
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Size32KB),
            0x01 => Some(Self::Size64KB),
            0x02 => Some(Self::Size128KB),
            0x03 => Some(Self::Size256KB),
            0x04 => Some(Self::Size512KB),
            0x05 => Some(Self::Size1MB),
            0x06 => Some(Self::Size2MB),
            0x07 => Some(Self::Size4MB),
            0x08 => Some(Self::Size8MB),
            _ => None,
        }
    }
    /// Returns the size in bytes for this ROM size.
    pub const fn size(self) -> usize {
        self.as_bytes()
    }
    /// Returns the size in bytes.
    pub const fn as_bytes(self) -> usize {
        match self {
            Self::Size32KB => 32 * 1024,
            Self::Size64KB => 64 * 1024,
            Self::Size128KB => 128 * 1024,
            Self::Size256KB => 256 * 1024,
            Self::Size512KB => 512 * 1024,
            Self::Size1MB => 1024 * 1024,
            Self::Size2MB => 2 * 1024 * 1024,
            Self::Size4MB => 4 * 1024 * 1024,
            Self::Size8MB => 8 * 1024 * 1024,
        }
    }
}

/// RAM size enum (from header)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RamSize {
    None,
    Size2KB,
    Size8KB,
    Size32KB,
    Size128KB,
    Size64KB,
}

impl RamSize {
    /// Attempts to create a `RamSize` from its header byte (0x149).
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::None),
            0x01 => Some(Self::Size2KB),
            0x02 => Some(Self::Size8KB),
            0x03 => Some(Self::Size32KB),
            0x04 => Some(Self::Size128KB),
            0x05 => Some(Self::Size64KB),
            _ => None,
        }
    }
    /// Returns the size in bytes for this RAM size.
    pub const fn size(self) -> usize {
        self.as_bytes()
    }
    /// Returns the size in bytes.
    pub const fn as_bytes(self) -> usize {
        match self {
            Self::None => 0,
            Self::Size2KB => 2 * 1024,
            Self::Size8KB => 8 * 1024,
            Self::Size32KB => 32 * 1024,
            Self::Size128KB => 128 * 1024,
            Self::Size64KB => 64 * 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cartridge_type_from_u8() {
        assert_eq!(CartridgeType::from_u8(0x00), Some(CartridgeType::RomOnly));
        assert!(matches!(
            CartridgeType::from_u8(0x01),
            Some(CartridgeType::Mbc1 { .. })
        ));
        assert!(matches!(
            CartridgeType::from_u8(0x05),
            Some(CartridgeType::Mbc2 { .. })
        ));
        assert!(matches!(
            CartridgeType::from_u8(0x0F),
            Some(CartridgeType::Mbc3 { .. })
        ));
        assert!(matches!(
            CartridgeType::from_u8(0x19),
            Some(CartridgeType::Mbc5 { .. })
        ));
        assert_eq!(CartridgeType::from_u8(0xFE), Some(CartridgeType::HuC3));
        assert_eq!(CartridgeType::from_u8(0xFF), Some(CartridgeType::HuC1));
        assert_eq!(CartridgeType::from_u8(0xAB), None);
    }
    #[test]
    fn test_rom_size_from_u8_and_size() {
        assert_eq!(RomSize::from_u8(0x00), Some(RomSize::Size32KB));
        assert_eq!(RomSize::from_u8(0x01), Some(RomSize::Size64KB));
        assert_eq!(RomSize::from_u8(0x08), Some(RomSize::Size8MB));
        assert_eq!(RomSize::from_u8(0xFF), None);
        assert_eq!(RomSize::Size32KB.size(), 32 * 1024);
        assert_eq!(RomSize::Size8MB.as_bytes(), 8 * 1024 * 1024);
    }
    #[test]
    fn test_ram_size_from_u8_and_size() {
        assert_eq!(RamSize::from_u8(0x00), Some(RamSize::None));
        assert_eq!(RamSize::from_u8(0x01), Some(RamSize::Size2KB));
        assert_eq!(RamSize::from_u8(0x05), Some(RamSize::Size64KB));
        assert_eq!(RamSize::from_u8(0xFF), None);
        assert_eq!(RamSize::Size2KB.size(), 2 * 1024);
        assert_eq!(RamSize::Size64KB.as_bytes(), 64 * 1024);
    }
}
