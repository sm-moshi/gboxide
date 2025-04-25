use crate::helpers::{extract_colour_index, unpack_tile_attributes};
use crate::ppu::color::Color;
use crate::ppu::sprite::Sprite;
use crate::ppu::MAX_SPRITES_PER_LINE;

/// Collect up to 10 visible sprites for this scanline, in OAM order
///
/// Hardware-accurate: Only the first 10 visible sprites (lowest OAM index) are used per scanline (see Pandocs).
/// This function enforces the hardware limit and OAM priority order. Sprites are visible if the scanline is within [Y, Y+height).
#[allow(dead_code)]
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
///
/// Returns the first non-transparent sprite pixel at x for the current scanline, in OAM order (lowest index = highest priority).
/// Handles X/Y flipping, palette selection, CGB attributes, and transparency (colour 0 is always transparent).
///
/// Hardware: When sprites overlap, the one with the lower OAM index is drawn in front (see Pandocs). Only non-zero colour indices are visible.
#[allow(dead_code)]
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
            let low_addr = vram_offset + tile_addr as usize;
            let high_addr = vram_offset + (tile_addr + 1) as usize;
            let low_byte = ppu.vram[low_addr];
            let high_byte = ppu.vram[high_addr];
            eprintln!("[DEBUG VRAM ACCESS] vram_bank={}, tile_addr=0x{:04X}, low_addr=0x{:04X}, high_addr=0x{:04X}, low_byte={:08b}, high_byte={:08b}", vram_bank, tile_addr, low_addr, high_addr, low_byte, high_byte);
            let screen_x = x_i32 - x_pos;
            let bit = if x_flip {
                u8::try_from(screen_x).unwrap_or(0)
            } else {
                7 - u8::try_from(screen_x).unwrap_or(0)
            };
            let color_idx = extract_colour_index(low_byte, high_byte, bit);
            if color_idx == 0 {
                eprintln!("[DEBUG sprite_pixel_for_x] color_idx==0: oam_index={oam_index}, x={x}, ly={ly}, sprite_height={sprite_height}, x_pos={x_pos}, y_pos={}, tile_addr={}, vram_offset={}, low_byte={:08b}, high_byte={:08b}, bit={}, attributes={:08b}", sprite.y_position(), tile_addr, vram_offset, low_byte, high_byte, bit, sprite.attributes);
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
            eprintln!("[DEBUG sprite_pixel_for_x] RETURNING: oam_index={oam_index}, x={x}, ly={ly}, sprite_height={sprite_height}, x_pos={x_pos}, y_pos={}, tile_addr={}, vram_offset={}, low_byte={:08b}, high_byte={:08b}, bit={}, color_idx={}, color={:?}, attributes={:08b}", sprite.y_position(), tile_addr, vram_offset, low_byte, high_byte, bit, color_idx, color, sprite.attributes);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppu::ppu::Ppu;
    use pretty_assertions::assert_eq;

    fn dummy_ppu() -> Ppu {
        Ppu::new()
    }

    #[test]
    fn test_collect_visible_sprites_none() {
        let ppu = dummy_ppu();
        let sprites = collect_visible_sprites(&ppu, 8);
        assert!(sprites.iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_collect_visible_sprites_some() {
        let mut ppu = dummy_ppu();
        // Place a sprite at LY=0, visible
        ppu.oam[0] = 16; // y_pos = 0
        ppu.oam[1] = 8; // x_pos = 0
        ppu.oam[2] = 0; // tile_idx
        ppu.oam[3] = 0; // attributes
        ppu.ly = 0;
        let sprites = collect_visible_sprites(&ppu, 8);
        assert!(sprites[0].is_some());
        assert!(sprites[1..].iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_collect_visible_sprites_cap() {
        let mut ppu = dummy_ppu();
        // Place 12 visible sprites at LY=0
        for i in 0..12 {
            let base = i * 4;
            ppu.oam[base] = 16; // y_pos = 0
            ppu.oam[base + 1] = 8; // x_pos = 0
            ppu.oam[base + 2] = 0;
            ppu.oam[base + 3] = 0;
        }
        ppu.ly = 0;
        let sprites = collect_visible_sprites(&ppu, 8);
        // Only 10 should be collected
        assert_eq!(sprites.iter().filter(|s| s.is_some()).count(), 10);
    }

    #[test]
    fn test_sprite_pixel_for_x_dmg_and_cgb() {
        let mut ppu = dummy_ppu();
        // Enable LCD and sprites
        use crate::ppu::lcdc::LcdControl;
        ppu.lcdc =
            LcdControl::LCD_ENABLE | LcdControl::SPRITE_ENABLE | LcdControl::BG_WINDOW_ENABLE;
        ppu.obp0 = 0xE4; // Standard DMG palette

        // DMG mode
        ppu.oam[0] = 16; // y_pos = 0
        ppu.oam[1] = 8; // x_pos = 0
        ppu.oam[2] = 0; // tile_idx
        ppu.oam[3] = 0; // attributes
        ppu.ly = 0;
        // Set VRAM so that bit 7 of low_byte is 1 (pixel at x=0)
        ppu.vram[0] = 0b1000_0000; // low byte: pixel at x=0 is visible
        ppu.vram[1] = 0x00; // high byte
        let visible = collect_visible_sprites(&ppu, 8);
        let result = sprite_pixel_for_x(0, 0, 8, &visible, &ppu);
        if result.is_none() {
            eprintln!(
                "[DMG] result: {:?}, vram[0]={:08b}, vram[1]={:08b}, oam={:?}, lcdc={:?}",
                result,
                ppu.vram[0],
                ppu.vram[1],
                &ppu.oam[0..4],
                ppu.lcdc
            );
        }
        assert!(result.is_some());

        // CGB mode, with flipping
        ppu.is_cgb = true;
        // Attributes: vbank=1, yflip=true, xflip=true, pal=0
        ppu.oam[3] = 0b0110_1000;

        // Calculate target VRAM address based on attributes
        // line = 7 (ly=0, y_pos=0, yflip=true)
        // tile_addr = 0*16 + 7*2 = 0x0E
        // vram_offset = 0x2000 (vbank=1)
        let target_addr = 0x2000 + 0x0E;

        // Set VRAM so that bit 0 of low_byte at target_addr is 1
        // (pixel at x=0 with x_flip)
        ppu.vram[target_addr] = 0b0000_0001; // low byte for tile 0, line 7, in VRAM bank 1
        ppu.vram[target_addr + 1] = 0x00; // high byte

        let visible_cgb = collect_visible_sprites(&ppu, 8);
        let result_cgb = sprite_pixel_for_x(0, 0, 8, &visible_cgb, &ppu);

        // --- Begin CGB Debug Print ---
        // Removed debug prints as test now passes
        // --- End CGB Debug Print ---

        assert!(result_cgb.is_some(), "CGB sprite pixel should be visible");
    }

    #[test]
    fn test_sprite_pixel_for_x_color_idx_zero() {
        let mut ppu = dummy_ppu();
        ppu.oam[0] = 16; // y_pos = 0
        ppu.oam[1] = 8; // x_pos = 0
        ppu.oam[2] = 0; // tile_idx
        ppu.oam[3] = 0; // attributes
        ppu.ly = 0;
        ppu.vram[0] = 0x00; // low byte (color_idx = 0)
        ppu.vram[1] = 0x00; // high byte
        let visible = collect_visible_sprites(&ppu, 8);
        let result = sprite_pixel_for_x(0, 0, 8, &visible, &ppu);
        assert!(result.is_none());
    }

    #[test]
    fn test_cgb_palette_bounds() {
        let mut data = [0u8; 64];
        data[63] = 0xFF;
        let c = cgb_bg_color(3, 7, &data);
        let c2 = cgb_obj_color(3, 7, &data);
        // Should not panic, should return a Color
        assert_eq!(c.a, 0xFF);
        assert_eq!(c2.a, 0xFF);
    }

    #[test]
    fn test_cgb_palette_out_of_bounds() {
        let data = [0u8; 2];
        // Should not panic, should return a Color with all zeroes except alpha
        let c = cgb_bg_color(3, 7, unsafe { &*(data.as_ptr() as *const [u8; 64]) });
        assert_eq!(c.r, 0);
        assert_eq!(c.a, 0xFF);
    }

    #[test]
    fn test_set_get_bg_priority_boundaries() {
        let mut buffer = [0u8; 2];
        set_bg_priority(&mut buffer, 15, true);
        assert!(get_bg_priority(&buffer, 15));
        set_bg_priority(&mut buffer, 15, false);
        assert!(!get_bg_priority(&buffer, 15));
    }

    #[test]
    fn test_set_get_bg_priority_multiple_bits() {
        let mut buffer = [0u8; 2];
        set_bg_priority(&mut buffer, 1, true);
        set_bg_priority(&mut buffer, 2, true);
        assert!(get_bg_priority(&buffer, 1));
        assert!(get_bg_priority(&buffer, 2));
        set_bg_priority(&mut buffer, 1, false);
        assert!(!get_bg_priority(&buffer, 1));
        assert!(get_bg_priority(&buffer, 2));
    }
}

#[cfg(test)]
mod extra_coverage {
    use super::*;
    use crate::ppu::ppu::Ppu;
    use pretty_assertions::assert_eq;

    fn dummy_ppu() -> Ppu {
        Ppu::new()
    }

    #[test]
    fn test_sprite_pixel_for_x_8x16_and_flipping() {
        let mut ppu = dummy_ppu();
        ppu.lcdc =
            crate::ppu::lcdc::LcdControl::LCD_ENABLE | crate::ppu::lcdc::LcdControl::SPRITE_ENABLE;
        // 8x16 sprite, no flip
        ppu.oam[0] = 16; // y_pos = 0
        ppu.oam[1] = 8; // x_pos = 0
        ppu.oam[2] = 0; // tile_idx
        ppu.oam[3] = 0; // attributes
        ppu.ly = 8; // second tile row
                    // Set VRAM for lower tile
        ppu.vram[16] = 0b1000_0000; // low byte for tile 1, line 0
        ppu.vram[17] = 0x00;
        let visible = collect_visible_sprites(&ppu, 16);
        let result = sprite_pixel_for_x(0, 8, 16, &visible, &ppu);
        if result.is_none() {
            eprintln!(
                "[8x16] OAM: {:?}, VRAM[16]: {:08b}, VRAM[17]: {:08b}, visible: {:?}",
                &ppu.oam[0..4],
                ppu.vram[16],
                ppu.vram[17],
                visible
            );
        }
        assert!(result.is_some());
        // 8x16 sprite, y flip
        ppu.oam[3] = 0b0100_0000; // y flip
        ppu.vram[0] = 0b1000_0000; // low byte for tile 1, line 0 (yflip: line=0)
        let visible = collect_visible_sprites(&ppu, 16);
        let result = sprite_pixel_for_x(0, 15, 16, &visible, &ppu);
        if result.is_none() {
            eprintln!(
                "[8x16 yflip] OAM: {:?}, VRAM[0]: {:08b}, visible: {:?}",
                &ppu.oam[0..4],
                ppu.vram[0],
                visible
            );
        }
        assert!(result.is_some());
    }

    #[test]
    fn test_sprite_pixel_for_x_cgb_all_attributes() {
        let mut ppu = dummy_ppu();
        ppu.is_cgb = true;
        ppu.oam[0] = 16; // y_pos = 0
        ppu.oam[1] = 8; // x_pos = 0 (x_position = 0)
        ppu.oam[2] = 0; // tile_idx
                        // Attributes: vram bank 1, palette 3, x flip, y flip, priority
        ppu.oam[3] = 0b1110_1111;
        ppu.ly = 0;
        let target_addr = 0x2000 + 0x0E;
        ppu.vram[target_addr] = 0b0000_0001;
        ppu.vram[target_addr + 1] = 0x00;
        let visible = collect_visible_sprites(&ppu, 8);
        let result = sprite_pixel_for_x(0, 0, 8, &visible, &ppu);
        if result.is_none() {
            eprintln!(
                "[CGB attr] OAM: {:?}, VRAM[0x200E]: {:08b}, visible: {:?}",
                &ppu.oam[0..4],
                ppu.vram[target_addr],
                visible
            );
        }
        assert!(result.is_some());
        let (_idx, _color_idx, _color, _dmg_prio, cgb_prio) = result.unwrap();
        assert!(cgb_prio); // priority bit set
    }

    #[test]
    fn test_bg_priority_buffer_out_of_bounds() {
        let mut buffer = [0u8; 1];
        // Should not panic, should be a no-op
        set_bg_priority(&mut buffer, 100, true);
        assert!(!get_bg_priority(&buffer, 100));
    }

    #[test]
    fn test_overlapping_sprites_oam_priority() {
        let mut ppu = dummy_ppu();
        ppu.lcdc =
            crate::ppu::lcdc::LcdControl::LCD_ENABLE | crate::ppu::lcdc::LcdControl::SPRITE_ENABLE;
        // Sprite 0 at x=8, Sprite 1 at x=8 (overlap)
        ppu.oam[0] = 16;
        ppu.oam[1] = 8;
        ppu.oam[2] = 0;
        ppu.oam[3] = 0;
        ppu.oam[4] = 16;
        ppu.oam[5] = 8;
        ppu.oam[6] = 0;
        ppu.oam[7] = 0;
        ppu.ly = 0;
        ppu.vram[0] = 0b1000_0000;
        ppu.vram[1] = 0x00;
        let visible = collect_visible_sprites(&ppu, 8);
        let result = sprite_pixel_for_x(0, 0, 8, &visible, &ppu);
        assert!(result.is_some());
        let (idx, _, _, _, _) = result.unwrap();
        assert_eq!(idx, 0); // Lower OAM index wins
    }

    #[test]
    fn test_sprite_pixel_for_x_invalid_vram_palette_indices() {
        let mut ppu = dummy_ppu();
        ppu.is_cgb = true;
        ppu.oam[0] = 16;
        ppu.oam[1] = 8;
        ppu.oam[2] = 0;
        ppu.oam[3] = 0b0000_0111; // palette 7
        ppu.ly = 0;
        // No VRAM set, palette data is all zero
        let visible = collect_visible_sprites(&ppu, 8);
        let result = sprite_pixel_for_x(0, 0, 8, &visible, &ppu);
        // Should not panic, should return a Color with alpha=0xFF
        if let Some((_idx, _color_idx, color, _, _)) = result {
            assert_eq!(color.a, 0xFF);
        }
    }
}
