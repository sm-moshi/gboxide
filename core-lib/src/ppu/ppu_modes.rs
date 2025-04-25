use crate::ppu::lcdc::LcdControl;
use crate::ppu::sprite::{collect_visible_sprites, OamScanResult};
use crate::ppu::PpuMode;

/// Handles PPU Mode 2 (OAM scan) logic
pub fn handle_mode_2(
    cycles: u32,
    oam: &[u8],
    ly: u8,
    lcdc: LcdControl,
    scan_result: &mut OamScanResult,
) -> (PpuMode, u32) {
    // Mode 2 takes 80 T-cycles
    if cycles >= 80 {
        // Reset scan result at the beginning of Mode 2
        scan_result.reset();

        // Only perform the OAM scan if sprites are enabled
        if lcdc.contains(LcdControl::SPRITE_ENABLE) {
            // Calculate sprite height based on LCDC flag (8x8 or 8x16)
            let sprite_height = if lcdc.contains(LcdControl::SPRITE_SIZE) {
                16
            } else {
                8
            };

            // Collect visible sprites for the current scanline
            *scan_result = collect_visible_sprites(oam, ly, sprite_height);
        }

        // Move to Mode 3 (Pixel Transfer) and return remaining cycles
        (PpuMode::PixelTransfer, cycles - 80)
    } else {
        // Stay in Mode 2 and continue counting cycles
        (PpuMode::OamSearch, cycles)
    }
}

/// Handles PPU Mode 3 (Pixel Transfer) logic
pub const fn handle_mode_3(cycles: u32) -> (PpuMode, u32) {
    // Mode 3 takes a variable number of cycles (typically 172-289)
    // For now, we'll use a fixed value of 172
    if cycles >= 172 {
        // Move to Mode 0 (HBlank) and return remaining cycles
        (PpuMode::HBlank, cycles - 172)
    } else {
        // Stay in Mode 3 and continue counting cycles
        (PpuMode::PixelTransfer, cycles)
    }
}

/// Handles PPU Mode 0 (`HBlank`) logic
pub const fn handle_mode_0(cycles: u32) -> (PpuMode, u32) {
    // Mode 0 takes a variable number of cycles (typically 204-456)
    // For simplicity, we'll use a fixed value to make the total line time 456 cycles
    // 456 - 80 (Mode 2) - 172 (Mode 3) = 204
    if cycles >= 204 {
        // Move to Mode 2 (OAM scan) of the next line and return remaining cycles
        (PpuMode::OamSearch, cycles - 204)
    } else {
        // Stay in Mode 0 and continue counting cycles
        (PpuMode::HBlank, cycles)
    }
}

/// Handles PPU Mode 1 (`VBlank`) logic
pub const fn handle_mode_1(cycles: u32, ly: u8) -> (PpuMode, u32, bool) {
    // Each VBlank line takes 456 cycles
    if cycles >= 456 {
        let new_ly = ly + 1;
        let completed_vblank = new_ly > 153;

        // If VBlank is complete, go back to Mode 2 (OAM scan) and reset LY to 0
        if completed_vblank {
            (PpuMode::OamSearch, cycles - 456, true)
        } else {
            // Continue in VBlank mode for the next line
            (PpuMode::VBlank, cycles - 456, false)
        }
    } else {
        // Stay in Mode 1 and continue counting cycles
        (PpuMode::VBlank, cycles, false)
    }
}
