/// Unpack CGB tile/sprite attribute byte into (`palette_num`, `vram_bank`, `x_flip`, `y_flip`, priority)
///
/// - `palette_num`: Palette number (0-7)
/// - `vram_bank`: VRAM bank (0 or 1)
/// - `x_flip`: Horizontal flip
/// - `y_flip`: Vertical flip
/// - `priority`: BG-to-OAM priority (true = BG priority, false = OBJ priority)
pub const fn unpack_tile_attributes(attr: u8) -> (u8, u8, bool, bool, bool) {
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
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub const fn tile_data_address(base: u16, tile_id: u8, signed: bool) -> u16 {
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
pub const fn extract_colour_index(low: u8, high: u8, bit: u8) -> u8 {
    (((high >> bit) & 1) << 1) | ((low >> bit) & 1)
}

/// Computes a banked index into a linear buffer.
///
/// - `addr`: the memory address
/// - `bank`: current bank number
/// - `base`: base address where bank switching starts
/// - `bank_size`: size of each bank in bytes
pub const fn banked_index(addr: u8, bank: usize, base: u8, bank_size: usize) -> usize {
    bank * bank_size + (addr as usize - base as usize)
}

/// Computes a banked index into a linear buffer for 16-bit addresses.
///
/// - `addr`: the memory address (u16)
/// - `base`: base address where bank switching starts
/// - `bank`: current bank number
/// - `bank_size`: size of each bank in bytes
pub const fn banked_index_u16(addr: u16, base: u16, bank: usize, bank_size: usize) -> usize {
    bank * bank_size + (addr as usize - base as usize)
}

/// Allocates an optional byte buffer of length `len`, filled with `fill`.
///
/// Returns `None` if `len` is zero.
pub fn optional_buffer(len: usize, fill: u8) -> Option<Vec<u8>> {
    if len == 0 {
        None
    } else {
        Some(vec![fill; len])
    }
}

/// Gets a copy of an element from a slice or returns an error if out of bounds.
pub fn get_or_error<T: Copy, E>(slice: &[T], idx: usize, err: E) -> Result<T, E> {
    slice.get(idx).copied().ok_or(err)
}

/// Extracts a null-terminated string from a byte slice over the given range.
pub fn extract_c_string(data: &[u8], range: std::ops::Range<usize>) -> String {
    let slice = &data[range];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    String::from_utf8_lossy(&slice[..end]).into_owned()
}

/// Extracts `mask` bits from `v` starting at `shift`.
/// Equivalent to `(v >> shift) & mask`.
pub const fn get_bits(v: u8, mask: u8, shift: u8) -> u8 {
    (v >> shift) & mask
}

/// Sets `mask` bits at `shift` in `v` to `bits`.
/// Equivalent to `(v & !(mask << shift)) | ((bits & mask) << shift)`.
pub const fn set_bits(v: u8, mask: u8, shift: u8, bits: u8) -> u8 {
    (v & !(mask << shift)) | ((bits & mask) << shift)
}

/// Reads a single bit from `v` at position `bit`.
pub const fn get_bit(v: u8, bit: u8) -> bool {
    ((v >> bit) & 1) != 0
}

/// Sets or clears a single `bit` in `v`.
pub const fn set_bit(v: u8, bit: u8, on: bool) -> u8 {
    if on {
        v | (1 << bit)
    } else {
        v & !(1 << bit)
    }
}
