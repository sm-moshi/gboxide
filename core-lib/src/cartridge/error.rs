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
    #[error("Invalid cartridge address: {0:#06X}")]
    InvalidAddress(u16),
}
