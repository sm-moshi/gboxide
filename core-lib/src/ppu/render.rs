use super::color::Color;
use super::helpers::{get_bg_priority, set_bg_priority};
use super::lcdc::LcdControl;
use super::ppu::Ppu;
use super::SCREEN_WIDTH;
use crate::helpers::{extract_colour_index, tile_data_address, unpack_tile_attributes};

/// Renderer encapsulates all scanline rendering logic for the PPU.
pub(crate) struct Renderer;

impl Renderer {
    /// Render the current scanline to the frame buffer
    pub fn render_scanline(&self, ppu: &mut Ppu) {
        if !ppu.lcdc.contains(LcdControl::LCD_ENABLE) {
            // LCD is disabled, fill with white
            let start = ppu.ly as usize * SCREEN_WIDTH;
            let end = start + SCREEN_WIDTH;
            ppu.frame_buffer[start..end].fill(Color::WHITE);
            return;
        }

        // Render background and window first (if enabled)
        if ppu.lcdc.contains(LcdControl::BG_WINDOW_ENABLE) {
            self.render_background(ppu);
            // Render window if enabled
            if ppu.lcdc.contains(LcdControl::WINDOW_ENABLE) {
                self.render_window(ppu);
            }
        } else {
            // If background is disabled, fill with white
            let start = ppu.ly as usize * SCREEN_WIDTH;
            let end = start + SCREEN_WIDTH;
            ppu.frame_buffer[start..end].fill(Color::WHITE);
        }

        // Render sprites if enabled
        if ppu.lcdc.contains(LcdControl::SPRITE_ENABLE) {
            self.render_sprites(ppu);
        }
    }

    /// Render the background for the current scanline
    pub fn render_background(&self, ppu: &mut Ppu) {
        let tilemap_base = if ppu.lcdc.contains(LcdControl::BG_TILEMAP) {
            0x1C00u16
        } else {
            0x1800u16
        };
        let tiledata_base = if ppu.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA) {
            0x0000u16
        } else {
            0x1000u16
        };
        let signed = !ppu.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA);
        let y_pos = (u16::from(ppu.ly) + u16::from(ppu.scy)) & 0xFF;
        let tile_y = y_pos / 8;
        let tile_y_offset = y_pos % 8;
        let scanline_offset = ppu.ly as usize * SCREEN_WIDTH;
        for x in 0..SCREEN_WIDTH {
            let x_pos = (u16::try_from(x).unwrap_or(0) + u16::from(ppu.scx)) & 0xFF;
            let tile_x = x_pos / 8;
            let tile_x_offset = x_pos % 8;
            let tile_map_addr = tilemap_base + tile_y * 32 + tile_x;
            let tile_id = ppu.vram[tile_map_addr as usize];
            let (palette_num, vram_bank, x_flip, _y_flip, priority) = if ppu.is_cgb {
                let attr = ppu
                    .vram
                    .get(tile_map_addr as usize + 0x2000)
                    .copied()
                    .unwrap_or(0);
                unpack_tile_attributes(attr)
            } else {
                (0, 0, false, false, false)
            };
            let tile_addr = tile_data_address(tiledata_base, tile_id, signed);
            let vram_offset = if ppu.is_cgb && vram_bank == 1 {
                0x2000
            } else {
                0
            };
            let tile_line = tile_y_offset;
            let tile_line_addr = tile_addr + tile_line * 2;
            let tile_line_addr_usize = usize::from(tile_line_addr);
            let tile_data_low = ppu.vram[vram_offset + tile_line_addr_usize];
            let tile_data_high = ppu.vram[vram_offset + tile_line_addr_usize + 1];

            // For x_flip=true, we read pixels right-to-left (0 to 7)
            // For x_flip=false, we read pixels left-to-right (7 to 0)
            let bit = u8::try_from(if x_flip {
                7 - tile_x_offset
            } else {
                tile_x_offset
            })
            .unwrap_or(0);

            let color_idx = extract_colour_index(tile_data_low, tile_data_high, bit);
            let color = if ppu.is_cgb {
                super::helpers::cgb_bg_color(color_idx, palette_num, &ppu.bgp_data)
            } else {
                Color::from_palette(color_idx, ppu.bgp)
            };
            ppu.frame_buffer[scanline_offset + x] = color;
            if ppu.is_cgb {
                set_bg_priority(ppu.bg_priority_buffer_mut(), scanline_offset + x, priority);
            }
        }
    }

    /// Render the window for the current scanline
    ///
    /// Hardware-accurate: The window is only drawn if LY >= WY and WX <= 166 (see Pandocs).
    /// WX is offset by 7; window appears at (WX-7, WY). The window line counter increments only when the window is drawn.
    /// CGB attributes are handled per-tile as in background rendering.
    pub fn render_window(&self, ppu: &mut Ppu) {
        // Window is only visible if LY >= WY and WX <= 166
        if ppu.ly < ppu.wy || ppu.wx > 166 {
            return;
        }
        let window_tilemap_base = if ppu.lcdc.contains(LcdControl::WINDOW_TILEMAP) {
            0x1C00u16
        } else {
            0x1800u16
        };
        let tiledata_base = if ppu.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA) {
            0x0000u16
        } else {
            0x1000u16
        };
        let signed = !ppu.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA);
        let window_y = ppu.window_line;
        let tile_y = u16::from(window_y / 8);
        let tile_y_offset = window_y % 8;
        let scanline_offset = ppu.ly as usize * SCREEN_WIDTH;
        let window_x = ppu.wx as isize - 7; // Pandocs: WX is offset by 7
        let mut window_drawn = false;
        for x in 0..SCREEN_WIDTH {
            if isize::try_from(x).unwrap_or(0) < window_x {
                continue;
            }
            window_drawn = true;
            let window_x_pos =
                u16::try_from(isize::try_from(x).unwrap_or(0) - window_x).unwrap_or(0);
            let tile_x = window_x_pos / 8;
            let tile_x_offset = window_x_pos % 8;
            let tile_map_addr = window_tilemap_base + tile_y * 32 + tile_x;
            let tile_id = ppu.vram[tile_map_addr as usize];
            let (palette_num, vram_bank, x_flip, _y_flip, priority) = if ppu.is_cgb {
                let attr = ppu
                    .vram
                    .get(tile_map_addr as usize + 0x2000)
                    .copied()
                    .unwrap_or(0);
                unpack_tile_attributes(attr)
            } else {
                (0, 0, false, false, false)
            };

            // In CGB mode, always use the 8000 addressing mode (unsigned)
            let tile_addr = if ppu.is_cgb {
                u16::from(tile_id) * 16
            } else {
                tile_data_address(tiledata_base, tile_id, signed)
            };

            let vram_offset = if ppu.is_cgb && vram_bank == 1 {
                0x2000
            } else {
                0
            };

            let tile_line = if _y_flip {
                u16::from(7 - tile_y_offset)
            } else {
                u16::from(tile_y_offset)
            };
            let tile_line_addr = tile_addr + tile_line * 2;
            let tile_line_addr_usize = usize::from(tile_line_addr);

            let tile_data_low = ppu.vram[vram_offset + tile_line_addr_usize];
            let tile_data_high = ppu.vram[vram_offset + tile_line_addr_usize + 1];

            // For x_flip=true, we read pixels right-to-left (0 to 7)
            // For x_flip=false, we read pixels left-to-right (7 to 0)
            let bit = u8::try_from(if x_flip {
                7 - tile_x_offset
            } else {
                tile_x_offset
            })
            .unwrap_or(0);

            let color_idx = extract_colour_index(tile_data_low, tile_data_high, bit);
            let color = if ppu.is_cgb {
                super::helpers::cgb_bg_color(color_idx, palette_num, &ppu.bgp_data)
            } else {
                Color::from_palette(color_idx, ppu.bgp)
            };
            ppu.frame_buffer[scanline_offset + x] = color;
            if ppu.is_cgb {
                set_bg_priority(ppu.bg_priority_buffer_mut(), scanline_offset + x, priority);
            }
        }
        // Only increment window_line if the window was drawn on this scanline
        if window_drawn {
            ppu.window_line = ppu.window_line.wrapping_add(1);
        }
    }

    /// Render sprites for the current scanline
    ///
    /// Hardware-accurate priority rules:
    /// - DMG: Sprite priority bit (bit 7) set means sprite is behind BG unless BG colour is 0 (white).
    /// - CGB: BG priority bit (from BG tile attr) set means sprite is only drawn if BG colour is 0 (white).
    /// Otherwise, CGB OBJ priority bit (attr bit 7) controls if sprite is drawn over BG (false = in front).
    /// Lower OAM index always wins for overlapping sprites.
    pub fn render_sprites(&self, ppu: &mut Ppu) {
        if !ppu.lcdc.contains(LcdControl::SPRITE_ENABLE) {
            return;
        }
        let sprite_height = if ppu.lcdc.contains(LcdControl::SPRITE_SIZE) {
            16
        } else {
            8
        };
        // For each pixel, collect all OAM indices that would draw a non-transparent pixel
        for x in 0..SCREEN_WIDTH {
            let mut contributors = Vec::new();
            for (oam_index, sprite_opt) in ppu.oam_scan_result.sprites.iter().enumerate() {
                let Some(sprite) = sprite_opt else { continue };
                let x_pos = sprite.x_position();
                if x_pos <= (x as i32) && (x as i32) < (x_pos + 8) {
                    // Calculate which line of the sprite this is
                    let mut line =
                        u8::try_from(i32::from(ppu.ly) - sprite.y_position()).unwrap_or(0);
                    let y_flip = sprite.is_y_flipped();
                    if y_flip {
                        let max_line = if sprite_height == 16 { 15 } else { 7 };
                        line = max_line - line;
                    }
                    let tile_addr = if sprite_height == 16 {
                        (u16::from(sprite.tile_idx & 0xFE) * 16) + u16::from(line & 0xF) * 2
                    } else {
                        u16::from(sprite.tile_idx) * 16 + u16::from(line) * 2
                    };
                    let vram_offset = if ppu.is_cgb && (sprite.attributes & 0x08) != 0 {
                        0x2000
                    } else {
                        0
                    };
                    let low_addr = vram_offset + tile_addr as usize;
                    let high_addr = vram_offset + (tile_addr + 1) as usize;
                    let low_byte = ppu.vram[low_addr];
                    let high_byte = ppu.vram[high_addr];
                    let screen_x = x as i32 - x_pos;
                    let x_flip = sprite.is_x_flipped();
                    let bit = if x_flip {
                        u8::try_from(screen_x).unwrap_or(0)
                    } else {
                        7 - u8::try_from(screen_x).unwrap_or(0)
                    };
                    let color_idx = extract_colour_index(low_byte, high_byte, bit);
                    if color_idx != 0 {
                        contributors.push(oam_index);
                    }
                }
            }
            // Print contributors for X=8..15 for debugging
            if (8..16).contains(&x) {
                println!("[DEBUG] x={} contributors={:?}", x, contributors);
            }
            if contributors.len() > 1 {
                ppu.record_sprite_collision(ppu.ly, x as u8, contributors.clone());
            }
        }
        // Now do the normal rendering (topmost sprite drawn)
        for x in 0..SCREEN_WIDTH {
            if let Some((oam_index, _color_idx, color, dmg_priority, cgb_priority)) =
                super::sprite::sprite_pixel_for_x(
                    x,
                    ppu.ly,
                    sprite_height,
                    &ppu.oam_scan_result,
                    &ppu.vram,
                    ppu.is_cgb,
                    ppu.obp0,
                    ppu.obp1,
                    &ppu.obp_data,
                )
            {
                let pixel_index = ppu.ly as usize * SCREEN_WIDTH + x;
                let bg_color = ppu.frame_buffer[pixel_index];
                let bg_is_white = bg_color == Color::WHITE;
                let bg_priority = if ppu.is_cgb {
                    get_bg_priority(ppu.bg_priority_buffer_mut(), pixel_index)
                } else {
                    false
                };
                if ppu.is_cgb {
                    if bg_priority {
                        if bg_is_white {
                            ppu.frame_buffer[pixel_index] = color;
                        }
                    } else {
                        if !cgb_priority || bg_is_white {
                            ppu.frame_buffer[pixel_index] = color;
                        }
                    }
                } else {
                    if !dmg_priority || bg_is_white {
                        ppu.frame_buffer[pixel_index] = color;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppu::lcdc::LcdControl;
    use crate::ppu::ppu::Ppu;
    use pretty_assertions::assert_eq;

    fn dummy_ppu() -> Ppu {
        Ppu::new()
    }

    #[test]
    fn test_window_not_drawn_ly_less_than_wy() {
        let mut ppu = dummy_ppu();
        ppu.lcdc =
            LcdControl::LCD_ENABLE | LcdControl::BG_WINDOW_ENABLE | LcdControl::WINDOW_ENABLE;
        ppu.ly = 5;
        ppu.wy = 10; // LY < WY
        ppu.wx = 7;
        let renderer = Renderer;
        let before = ppu.frame_buffer.clone();
        renderer.render_window(&mut ppu);
        // Frame buffer should be unchanged
        assert_eq!(ppu.frame_buffer, before);
    }

    #[test]
    fn test_window_not_drawn_wx_greater_than_166() {
        let mut ppu = dummy_ppu();
        ppu.lcdc =
            LcdControl::LCD_ENABLE | LcdControl::BG_WINDOW_ENABLE | LcdControl::WINDOW_ENABLE;
        ppu.ly = 10;
        ppu.wy = 5;
        ppu.wx = 167; // WX > 166
        let renderer = Renderer;
        let before = ppu.frame_buffer.clone();
        renderer.render_window(&mut ppu);
        assert_eq!(ppu.frame_buffer, before);
    }

    #[test]
    fn test_window_line_counter_increments_only_when_drawn() {
        let mut ppu = dummy_ppu();
        ppu.lcdc =
            LcdControl::LCD_ENABLE | LcdControl::BG_WINDOW_ENABLE | LcdControl::WINDOW_ENABLE;
        ppu.ly = 10;
        ppu.wy = 5;
        ppu.wx = 7;
        let renderer = Renderer;
        let before = ppu.window_line;
        renderer.render_window(&mut ppu);
        assert_eq!(ppu.window_line, before + 1);
        // Now set LY < WY, should not increment
        ppu.ly = 4;
        let before = ppu.window_line;
        renderer.render_window(&mut ppu);
        assert_eq!(ppu.window_line, before);
    }

    #[test]
    fn test_window_cgb_attributes_palette_and_flip() {
        let mut ppu = dummy_ppu();
        ppu.is_cgb = true;
        ppu.lcdc =
            LcdControl::LCD_ENABLE | LcdControl::BG_WINDOW_ENABLE | LcdControl::WINDOW_ENABLE;
        ppu.ly = 0;
        ppu.wy = 0;
        ppu.wx = 7;

        // Set up window tilemap and attributes
        let tile_id = 1; // Use tile 1 instead of 0
        let palette_num = 2;
        let vram_bank = 1;
        let x_flip = true;
        let y_flip = true;
        let priority = true;
        let attr = ((priority as u8) << 7)
            | ((y_flip as u8) << 6)
            | ((x_flip as u8) << 5)
            | ((vram_bank as u8) << 3)
            | palette_num;

        // Set tile ID in tilemap
        ppu.vram[0x1800] = tile_id;
        // Set attributes in VRAM bank 1
        ppu.vram[0x3800] = attr;

        // Set up tile data in VRAM bank 1 (since that's what the attribute specifies)
        // Create a simple pattern that will produce color index 3
        // Each tile is 16 bytes (8 rows × 2 bytes per row)
        let tile_offset = tile_id as usize * 16;

        // Fill all lines of the tile with the same pattern
        for line in 0..8 {
            let line_offset = tile_offset + (line * 2);
            ppu.vram[0x2000 + line_offset] = 0xFF; // Low byte
            ppu.vram[0x2000 + line_offset + 1] = 0xFF; // High byte
        }

        println!("[TEST] Tile data setup:");
        println!("  Tile ID: {tile_id}");
        println!("  VRAM bank: {vram_bank}");
        println!("  Tile offset: 0x{tile_offset:04x}");
        println!("  Attribute byte: 0b{attr:08b}");
        for line in 0..8 {
            let line_offset = tile_offset + (line * 2);
            println!(
                "  Line {line} data at 0x{:04x}: 0x{:02x}{:02x}",
                0x2000 + line_offset,
                ppu.vram[0x2000 + line_offset + 1],
                ppu.vram[0x2000 + line_offset]
            );
        }

        // Set up palette data for palette 2 (as specified in attributes)
        // Color 3 in palette 2 = RGB15 value 0xBBAA
        let color_idx = 3;
        let palette_offset = (palette_num as usize) * 8 + (color_idx * 2);
        ppu.bgp_data[palette_offset] = 0xAA; // Low byte
        ppu.bgp_data[palette_offset + 1] = 0xBB; // High byte

        let renderer = Renderer;
        renderer.render_window(&mut ppu);

        // The first pixel of the window should use palette 2, color 3
        let idx = (ppu.ly as usize) * SCREEN_WIDTH + (ppu.wx as usize - 7);
        let color = ppu.frame_buffer[idx];

        // Convert RGB15 0xBBAA to RGB24
        let expected_r = 0xAA & 0x1F;
        let expected_g = ((0xAA >> 5) | ((0xBB & 0x03) << 3)) & 0x1F;
        let expected_b = (0xBB >> 2) & 0x1F;

        println!("[TEST] Color values:");
        println!(
            "  Expected RGB: ({}, {}, {})",
            expected_r << 3,
            expected_g << 3,
            expected_b << 3
        );
        println!("  Actual RGB: ({}, {}, {})", color.r, color.g, color.b);
        println!("  Color index: {color_idx}");
        println!("  Palette number: {palette_num}");
        println!("  Palette offset: 0x{palette_offset:04x}");
        println!(
            "  Palette data: 0x{:02x}{:02x}",
            ppu.bgp_data[palette_offset + 1],
            ppu.bgp_data[palette_offset]
        );

        assert_eq!(color.r, (expected_r << 3) as u8, "Red component mismatch");
        assert_eq!(color.g, (expected_g << 3) as u8, "Green component mismatch");
        assert_eq!(color.b, (expected_b << 3) as u8, "Blue component mismatch");
    }

    #[test]
    fn test_window_at_screen_edges() {
        let mut ppu = dummy_ppu();
        ppu.lcdc =
            LcdControl::LCD_ENABLE | LcdControl::BG_WINDOW_ENABLE | LcdControl::WINDOW_ENABLE;
        ppu.ly = 0;
        ppu.wy = 0;
        let renderer = Renderer;
        // WX = 7 (leftmost)
        ppu.wx = 7;
        renderer.render_window(&mut ppu);
        // WX = 166 (rightmost)
        ppu.wx = 166;
        renderer.render_window(&mut ppu);
        // WY = 143 (bottom)
        ppu.ly = 143;
        ppu.wy = 143;
        renderer.render_window(&mut ppu);
    }
}
