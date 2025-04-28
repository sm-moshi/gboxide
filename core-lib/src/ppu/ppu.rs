use crate::interrupts::InterruptFlag;

use super::color::Color;
use super::lcdc::LcdControl;
use super::ppu_modes;
use super::render;
use super::sprite::OamScanResult;
use super::stat::LcdStatus;
use super::VRAM_BANK_SIZE;
use super::{PpuError, PpuMode, SCREEN_HEIGHT, SCREEN_WIDTH};
use super::{
    BGP_ADDR, LCDC_ADDR, LYC_ADDR, LY_ADDR, OAM_SIZE, OBP0_ADDR, OBP1_ADDR, SCX_ADDR, SCY_ADDR,
    STAT_ADDR, VRAM_SIZE, WX_ADDR, WY_ADDR,
};

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

    // OAM scan results (sprites visible on current scanline)
    pub(crate) oam_scan_result: OamScanResult,

    // Frame buffer
    pub frame_buffer: Box<[Color]>,
    pub frame_ready: bool, // Flag indicating a new frame is ready

    pub is_cgb: bool, // True if running in CGB mode
    // Per-pixel BG priority buffer for CGB (bitfield, 1 bit per pixel)
    bg_priority_buffer: Box<[u8]>,
    /// Sprite collision info: (scanline, x, Vec<OAM indices>)
    sprite_collisions: Vec<(u8, u8, Vec<usize>)>,
    /// Enable/disable collision tracking (for debug/perf)
    pub collision_tracking: bool,
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
            oam_scan_result: OamScanResult::new(),
            frame_buffer: vec![Color::WHITE; SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
            frame_ready: false,
            vram_bank: 0,
            bgp_index: 0,
            bgp_data: [0; 64],
            obp_index: 0,
            obp_data: [0; 64],
            is_cgb: false,
            bg_priority_buffer: vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT).div_ceil(8)]
                .into_boxed_slice(),
            sprite_collisions: Vec::new(),
            collision_tracking: false,
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
    const fn reset_window_counter(&mut self) {
        self.window_line = 0;
    }

    /// Step the PPU by the given number of cycles
    /// Returns any interrupts that should be triggered
    #[allow(clippy::branches_sharing_code)]
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
                let (new_mode, remaining_cycles) = ppu_modes::handle_mode_2(
                    self.mode_cycles,
                    &self.oam,
                    self.ly,
                    self.lcdc,
                    &mut self.oam_scan_result,
                );

                if new_mode != PpuMode::OamSearch {
                    self.set_mode(new_mode);
                    self.mode_cycles = remaining_cycles;
                }
            }
            PpuMode::PixelTransfer => {
                let (new_mode, remaining_cycles) = ppu_modes::handle_mode_3(self.mode_cycles);

                if new_mode != PpuMode::PixelTransfer {
                    // Render scanline before transitioning to HBlank
                    render::Renderer.render_scanline(self);

                    self.set_mode(new_mode);
                    self.mode_cycles = remaining_cycles;

                    // Check if HBlank interrupt should be triggered
                    if self.check_stat_interrupt() {
                        interrupt = Some(InterruptFlag::LcdStat);
                    }
                }
            }
            PpuMode::HBlank => {
                let (new_mode, remaining_cycles) = ppu_modes::handle_mode_0(self.mode_cycles);

                if new_mode != self.mode {
                    self.ly += 1;
                    self.update_lyc_flag();
                    self.mode_cycles = remaining_cycles;

                    if self.ly == 144 {
                        self.set_mode(PpuMode::VBlank);
                        interrupt = Some(InterruptFlag::VBlank);
                    } else {
                        self.set_mode(PpuMode::OamSearch);
                        if self.check_stat_interrupt() {
                            interrupt = Some(InterruptFlag::LcdStat);
                        }
                    }
                    if self.check_lyc_interrupt() {
                        interrupt = Some(InterruptFlag::LcdStat);
                    }
                }
            }
            PpuMode::VBlank => {
                let (new_mode, remaining_cycles, reset_ly) =
                    ppu_modes::handle_mode_1(self.mode_cycles, self.ly);

                if new_mode != self.mode || reset_ly {
                    if reset_ly {
                        self.ly = 0;
                        self.reset_window_counter(); // Reset window line counter at start of frame
                    } else {
                        self.ly += 1;
                    }

                    self.update_lyc_flag();
                    self.mode_cycles = remaining_cycles;

                    if new_mode != self.mode {
                        self.set_mode(new_mode);

                        // Check if OAM interrupt should be triggered when returning to OAM scan
                        if new_mode == PpuMode::OamSearch && self.check_stat_interrupt() {
                            interrupt = Some(InterruptFlag::LcdStat);
                        }
                    }

                    // Check if LYC=LY interrupt should be triggered
                    if self.check_lyc_interrupt() {
                        interrupt = Some(InterruptFlag::LcdStat);
                    }
                }
            }
        }

        interrupt
    }

    /// Read from a PPU register or memory
    ///
    /// # Errors
    /// Returns an error if the address is invalid or the memory is locked (e.g., during pixel transfer).
    pub fn read(&self, addr: u16) -> Result<u8, PpuError> {
        match addr {
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                // VRAM is inaccessible during pixel transfer
                if self.mode == PpuMode::PixelTransfer {
                    return Err(PpuError::VramLocked);
                }
                let bank = if self.is_cgb { self.vram_bank & 0x1 } else { 0 };
                let offset = (bank as usize) * VRAM_BANK_SIZE + (addr as usize - 0x8000);
                Ok(self.vram[offset])
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
    ///
    /// # Errors
    /// Returns an error if the address is invalid or the memory is locked (e.g., during pixel transfer).
    #[allow(clippy::too_many_lines)]
    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), PpuError> {
        match addr {
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                // VRAM is inaccessible during pixel transfer
                if self.mode == PpuMode::PixelTransfer {
                    return Err(PpuError::VramLocked);
                }
                let bank = if self.is_cgb { self.vram_bank & 0x1 } else { 0 };
                let offset = (bank as usize) * VRAM_BANK_SIZE + (addr as usize - 0x8000);
                self.vram[offset] = value;
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
    pub const fn clear_frame_ready(&mut self) {
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
    pub const fn dma_write(&mut self, data: &[u8; OAM_SIZE]) {
        // During DMA, data is copied from the source to OAM regardless of PPU mode
        self.oam.copy_from_slice(data);
    }

    pub(crate) fn bg_priority_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bg_priority_buffer
    }

    /// Record a sprite collision at (ly, x) with the given OAM indices
    pub(crate) fn record_sprite_collision(&mut self, ly: u8, x: u8, indices: Vec<usize>) {
        if self.collision_tracking {
            self.sprite_collisions.push((ly, x, indices));
        }
    }

    /// Get and clear all sprite collisions (for tests/debug)
    pub fn take_sprite_collisions(&mut self) -> Vec<(u8, u8, Vec<usize>)> {
        std::mem::take(&mut self.sprite_collisions)
    }

    /// Enable or disable collision tracking
    pub fn set_collision_tracking(&mut self, enabled: bool) {
        self.collision_tracking = enabled;
    }
}
