/// core-lib/src/ppu/color.rs
/// Represents a color in the Game Boy's screen using the DMG (original Game Boy) color palette
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// White color (lightest shade, color 0 in the Game Boy palette)
    pub const WHITE: Self = Self {
        r: 224,
        g: 248,
        b: 208,
        a: 255,
    };

    /// Light gray color (color 1 in the Game Boy palette)
    pub const LIGHT_GRAY: Self = Self {
        r: 136,
        g: 192,
        b: 112,
        a: 255,
    };

    /// Dark gray color (color 2 in the Game Boy palette)
    pub const DARK_GRAY: Self = Self {
        r: 52,
        g: 104,
        b: 86,
        a: 255,
    };

    /// Black color (darkest shade, color 3 in the Game Boy palette)
    pub const BLACK: Self = Self {
        r: 8,
        g: 24,
        b: 32,
        a: 255,
    };

    /// Transparent color (used for sprite transparency)
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    /// Default DMG palette colors (green-tinted)
    pub const DMG_COLORS: [Self; 4] = [Self::WHITE, Self::LIGHT_GRAY, Self::DARK_GRAY, Self::BLACK];

    /// Create a new color with the given RGBA values
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Convert a Game Boy color index (0-3) to an RGBA color using the given palette
    ///
    /// The palette is an 8-bit value where:
    /// - Bits 0-1: Color for index 0
    /// - Bits 2-3: Color for index 1
    /// - Bits 4-5: Color for index 2
    /// - Bits 6-7: Color for index 3
    pub fn from_palette(color_idx: u8, palette: u8) -> Self {
        let shift = (color_idx & 0x3) << 1;
        let palette_idx = (palette >> shift) & 0x3;
        Self::DMG_COLORS[palette_idx as usize]
    }

    /// Convert the Color to a 32-bit RGBA value
    pub fn to_rgba32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Create a Color from a 32-bit RGBA value
    pub fn from_rgba32(rgba: u32) -> Self {
        Self {
            r: ((rgba >> 16) & 0xFF) as u8,
            g: ((rgba >> 8) & 0xFF) as u8,
            b: (rgba & 0xFF) as u8,
            a: ((rgba >> 24) & 0xFF) as u8,
        }
    }
}
