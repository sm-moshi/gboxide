use crate::helpers::{extract_colour_index, unpack_tile_attributes};
use crate::ppu::color::Color;
use crate::ppu::ppu::Sprite;
use crate::ppu::MAX_SPRITES_PER_LINE;

/// Collect up to 10 visible sprites for this scanline, in OAM order
pub(crate) fn collect_visible_sprites(
    ppu: &crate::ppu::Ppu,
    sprite_height: i32,
) -> [Option<Sprite>; MAX_SPRITES_PER_LINE] {
    let mut visible_sprites: [Option<Sprite>; MAX_SPRITES_PER_LINE] = [None; MAX_SPRITES_PER_LINE];
    let mut count = 0;
    for i in 0..40 {
        if count >= MAX_SPRITES_PER_LINE {
            break;
        }
        let sprite = Sprite::from_oam(&ppu.oam, i);
        let y_pos = sprite.y_position();
        if y_pos <= i32::from(ppu.ly) && (y_pos + sprite_height) > i32::from(ppu.ly) {
            visible_sprites[count] = Some(sprite);
            count += 1;
        }
    }
    visible_sprites
}

/// For a given x, find the topmost sprite pixel and its properties
pub(crate) fn sprite_pixel_for_x(
    x: usize,
    ly: u8,
    sprite_height: i32,
    visible_sprites: &[Option<Sprite>; MAX_SPRITES_PER_LINE],
    ppu: &crate::ppu::Ppu,
) -> Option<(usize, u8, Color, bool, bool)> {
    let x_i32 = i32::try_from(x).unwrap_or(0);
    for (oam_index, sprite_opt) in visible_sprites.iter().enumerate() {
        let Some(sprite) = sprite_opt else { continue };
        let x_pos = sprite.x_position();
        if x_pos <= x_i32 && x_i32 < x_pos + 8 {
            let (palette_num, vram_bank, x_flip, y_flip, cgb_priority) = if ppu.is_cgb {
                unpack_tile_attributes(sprite.attributes)
            } else {
                (
                    0,
                    0,
                    sprite.is_x_flipped(),
                    sprite.is_y_flipped(),
                    !sprite.has_priority(),
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
            let vram_offset = if ppu.is_cgb && vram_bank == 1 {
                0x2000
            } else {
                0
            };
            let low_byte = ppu.vram[vram_offset + tile_addr as usize];
            let high_byte = ppu.vram[vram_offset + (tile_addr + 1) as usize];
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
            let color = if ppu.is_cgb {
                cgb_obj_color(color_idx, palette_num, &ppu.obp_data)
            } else {
                let palette = if sprite.palette() == 0 {
                    ppu.obp0
                } else {
                    ppu.obp1
                };
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

/// Lookup a CGB BG palette colour from palette data.
///
/// - `color_idx`: Colour index (0-3)
/// - `palette_num`: Palette number (0-7)
/// - `bgp_data`: Background palette data (64 bytes)
///
/// Returns a 24-bit RGB `Color`.
pub(crate) fn cgb_bg_color(color_idx: u8, palette_num: u8, bgp_data: &[u8; 64]) -> Color {
    let base = (palette_num as usize) * 8 + (color_idx as usize) * 2;
    let lo = bgp_data.get(base).copied().unwrap_or(0);
    let hi = bgp_data.get(base + 1).copied().unwrap_or(0);
    let rgb15 = (u16::from(hi) << 8) | u16::from(lo);
    let r = u8::try_from((rgb15 & 0x1F) << 3).unwrap_or(0);
    let g = u8::try_from(((rgb15 >> 5) & 0x1F) << 3).unwrap_or(0);
    let b = u8::try_from(((rgb15 >> 10) & 0x1F) << 3).unwrap_or(0);
    Color { r, g, b, a: 0xFF }
}

/// Lookup a CGB OBJ palette colour from palette data.
///
/// - `color_idx`: Colour index (0-3)
/// - `palette_num`: Palette number (0-7)
/// - `obp_data`: Object palette data (64 bytes)
///
/// Returns a 24-bit RGB `Color`.
pub(crate) fn cgb_obj_color(color_idx: u8, palette_num: u8, obp_data: &[u8; 64]) -> Color {
    let base = (palette_num as usize) * 8 + (color_idx as usize) * 2;
    let lo = obp_data.get(base).copied().unwrap_or(0);
    let hi = obp_data.get(base + 1).copied().unwrap_or(0);
    let rgb15 = (u16::from(hi) << 8) | u16::from(lo);
    let r = u8::try_from((rgb15 & 0x1F) << 3).unwrap_or(0);
    let g = u8::try_from(((rgb15 >> 5) & 0x1F) << 3).unwrap_or(0);
    let b = u8::try_from(((rgb15 >> 10) & 0x1F) << 3).unwrap_or(0);
    Color { r, g, b, a: 0xFF }
}

/// Set a bit in a bitfield buffer (CGB BG priority buffer).
///
/// - `buffer`: Bitfield buffer (slice of u8)
/// - `idx`: Pixel index
/// - `value`: Bit value to set (true = 1, false = 0)
pub(crate) fn set_bg_priority(buffer: &mut [u8], idx: usize, value: bool) {
    let byte = idx / 8;
    let bit = idx % 8;
    if let Some(b) = buffer.get_mut(byte) {
        if value {
            *b |= 1 << bit;
        } else {
            *b &= !(1 << bit);
        }
    }
}

/// Get a bit from a bitfield buffer (CGB BG priority buffer).
///
/// - `buffer`: Bitfield buffer (slice of u8)
/// - `idx`: Pixel index
///
/// Returns true if the bit is set, false otherwise.
pub(crate) fn get_bg_priority(buffer: &[u8], idx: usize) -> bool {
    let byte = idx / 8;
    let bit = idx % 8;
    buffer.get(byte).is_some_and(|b| (*b & (1 << bit)) != 0)
}
