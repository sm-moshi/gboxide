use crate::helpers::{extract_colour_index, unpack_tile_attributes};
use crate::ppu::color::Color;
use crate::ppu::MAX_SPRITES_PER_LINE;

/// Sprite object representation with position and attributes.
///
/// Each sprite in OAM is 4 bytes of the following format (see Pandocs):
/// - Byte 0: Y position (minus 16). Offscreen if Y=0 or Y>=160.
/// - Byte 1: X position (minus 8). Offscreen if X=0 or X>=168.
/// - Byte 2: Tile index (unsigned, selects tile from $8000-8FFF).
/// - Byte 3: Sprite attributes:
///   - Bit 7: OBJ-to-BG Priority (0=OBJ above BG, 1=OBJ behind BG colour 1-3; BG colour 0 always behind OBJ)
///   - Bit 6: Y flip
///   - Bit 5: X flip
///   - Bit 4: Palette number (DMG only, 0=OBP0, 1=OBP1)
///   - Bit 3: VRAM bank (CGB only)
///   - Bits 2-0: Palette number (CGB only)
#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub y_pos: u8,      // Y coordinate (minus 16)
    pub x_pos: u8,      // X coordinate (minus 8)
    pub tile_idx: u8,   // Tile index
    pub attributes: u8, // Attributes (priority, flip, palette)
}

/// Results of an OAM scan: up to 10 sprites visible on the current scanline
///
/// Hardware: Only 10 sprites can be displayed per scanline (see Pandocs).
/// This struct holds the scan result for a single scanline, in OAM order (lowest index = highest priority).
#[derive(Debug)]
pub struct OamScanResult {
    /// Array of up to 10 sprites visible on the current scanline
    pub sprites: [Option<Sprite>; MAX_SPRITES_PER_LINE],
    /// Number of sprites found in the scan
    pub count: usize,
}

impl Default for OamScanResult {
    fn default() -> Self {
        Self::new()
    }
}

impl OamScanResult {
    /// Create a new empty OAM scan result
    ///
    /// All slots are initialised to None, count is zero.
    pub const fn new() -> Self {
        Self {
            sprites: [None; MAX_SPRITES_PER_LINE],
            count: 0,
        }
    }

    /// Reset the OAM scan result for reuse
    pub const fn reset(&mut self) {
        self.sprites = [None; MAX_SPRITES_PER_LINE];
        self.count = 0;
    }

    /// Add a sprite to the OAM scan result if there's room
    ///
    /// Returns true if the sprite was added, false if at capacity (10 sprites per scanline, hardware limit).
    pub const fn add_sprite(&mut self, sprite: Sprite) -> bool {
        if self.count >= MAX_SPRITES_PER_LINE {
            return false;
        }

        self.sprites[self.count] = Some(sprite);
        self.count += 1;
        true
    }
}

impl Sprite {
    /// Create a new Sprite from OAM data
    ///
    /// OAM is 160 bytes (40 sprites × 4 bytes). Each sprite is 4 bytes as per Pandocs.
    pub fn from_oam(oam: &[u8], index: usize) -> Self {
        let base = index * 4;
        Self {
            y_pos: oam[base],
            x_pos: oam[base + 1],
            tile_idx: oam[base + 2],
            attributes: oam[base + 3],
        }
    }

    /// Check if sprite has priority over background
    ///
    /// Returns true if sprite should be drawn on top of background (except colour 0), which happens when bit 7 is NOT set.
    /// Returns false if sprite is behind background (bit 7 IS set). See Pandocs: OBJ-to-BG Priority.
    pub const fn has_priority(&self) -> bool {
        self.attributes & 0x80 == 0
    }

    /// Check if sprite is flipped horizontally
    pub const fn is_x_flipped(&self) -> bool {
        self.attributes & 0x20 != 0
    }

    /// Check if sprite is flipped vertically
    pub const fn is_y_flipped(&self) -> bool {
        self.attributes & 0x40 != 0
    }

    /// Get sprite palette number (DMG only)
    ///
    /// Returns 0 for OBP0, 1 for OBP1
    pub const fn palette(&self) -> u8 {
        (self.attributes & 0x10) >> 4
    }

    /// Get actual Y position on screen
    ///
    /// Y position is stored as position minus 16
    pub const fn y_position(&self) -> i32 {
        (self.y_pos as i32) - 16
    }

    /// Get actual X position on screen
    ///
    /// X position is stored as position minus 8
    pub const fn x_position(&self) -> i32 {
        (self.x_pos as i32) - 8
    }

    /// Check if sprite is visible on the given scanline with the specified height
    ///
    /// Hardware: Sprite is visible if scanline is within [Y, Y+height). Y is y_pos-16 (see Pandocs).
    pub fn is_visible_on_scanline(&self, scanline: u8, sprite_height: i32) -> bool {
        let y_pos = self.y_position();
        y_pos <= i32::from(scanline) && (y_pos + sprite_height) > i32::from(scanline)
    }
}

/// Collect up to 10 visible sprites for this scanline, in OAM order
///
/// Hardware-accurate: Only the first 10 visible sprites (lowest OAM index) are used per scanline (see Pandocs).
/// This function enforces the hardware limit and OAM priority order.
pub fn collect_visible_sprites(oam: &[u8], ly: u8, sprite_height: i32) -> OamScanResult {
    let mut result = OamScanResult::new();

    for i in 0..40 {
        if result.count >= MAX_SPRITES_PER_LINE {
            break;
        }

        let sprite = Sprite::from_oam(oam, i);
        if sprite.is_visible_on_scanline(ly, sprite_height) {
            result.add_sprite(sprite);
        }
    }

    result
}

/// For a given x, find the topmost sprite pixel and its properties
///
/// Returns the first non-transparent sprite pixel at x for the current scanline, in OAM order (lowest index = highest priority).
/// Handles:
///   - X/Y flipping (bit 5/6)
///   - Palette selection (DMG: OBP0/OBP1, CGB: OBJ palette 0-7)
///   - CGB attributes (priority, VRAM bank, palette)
///   - Transparency (colour 0 is always transparent and not drawn)
///   - 8x8 and 8x16 sprites (tile index handling)
///
/// Hardware: When sprites overlap, the one with the lower OAM index is drawn in front (see Pandocs: Sprite Priorities and Conflicts).
/// Only non-zero colour indices are visible (colour 0 is transparent). CGB OBJ attributes are handled per Pandocs.
///
/// Rationale:
///   - OAM order is respected for priority (lowest index wins).
///   - Flipping and palette logic matches hardware for both DMG and CGB.
///   - VRAM bank and palette selection are CGB-only features.
///   - 8x16 sprites use even tile index (see Pandocs: Sprite Patterns).
///   - All bounds checks are defensive for robustness.
pub fn sprite_pixel_for_x(
    x: usize,
    ly: u8,
    sprite_height: i32,
    scan_result: &OamScanResult,
    vram: &[u8],
    is_cgb: bool,
    obp0: u8,
    obp1: u8,
    obp_data: &[u8; 64],
) -> Option<(usize, u8, Color, bool, bool)> {
    let x_i32 = i32::try_from(x).unwrap_or(0);

    for (oam_index, sprite_opt) in scan_result.sprites.iter().enumerate() {
        let Some(sprite) = sprite_opt else { continue };
        let x_pos = sprite.x_position();

        if x_pos <= x_i32 && x_i32 < x_pos + 8 {
            let (palette_num, vram_bank, x_flip, y_flip, cgb_priority) = if is_cgb {
                unpack_tile_attributes(sprite.attributes)
            } else {
                (
                    0,
                    0,
                    sprite.is_x_flipped(),
                    sprite.is_y_flipped(),
                    sprite.has_priority(),
                )
            };

            let mut line = u8::try_from(i32::from(ly) - sprite.y_position()).unwrap_or(0);
            if y_flip {
                let max_line = match sprite_height {
                    16 => 15,
                    _ => 7,
                };
                line = max_line - line;
            }

            let tile_addr = if sprite_height == 16 {
                (u16::from(sprite.tile_idx & 0xFE) * 16) + u16::from(line & 0xF) * 2
            } else {
                u16::from(sprite.tile_idx) * 16 + u16::from(line) * 2
            };

            let vram_offset = if is_cgb && vram_bank == 1 { 0x2000 } else { 0 };

            let low_byte = vram[vram_offset + tile_addr as usize];
            let high_byte = vram[vram_offset + (tile_addr + 1) as usize];

            let screen_x = x_i32 - x_pos;
            let bit = if x_flip {
                u8::try_from(screen_x).unwrap_or(0)
            } else {
                7 - u8::try_from(screen_x).unwrap_or(0)
            };

            let color_idx = extract_colour_index(low_byte, high_byte, bit);
            if color_idx == 0 {
                continue;
            }

            let color = if is_cgb {
                cgb_obj_color(color_idx, palette_num, obp_data)
            } else {
                let palette = if sprite.palette() == 0 { obp0 } else { obp1 };
                Color::from_palette(color_idx, palette)
            };

            return Some((
                oam_index,
                color_idx,
                color,
                sprite.has_priority(),
                cgb_priority,
            ));
        }
    }

    None
}

/// Lookup a CGB OBJ palette colour from palette data.
///
/// - `color_idx`: Colour index (0-3)
/// - `palette_num`: Palette number (0-7)
/// - `obp_data`: Object palette data (64 bytes)
///
/// Returns a 24-bit RGB `Color`.
pub fn cgb_obj_color(color_idx: u8, palette_num: u8, obp_data: &[u8; 64]) -> Color {
    let base = (palette_num as usize) * 8 + (color_idx as usize) * 2;
    let lo = obp_data.get(base).copied().unwrap_or(0);
    let hi = obp_data.get(base + 1).copied().unwrap_or(0);
    let rgb15 = (u16::from(hi) << 8) | u16::from(lo);
    let r = u8::try_from((rgb15 & 0x1F) << 3).unwrap_or(0);
    let g = u8::try_from(((rgb15 >> 5) & 0x1F) << 3).unwrap_or(0);
    let b = u8::try_from(((rgb15 >> 10) & 0x1F) << 3).unwrap_or(0);
    Color { r, g, b, a: 0xFF }
}
