use crate::helpers::{extract_colour_index, tile_data_address, unpack_tile_attributes};
/// core-lib/src/ppu/ppu.rs
use crate::interrupts::InterruptFlag;

use super::color::Color;
use super::lcdc::LcdControl;
use super::stat::LcdStatus;
use super::{PpuError, PpuMode, SCREEN_HEIGHT, SCREEN_WIDTH};
use super::{
    BGP_ADDR, LCDC_ADDR, LYC_ADDR, LY_ADDR, MAX_SPRITES_PER_LINE, OAM_SIZE, OBP0_ADDR, OBP1_ADDR,
    SCX_ADDR, SCY_ADDR, STAT_ADDR, VRAM_SIZE, WX_ADDR, WY_ADDR,
};

/// Sprite attributes structure (OAM entry)
#[derive(Debug, Clone, Copy, Default)]
pub struct Sprite {
    pub y_pos: u8,      // Y coordinate (minus 16)
    pub x_pos: u8,      // X coordinate (minus 8)
    pub tile_idx: u8,   // Tile index
    pub attributes: u8, // Attributes (priority, flip, palette)
}

bitflags::bitflags! {
    /// Sprite attribute flags
    #[derive(Debug, Clone, Copy)]
    pub struct SpriteAttributes: u8 {
        const PRIORITY = 0x80;
        const Y_FLIP = 0x40;
        const X_FLIP = 0x20;
        const PALETTE = 0x10;
    }
}

impl Sprite {
    /// Create a sprite from OAM data
    pub fn from_oam(oam: &[u8], index: usize) -> Self {
        let base = index * 4;
        Self {
            y_pos: oam[base],
            x_pos: oam[base + 1],
            tile_idx: oam[base + 2],
            attributes: oam[base + 3],
        }
    }

    /// Check if this sprite has priority over the background
    /// When this bit is 0, sprite has priority
    /// When this bit is 1, sprite is behind colors 1-3 of BG/Window
    pub const fn has_priority(&self) -> bool {
        !SpriteAttributes::from_bits_truncate(self.attributes).contains(SpriteAttributes::PRIORITY)
    }

    /// Check if this sprite is flipped horizontally (X)
    pub const fn is_x_flipped(&self) -> bool {
        SpriteAttributes::from_bits_truncate(self.attributes).contains(SpriteAttributes::X_FLIP)
    }

    /// Check if this sprite is flipped vertically (Y)
    pub const fn is_y_flipped(&self) -> bool {
        SpriteAttributes::from_bits_truncate(self.attributes).contains(SpriteAttributes::Y_FLIP)
    }

    /// Get the palette for this sprite (0 or 1)
    pub const fn palette(&self) -> u8 {
        if SpriteAttributes::from_bits_truncate(self.attributes).contains(SpriteAttributes::PALETTE)
        {
            1
        } else {
            0
        }
    }

    /// Get the adjusted Y position
    pub const fn y_position(&self) -> i32 {
        self.y_pos as i32 - 16
    }

    /// Get the adjusted X position
    pub const fn x_position(&self) -> i32 {
        self.x_pos as i32 - 8
    }
}

/// Pixel Processing Unit implementation
#[derive(Debug)]
pub struct Ppu {
    // Memory regions
    pub(crate) vram: [u8; VRAM_SIZE], // 8KB Video RAM
    pub(crate) oam: [u8; OAM_SIZE],   // Object Attribute Memory (160 bytes)
    pub(crate) vram_bank: u8,         // VRAM bank selection (CGB)
    pub(crate) bgp_index: u8,         // Background Palette Index (CGB)
    pub(crate) bgp_data: [u8; 64],    // Background Palette Data (CGB)
    pub(crate) obp_index: u8,         // Object Palette Index (CGB)
    pub(crate) obp_data: [u8; 64],    // Object Palette Data (CGB)

    // Registers
    pub(crate) lcdc: LcdControl, // LCD Control Register (0xFF40)
    pub(crate) stat: LcdStatus,  // LCD Status Register (0xFF41)
    pub(crate) scy: u8,          // Scroll Y (0xFF42)
    pub(crate) scx: u8,          // Scroll X (0xFF43)
    pub(crate) ly: u8,           // LCD Y-Coordinate (0xFF44)
    pub(crate) lyc: u8,          // LY Compare (0xFF45)
    pub(crate) bgp: u8,          // BG Palette Data (0xFF47)
    pub(crate) obp0: u8,         // Object Palette 0 Data (0xFF48)
    pub(crate) obp1: u8,         // Object Palette 1 Data (0xFF49)
    pub(crate) wy: u8,           // Window Y Position (0xFF4A)
    pub(crate) wx: u8,           // Window X Position minus 7 (0xFF4B)

    // Internal state
    pub(crate) mode: PpuMode,    // Current PPU mode
    pub(crate) mode_cycles: u32, // Cycles in the current mode
    pub(crate) window_line: u8,  // Current window line (internal counter)

    // Frame buffer
    pub frame_buffer: Box<[Color]>,
    pub frame_ready: bool, // Flag indicating a new frame is ready

    pub is_cgb: bool, // True if running in CGB mode
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            lcdc: LcdControl::from_bits_truncate(0x91), // Default: LCD on, BG enabled
            stat: LcdStatus::default(),
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC, // Default palette: 11 10 01 00 (black to white)
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            mode: PpuMode::OamSearch,
            mode_cycles: 0,
            window_line: 0,
            frame_buffer: vec![Color::WHITE; SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
            frame_ready: false,
            vram_bank: 0,
            bgp_index: 0,
            bgp_data: [0; 64],
            obp_index: 0,
            obp_data: [0; 64],
            is_cgb: false,
        }
    }
}

impl Ppu {
    /// Create a new PPU instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the LCD is enabled
    pub const fn is_lcd_enabled(&self) -> bool {
        self.lcdc.contains(LcdControl::LCD_ENABLE)
    }

    /// Set the current PPU mode and update the STAT register
    pub(crate) fn set_mode(&mut self, mode: PpuMode) {
        self.mode = mode;
        // Update mode bits in STAT register (bits 0-1)
        self.stat.remove(LcdStatus::MODE_FLAG_MASK);
        self.stat.insert(LcdStatus::from_bits_truncate(mode as u8));
    }

    /// Check if a STAT interrupt should be triggered
    pub(crate) const fn check_stat_interrupt(&self) -> bool {
        match self.mode {
            PpuMode::HBlank => self.stat.contains(LcdStatus::HBLANK_INTERRUPT),
            PpuMode::VBlank => self.stat.contains(LcdStatus::VBLANK_INTERRUPT),
            PpuMode::OamSearch => self.stat.contains(LcdStatus::OAM_INTERRUPT),
            PpuMode::PixelTransfer => false, // No interrupt during pixel transfer
        }
    }

    /// Check if an LYC=LY interrupt should be triggered
    pub(crate) const fn check_lyc_interrupt(&self) -> bool {
        self.stat.contains(LcdStatus::LYC_EQUAL_LY) && self.stat.contains(LcdStatus::LYC_INTERRUPT)
    }

    /// Update the LYC=LY flag in the STAT register
    pub(crate) fn update_lyc_flag(&mut self) {
        if self.ly == self.lyc {
            self.stat.insert(LcdStatus::LYC_EQUAL_LY);
        } else {
            self.stat.remove(LcdStatus::LYC_EQUAL_LY);
        }
    }

    /// Reset the PPU to its initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Reset window line counter
    fn reset_window_counter(&mut self) {
        self.window_line = 0;
    }

    /// Helper: Lookup CGB BG palette colour (stub, assumes palette 0)
    fn cgb_bg_color(&self, color_idx: u8, palette_num: u8) -> Color {
        // Each palette is 8 bytes (4 colours, 2 bytes each, little endian)
        let base = (palette_num as usize) * 8 + (color_idx as usize) * 2;
        let lo = self.bgp_data.get(base).copied().unwrap_or(0);
        let hi = self.bgp_data.get(base + 1).copied().unwrap_or(0);
        let rgb15 = ((hi as u16) << 8) | (lo as u16);
        // Convert 15-bit BGR to 24-bit RGB
        let r = ((rgb15 & 0x1F) << 3) as u8;
        let g = (((rgb15 >> 5) & 0x1F) << 3) as u8;
        let b = (((rgb15 >> 10) & 0x1F) << 3) as u8;
        Color { r, g, b, a: 0xFF }
    }

    /// Helper: Lookup CGB OBJ palette colour (stub, assumes palette 0)
    fn cgb_obj_color(&self, color_idx: u8, palette_num: u8) -> Color {
        // Each palette is 8 bytes (4 colours, 2 bytes each, little endian)
        let base = (palette_num as usize) * 8 + (color_idx as usize) * 2;
        let lo = self.obp_data.get(base).copied().unwrap_or(0);
        let hi = self.obp_data.get(base + 1).copied().unwrap_or(0);
        let rgb15 = ((hi as u16) << 8) | (lo as u16);
        let r = ((rgb15 & 0x1F) << 3) as u8;
        let g = (((rgb15 >> 5) & 0x1F) << 3) as u8;
        let b = (((rgb15 >> 10) & 0x1F) << 3) as u8;
        Color { r, g, b, a: 0xFF }
    }

    /// Render the window for the current scanline
    ///
    /// Note: The `_priority` variable is extracted for future CGB compatibility and to document the attribute layout, but is currently unused. Prefixing with underscore silences warnings. 🦀
    fn render_window(&mut self) {
        // Check if window is visible on this scanline
        if self.ly < self.wy || self.wx > 166 {
            return;
        }

        // Get window tile map address based on LCDC bit 6
        let window_tilemap_base = if self.lcdc.contains(LcdControl::WINDOW_TILEMAP) {
            0x1C00u16 // 0x9C00 relative to 0x8000
        } else {
            0x1800u16 // 0x9800 relative to 0x8000
        };

        // Get tile data address based on LCDC bit 4 (same as BG)
        let tiledata_base = if self.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA) {
            0x0000u16 // 0x8000 (unsigned tile indices)
        } else {
            0x1000u16 // 0x9000 (signed tile indices)
        };
        let signed = !self.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA);

        // Calculate Y position in the window
        let window_y = self.window_line;
        let tile_y = (window_y / 8) as u16;
        let tile_y_offset = window_y % 8;

        // Calculate offset in the frame buffer for this scanline
        let scanline_offset = self.ly as usize * SCREEN_WIDTH;

        // Calculate X positions
        let window_x = self.wx as isize - 7;

        // Render window pixels
        for x in 0..SCREEN_WIDTH {
            // Skip pixels to the left of the window
            if (x as isize) < window_x {
                continue;
            }

            // Calculate X position in the window
            let window_x_pos = (x as isize - window_x) as u16;
            let tile_x = window_x_pos / 8;
            let tile_x_offset = window_x_pos % 8;

            // Calculate tile map address
            let tile_map_addr = window_tilemap_base + tile_y * 32 + tile_x;
            let tile_id = self.vram[tile_map_addr as usize];

            // CGB: Read tile attributes for palette, bank, flipping, priority
            let (palette_num, vram_bank, x_flip, y_flip, _priority) = if self.is_cgb {
                let attr = self
                    .vram
                    .get(tile_map_addr as usize + 0x2000)
                    .copied()
                    .unwrap_or(0);
                unpack_tile_attributes(attr)
            } else {
                (0, 0, false, false, false)
            };

            // Calculate tile data address (depends on addressing mode)
            let tile_addr = tile_data_address(tiledata_base, tile_id, signed);

            // CGB: Use correct VRAM bank for tile data
            let vram_offset = if self.is_cgb && vram_bank == 1 {
                0x2000
            } else {
                0
            };

            let tile_line = if self.is_cgb && y_flip {
                7 - tile_y_offset
            } else {
                tile_y_offset
            };

            let tile_line_addr = tile_addr + (tile_line as u16) * 2;
            let tile_data_low = self.vram[vram_offset + tile_line_addr as usize];
            let tile_data_high = self.vram[vram_offset + tile_line_addr as usize + 1];

            let bit = if self.is_cgb && x_flip {
                tile_x_offset
            } else {
                7 - tile_x_offset
            } as u8;
            let color_idx = extract_colour_index(tile_data_low, tile_data_high, bit);

            let color = if self.is_cgb {
                self.cgb_bg_color(color_idx, palette_num)
            } else {
                Color::from_palette(color_idx, self.bgp)
            };
            self.frame_buffer[scanline_offset + x] = color;
        }

        // Increment internal window line counter
        self.window_line = self.window_line.wrapping_add(1);
    }

    /// Step the PPU by the given number of cycles
    /// Returns any interrupts that should be triggered
    pub fn step(&mut self, cycles: u32) -> Option<InterruptFlag> {
        if !self.is_lcd_enabled() {
            // LCD is disabled, don't do anything
            return None;
        }

        let mut interrupt = None;
        self.mode_cycles += cycles;

        // Process current PPU mode
        match self.mode {
            PpuMode::OamSearch => {
                if self.mode_cycles >= self.mode.duration() {
                    // Transition to pixel transfer mode
                    self.set_mode(PpuMode::PixelTransfer);
                    self.mode_cycles -= PpuMode::OamSearch.duration();
                }
            }
            PpuMode::PixelTransfer => {
                if self.mode_cycles >= self.mode.duration() {
                    // Render scanline before transitioning to HBlank
                    self.render_scanline();

                    // Transition to HBlank mode
                    self.set_mode(PpuMode::HBlank);
                    self.mode_cycles -= PpuMode::PixelTransfer.duration();

                    // Check if HBlank interrupt should be triggered
                    if self.check_stat_interrupt() {
                        interrupt = Some(InterruptFlag::LcdStat);
                    }
                }
            }
            PpuMode::HBlank => {
                if self.mode_cycles >= self.mode.duration() {
                    self.ly += 1;
                    self.update_lyc_flag();

                    // Check if LYC=LY interrupt should be triggered
                    if self.check_lyc_interrupt() {
                        interrupt = Some(InterruptFlag::LcdStat);
                    }

                    self.mode_cycles -= PpuMode::HBlank.duration();

                    if self.ly >= SCREEN_HEIGHT as u8 {
                        // End of frame, transition to VBlank
                        self.set_mode(PpuMode::VBlank);
                        self.frame_ready = true; // Signal that a new frame is ready
                        interrupt = Some(InterruptFlag::VBlank);

                        // Also check if VBlank STAT interrupt should be triggered
                        if self.check_stat_interrupt() {
                            interrupt = Some(InterruptFlag::LcdStat);
                        }
                    } else {
                        // Start next scanline with OAM search
                        self.set_mode(PpuMode::OamSearch);

                        // Check if OAM interrupt should be triggered
                        if self.check_stat_interrupt() {
                            interrupt = Some(InterruptFlag::LcdStat);
                        }
                    }
                }
            }
            PpuMode::VBlank => {
                if self.mode_cycles >= self.mode.duration() {
                    self.ly += 1;
                    self.update_lyc_flag();

                    // Check if LYC=LY interrupt should be triggered
                    if self.check_lyc_interrupt() {
                        interrupt = Some(InterruptFlag::LcdStat);
                    }

                    self.mode_cycles -= PpuMode::VBlank.duration();

                    if self.ly >= 154 {
                        // End of VBlank, start new frame
                        self.ly = 0;
                        self.reset_window_counter(); // Reset window line counter at start of frame
                        self.update_lyc_flag();
                        self.set_mode(PpuMode::OamSearch);

                        // Check if OAM interrupt should be triggered
                        if self.check_stat_interrupt() {
                            interrupt = Some(InterruptFlag::LcdStat);
                        }
                    }
                }
            }
        }

        interrupt
    }

    /// Render the current scanline to the frame buffer
    fn render_scanline(&mut self) {
        if !self.lcdc.contains(LcdControl::LCD_ENABLE) {
            // LCD is disabled, fill with white
            let start = self.ly as usize * SCREEN_WIDTH;
            let end = start + SCREEN_WIDTH;
            self.frame_buffer[start..end].fill(Color::WHITE);
            return;
        }

        // Render background and window first (if enabled)
        if self.lcdc.contains(LcdControl::BG_WINDOW_ENABLE) {
            self.render_background();

            // Render window if enabled
            if self.lcdc.contains(LcdControl::WINDOW_ENABLE) {
                self.render_window();
            }
        } else {
            // If background is disabled, fill with white
            let start = self.ly as usize * SCREEN_WIDTH;
            let end = start + SCREEN_WIDTH;
            self.frame_buffer[start..end].fill(Color::WHITE);
        }

        // Render sprites if enabled
        if self.lcdc.contains(LcdControl::SPRITE_ENABLE) {
            self.render_sprites();
        }
    }

    /// Render the background for the current scanline
    fn render_background(&mut self) {
        // Get tile map address based on LCDC bit 3
        let tilemap_base = if self.lcdc.contains(LcdControl::BG_TILEMAP) {
            0x1C00u16 // 0x9C00 relative to 0x8000
        } else {
            0x1800u16 // 0x9800 relative to 0x8000
        };

        // Get tile data address based on LCDC bit 4
        let tiledata_base = if self.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA) {
            0x0000u16 // 0x8000 (unsigned tile indices)
        } else {
            0x1000u16 // 0x9000 (signed tile indices)
        };
        let signed = !self.lcdc.contains(LcdControl::BG_WINDOW_TILE_DATA);

        // Calculate Y position in the background map
        let y_pos = (self.ly as u16 + self.scy as u16) & 0xFF;
        let tile_y = y_pos / 8;
        let tile_y_offset = y_pos % 8;

        // Calculate offset in the frame buffer for this scanline
        let scanline_offset = self.ly as usize * SCREEN_WIDTH;

        for x in 0..SCREEN_WIDTH {
            let x_pos = (x as u16 + self.scx as u16) & 0xFF;
            let tile_x = x_pos / 8;
            let tile_x_offset = x_pos % 8;

            let tile_map_addr = tilemap_base + tile_y * 32 + tile_x;
            let tile_id = self.vram[tile_map_addr as usize];

            // CGB: Read tile attributes for palette, bank, flipping, priority
            let (palette_num, vram_bank, x_flip, y_flip, _priority) = if self.is_cgb {
                // Attribute map is at 0x2000 offset in VRAM
                let attr = self
                    .vram
                    .get(tile_map_addr as usize + 0x2000)
                    .copied()
                    .unwrap_or(0);
                unpack_tile_attributes(attr)
            } else {
                (0, 0, false, false, false)
            };

            // Calculate tile data address (depends on addressing mode)
            let tile_addr = tile_data_address(tiledata_base, tile_id, signed);

            // CGB: Use correct VRAM bank for tile data
            let vram_offset = if self.is_cgb && vram_bank == 1 {
                0x2000
            } else {
                0
            };

            let tile_line = if self.is_cgb && y_flip {
                7 - tile_y_offset
            } else {
                tile_y_offset
            };

            let tile_line_addr = tile_addr + (tile_line as u16) * 2;
            let tile_data_low = self.vram[vram_offset + tile_line_addr as usize];
            let tile_data_high = self.vram[vram_offset + tile_line_addr as usize + 1];

            let bit = if self.is_cgb && x_flip {
                tile_x_offset
            } else {
                7 - tile_x_offset
            } as u8;
            let color_idx = extract_colour_index(tile_data_low, tile_data_high, bit);

            let color = if self.is_cgb {
                self.cgb_bg_color(color_idx, palette_num)
            } else {
                Color::from_palette(color_idx, self.bgp)
            };
            self.frame_buffer[scanline_offset + x] = color;
        }
    }

    /// Render sprites for the current scanline
    fn render_sprites(&mut self) {
        if !self.lcdc.contains(LcdControl::SPRITE_ENABLE) {
            return;
        }

        let sprite_height = if self.lcdc.contains(LcdControl::SPRITE_SIZE) {
            16
        } else {
            8
        };

        // Collect visible sprites for this scanline
        let mut visible_sprites = Vec::with_capacity(MAX_SPRITES_PER_LINE);
        for i in (0..OAM_SIZE).step_by(4) {
            let sprite = Sprite::from_oam(&self.oam, i / 4);
            let y_pos = sprite.y_position();

            // Check if sprite is visible on this scanline
            if y_pos <= self.ly as i32 && (y_pos + sprite_height as i32) > self.ly as i32 {
                visible_sprites.push(sprite);
                if visible_sprites.len() >= MAX_SPRITES_PER_LINE {
                    break;
                }
            }
        }

        // Sort sprites by priority (lower X coordinate has priority, if equal, lower OAM index has priority)
        visible_sprites.sort_by_key(|sprite| sprite.x_pos);

        // Render sprites in priority order
        for sprite in visible_sprites.iter().rev() {
            let y_pos = sprite.y_position();
            let x_pos = sprite.x_position();

            // CGB: Read OAM attribute for palette, bank, flipping, priority
            let (palette_num, vram_bank, x_flip, y_flip, _priority) = if self.is_cgb {
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

            // Calculate which line of the sprite we're drawing
            let mut line = (self.ly as i32 - y_pos) as u8;
            if y_flip {
                line = (sprite_height - 1) - line;
            }

            // Get the tile data
            let tile_addr = if sprite_height == 16 {
                // In 8x16 mode, bit 0 of tile index is ignored
                (u16::from(sprite.tile_idx & 0xFE) * 16) + u16::from(line & 0xF) * 2
            } else {
                u16::from(sprite.tile_idx) * 16 + u16::from(line) * 2
            };

            // CGB: Use correct VRAM bank for tile data
            let vram_offset = if self.is_cgb && vram_bank == 1 {
                0x2000
            } else {
                0
            };

            let low_byte = self.vram[vram_offset + tile_addr as usize];
            let high_byte = self.vram[vram_offset + (tile_addr + 1) as usize];

            // Draw each pixel of the sprite line
            for x in 0..8 {
                let screen_x = x_pos + x;
                if screen_x < 0 || screen_x >= SCREEN_WIDTH as i32 {
                    continue;
                }

                let bit = if x_flip { x } else { 7 - x } as u8;
                let color_idx = extract_colour_index(low_byte, high_byte, bit);

                // Skip transparent pixels (color 0)
                if color_idx == 0 {
                    continue;
                }

                let color = if self.is_cgb {
                    self.cgb_obj_color(color_idx, palette_num)
                } else {
                    let palette = if sprite.palette() == 0 {
                        self.obp0
                    } else {
                        self.obp1
                    };
                    Color::from_palette(color_idx, palette)
                };
                let pixel_index = self.ly as usize * SCREEN_WIDTH + screen_x as usize;

                // Check sprite priority
                if (self.is_cgb && _priority) || (!self.is_cgb && !sprite.has_priority()) {
                    // Only draw if background is white (color 0)
                    let bg_color = self.frame_buffer[pixel_index];
                    if bg_color == Color::WHITE {
                        self.frame_buffer[pixel_index] = color;
                    }
                } else {
                    // Always draw sprite
                    self.frame_buffer[pixel_index] = color;
                }
            }
        }
    }

    /// Read from a PPU register or memory
    pub fn read(&self, addr: u16) -> Result<u8, PpuError> {
        match addr {
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                // VRAM is inaccessible during pixel transfer
                if self.mode == PpuMode::PixelTransfer {
                    return Err(PpuError::VramLocked);
                }
                Ok(self.vram[(addr - 0x8000) as usize])
            }

            // OAM (0xFE00-0xFE9F)
            0xFE00..=0xFE9F => {
                // OAM is inaccessible during pixel transfer and OAM search
                if self.mode == PpuMode::PixelTransfer || self.mode == PpuMode::OamSearch {
                    return Err(PpuError::OamLocked);
                }
                Ok(self.oam[(addr - 0xFE00) as usize])
            }

            // PPU Registers
            LCDC_ADDR => Ok(self.lcdc.bits()),
            STAT_ADDR => {
                // Bits 0-1 are mode bits, 2 is LYC=LY flag, 3-6 are interrupt enable flags
                let stat_value = (self.stat.bits() & 0xF8)
                    | (self.mode as u8)
                    | if self.ly == self.lyc { 4 } else { 0 };
                Ok(stat_value)
            }
            SCY_ADDR => Ok(self.scy),
            SCX_ADDR => Ok(self.scx),
            LY_ADDR => Ok(self.ly),
            LYC_ADDR => Ok(self.lyc),
            BGP_ADDR => Ok(self.bgp),
            OBP0_ADDR => Ok(self.obp0),
            OBP1_ADDR => Ok(self.obp1),
            WY_ADDR => Ok(self.wy),
            WX_ADDR => Ok(self.wx),

            // CGB Registers
            0xFF4F => Ok(self.vram_bank),
            0xFF68 => Ok(self.bgp_index),
            0xFF69 => Ok(self.bgp_data[(self.bgp_index & 0x3F) as usize]),
            0xFF6A => Ok(self.obp_index),
            0xFF6B => Ok(self.obp_data[(self.obp_index & 0x3F) as usize]),

            // Invalid address
            _ => Err(PpuError::InvalidAccess(addr)),
        }
    }

    /// Write to a PPU register or memory
    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), PpuError> {
        match addr {
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                // VRAM is inaccessible during pixel transfer
                if self.mode == PpuMode::PixelTransfer {
                    return Err(PpuError::VramLocked);
                }
                self.vram[(addr - 0x8000) as usize] = value;
                Ok(())
            }

            // OAM (0xFE00-0xFE9F)
            0xFE00..=0xFE9F => {
                // OAM is inaccessible during pixel transfer and OAM search
                if self.mode == PpuMode::PixelTransfer || self.mode == PpuMode::OamSearch {
                    return Err(PpuError::OamLocked);
                }
                self.oam[(addr - 0xFE00) as usize] = value;
                Ok(())
            }

            // PPU Registers
            LCDC_ADDR => {
                let old_lcd_enabled = self.lcdc.contains(LcdControl::LCD_ENABLE);
                self.lcdc = LcdControl::from_bits_truncate(value);

                // Handle LCD enable/disable
                let new_lcd_enabled = self.lcdc.contains(LcdControl::LCD_ENABLE);
                if old_lcd_enabled && !new_lcd_enabled {
                    // LCD was disabled - reset PPU state
                    self.ly = 0;
                    self.mode_cycles = 0;
                    self.set_mode(PpuMode::HBlank);
                } else if !old_lcd_enabled && new_lcd_enabled {
                    // LCD was enabled - start in mode 0
                    self.set_mode(PpuMode::OamSearch);
                }
                Ok(())
            }
            STAT_ADDR => {
                // Only bits 3-6 are writable (interrupt enable flags)
                // Bits 0-2 are read-only
                let current_mode_bits = self.stat.bits() & 0x07;
                let new_stat_bits = (value & 0xF8) | current_mode_bits;
                self.stat = LcdStatus::from_bits_truncate(new_stat_bits);
                Ok(())
            }
            SCY_ADDR => {
                self.scy = value;
                Ok(())
            }
            SCX_ADDR => {
                self.scx = value;
                Ok(())
            }
            LY_ADDR => {
                // LY is read-only, writes are ignored
                Ok(())
            }
            LYC_ADDR => {
                self.lyc = value;
                self.update_lyc_flag();
                Ok(())
            }
            BGP_ADDR => {
                self.bgp = value;
                Ok(())
            }
            OBP0_ADDR => {
                self.obp0 = value;
                Ok(())
            }
            OBP1_ADDR => {
                self.obp1 = value;
                Ok(())
            }
            WY_ADDR => {
                self.wy = value;
                Ok(())
            }
            WX_ADDR => {
                self.wx = value;
                Ok(())
            }

            // CGB Registers
            0xFF4F => {
                self.vram_bank = value & 0x01;
                Ok(())
            }
            0xFF68 => {
                self.bgp_index = value;
                Ok(())
            }
            0xFF69 => {
                let auto_increment = (self.bgp_index & 0x80) != 0;
                let index = self.bgp_index & 0x3F;
                self.bgp_data[index as usize] = value;
                if auto_increment {
                    self.bgp_index = ((self.bgp_index + 1) & 0x3F) | (self.bgp_index & 0x80);
                }
                Ok(())
            }
            0xFF6A => {
                self.obp_index = value;
                Ok(())
            }
            0xFF6B => {
                let auto_increment = (self.obp_index & 0x80) != 0;
                let index = self.obp_index & 0x3F;
                self.obp_data[index as usize] = value;
                if auto_increment {
                    self.obp_index = ((self.obp_index + 1) & 0x3F) | (self.obp_index & 0x80);
                }
                Ok(())
            }

            // Invalid address
            _ => Err(PpuError::InvalidAccess(addr)),
        }
    }

    /// Get the current frame buffer
    pub fn get_frame_buffer(&self) -> &[Color] {
        self.frame_buffer.as_ref()
    }

    /// Clear the frame ready flag
    pub fn clear_frame_ready(&mut self) {
        self.frame_ready = false;
    }

    /// Check if a new frame is ready
    pub const fn is_frame_ready(&self) -> bool {
        self.frame_ready
    }

    /// Get the current PPU mode
    pub const fn get_mode(&self) -> PpuMode {
        self.mode
    }

    /// Write to OAM via DMA
    pub fn dma_write(&mut self, data: &[u8; OAM_SIZE]) {
        // During DMA, data is copied from the source to OAM regardless of PPU mode
        self.oam.copy_from_slice(data);
    }
}
