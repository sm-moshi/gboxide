//! Envelope register for APU channels (modularised, Mooneye style)

/// Envelope register (used for NR12, NR22, NR42)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Envelope {
    pub volume: u8,       // bits 7-4
    pub increasing: bool, // bit 3
    pub period: u8,       // bits 2-0
}

#[allow(dead_code)]
impl Envelope {
    /// Create a new Envelope with default values
    pub const fn new() -> Self {
        Self {
            volume: 0,
            increasing: false,
            period: 0,
        }
    }
    /// Read the envelope register as a raw u8 (`NRx2` format)
    pub const fn read_reg(self) -> u8 {
        (self.volume << 4) | ((self.increasing as u8) << 3) | (self.period & 0x7)
    }
    /// Write a raw u8 value to the envelope register (`NRx2` format)
    pub fn write_reg(&mut self, value: u8) {
        self.volume = (value >> 4) & 0x0F;
        self.increasing = (value & 0x08) != 0;
        self.period = value & 0x07;
    }
}
