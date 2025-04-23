/// Unpack CGB tile/sprite attribute byte into (`palette_num`, `vram_bank`, `x_flip`, `y_flip`, priority)
///
/// - `palette_num`: Palette number (0-7)
/// - `vram_bank`: VRAM bank (0 or 1)
/// - `x_flip`: Horizontal flip
/// - `y_flip`: Vertical flip
/// - `priority`: BG-to-OAM priority (true = BG priority, false = OBJ priority)
#[inline]
pub(crate) const fn unpack_tile_attributes(attr: u8) -> (u8, u8, bool, bool, bool) {
    let palette_num = attr & 0x07;
    let vram_bank = (attr >> 3) & 0x01;
    let x_flip = (attr & 0x20) != 0;
    let y_flip = (attr & 0x40) != 0;
    let priority = (attr & 0x80) != 0;
    (palette_num, vram_bank, x_flip, y_flip, priority)
}

/// Compute tile data address given base, `tile_id`, and addressing mode (signed/unsigned)
///
/// - `base`: Base address (0x0000 or 0x1000)
/// - `tile_id`: Tile index
/// - `signed`: If true, use signed addressing (0x9000 region)
#[inline]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub(crate) const fn tile_data_address(base: u16, tile_id: u8, signed: bool) -> u16 {
    if signed {
        // SAFETY: This matches Game Boy hardware behaviour for signed tile addressing.
        base + ((tile_id as i8 as i16 + 128) as u16) * 16
    } else {
        base + (tile_id as u16) * 16
    }
}

/// Extract 2-bit colour index from two bytes and a bit index
///
/// - `low`: Low byte of tile/sprite data
/// - `high`: High byte of tile/sprite data
/// - `bit`: Bit index (0-7)
#[inline]
pub(crate) const fn extract_colour_index(low: u8, high: u8, bit: u8) -> u8 {
    (((high >> bit) & 1) << 1) | ((low >> bit) & 1)
}
