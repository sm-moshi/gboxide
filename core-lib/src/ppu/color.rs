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
    pub const fn from_palette(color_idx: u8, palette: u8) -> Self {
        let shift = (color_idx & 0x3) << 1;
        let palette_idx = (palette >> shift) & 0x3;
        Self::DMG_COLORS[palette_idx as usize]
    }

    /// Convert the Color to a 32-bit RGBA value
    pub const fn to_rgba32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Create a Color from a 32-bit RGBA value
    pub const fn from_rgba32(rgba: u32) -> Self {
        Self {
            r: ((rgba >> 16) & 0xFF) as u8,
            g: ((rgba >> 8) & 0xFF) as u8,
            b: (rgba & 0xFF) as u8,
            a: ((rgba >> 24) & 0xFF) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_color_constants() {
        assert_eq!(
            Color::WHITE,
            Color {
                r: 224,
                g: 248,
                b: 208,
                a: 255
            }
        );
        assert_eq!(
            Color::LIGHT_GRAY,
            Color {
                r: 136,
                g: 192,
                b: 112,
                a: 255
            }
        );
        assert_eq!(
            Color::DARK_GRAY,
            Color {
                r: 52,
                g: 104,
                b: 86,
                a: 255
            }
        );
        assert_eq!(
            Color::BLACK,
            Color {
                r: 8,
                g: 24,
                b: 32,
                a: 255
            }
        );
        assert_eq!(
            Color::TRANSPARENT,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            }
        );
        assert_eq!(
            Color::DMG_COLORS,
            [
                Color::WHITE,
                Color::LIGHT_GRAY,
                Color::DARK_GRAY,
                Color::BLACK
            ]
        );
    }

    #[test]
    fn test_new_and_equality() {
        let c = Color::new(1, 2, 3, 4);
        assert_eq!(
            c,
            Color {
                r: 1,
                g: 2,
                b: 3,
                a: 4
            }
        );
    }

    #[test]
    fn test_from_palette() {
        // Palette: 0b11_10_01_00 (0xE4)
        let palette = 0xE4;
        assert_eq!(Color::from_palette(0, palette), Color::WHITE);
        assert_eq!(Color::from_palette(1, palette), Color::LIGHT_GRAY);
        assert_eq!(Color::from_palette(2, palette), Color::DARK_GRAY);
        assert_eq!(Color::from_palette(3, palette), Color::BLACK);
    }

    #[test]
    fn test_to_rgba32_and_from_rgba32() {
        let c = Color::new(10, 20, 30, 40);
        let rgba = c.to_rgba32();
        let c2 = Color::from_rgba32(rgba);
        assert_eq!(c, c2);
    }
}
