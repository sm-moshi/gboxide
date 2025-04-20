/// core-lib/src/ppu/mod.rs
use thiserror::Error;

pub mod ppu;
pub use ppu::Ppu;

pub mod color;
pub use color::Color;

/// PPU-related errors
#[derive(Debug, Error)]
pub enum PpuError {
    #[error("Invalid PPU register access at address: {0:#06X}")]
    InvalidAccess(u16),
    #[error("VRAM access during pixel transfer mode")]
    VramLocked,
    #[error("OAM access during pixel transfer or OAM search mode")]
    OamLocked,
}

/// LCD Control Register (LCDC) bit flags at 0xFF40
pub mod lcdc {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct LcdControl: u8 {
            const LCD_ENABLE          = 0b1000_0000; // Bit 7: LCD Display Enable
            const WINDOW_TILEMAP      = 0b0100_0000; // Bit 6: Window Tile Map Area (0=9800-9BFF, 1=9C00-9FFF)
            const WINDOW_ENABLE       = 0b0010_0000; // Bit 5: Window Display Enable
            const BG_WINDOW_TILE_DATA = 0b0001_0000; // Bit 4: BG & Window Tile Data (0=8800-97FF, 1=8000-8FFF)
            const BG_TILEMAP          = 0b0000_1000; // Bit 3: BG Tile Map Area (0=9800-9BFF, 1=9C00-9FFF)
            const SPRITE_SIZE         = 0b0000_0100; // Bit 2: Sprite Size (0=8x8, 1=8x16)
            const SPRITE_ENABLE       = 0b0000_0010; // Bit 1: Sprite Display Enable
            const BG_WINDOW_ENABLE    = 0b0000_0001; // Bit 0: BG & Window Display Enable
        }
    }
}

/// LCD Status Register (STAT) bit flags at 0xFF41
pub mod stat {
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct LcdStatus: u8 {
            const LYC_INTERRUPT      = 0b0100_0000; // Bit 6: LYC=LY Interrupt
            const OAM_INTERRUPT      = 0b0010_0000; // Bit 5: OAM Interrupt
            const VBLANK_INTERRUPT   = 0b0001_0000; // Bit 4: VBlank Interrupt
            const HBLANK_INTERRUPT   = 0b0000_1000; // Bit 3: HBlank Interrupt
            const LYC_EQUAL_LY       = 0b0000_0100; // Bit 2: LYC=LY Flag (Read-only)
            const MODE_FLAG_MASK     = 0b0000_0011; // Bits 0-1: Mode Flag (Read-only)
        }
    }
}

/// PPU Modes as defined in STAT register bits 0-1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpuMode {
    HBlank = 0,        // Mode 0: CPU can access VRAM and OAM
    VBlank = 1,        // Mode 1: CPU can access VRAM and OAM
    OamSearch = 2,     // Mode 2: CPU can access VRAM but not OAM
    PixelTransfer = 3, // Mode 3: CPU cannot access VRAM or OAM
}

impl PpuMode {
    /// Convert from raw mode value to PpuMode enum
    pub const fn from_mode(mode: u8) -> Self {
        match mode & 0x3 {
            0 => Self::HBlank,
            1 => Self::VBlank,
            2 => Self::OamSearch,
            _ => Self::PixelTransfer,
        }
    }

    /// Get the duration of this mode in CPU cycles
    pub const fn duration(&self) -> u32 {
        match self {
            Self::OamSearch => 80,      // 80 cycles (20 T-states)
            Self::PixelTransfer => 172, // ~172 cycles (can vary)
            Self::HBlank => 204,        // 204 cycles (51 T-states)
            Self::VBlank => 456,        // 456 cycles per line (114 T-states)
        }
    }
}

// Hardware constants
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const VRAM_SIZE: usize = 0x2000; // 8KB
pub const OAM_SIZE: usize = 0xA0; // 160 bytes (40 sprites × 4 bytes)
pub const MAX_SPRITES_PER_LINE: usize = 10;

// Register addresses
pub const LCDC_ADDR: u16 = 0xFF40;
pub const STAT_ADDR: u16 = 0xFF41;
pub const SCY_ADDR: u16 = 0xFF42;
pub const SCX_ADDR: u16 = 0xFF43;
pub const LY_ADDR: u16 = 0xFF44;
pub const LYC_ADDR: u16 = 0xFF45;
pub const DMA_ADDR: u16 = 0xFF46;
pub const BGP_ADDR: u16 = 0xFF47;
pub const OBP0_ADDR: u16 = 0xFF48;
pub const OBP1_ADDR: u16 = 0xFF49;
pub const WY_ADDR: u16 = 0xFF4A;
pub const WX_ADDR: u16 = 0xFF4B;

#[cfg(test)]
mod tests;
