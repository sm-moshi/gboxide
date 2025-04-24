//! Channel 4: Noise channel (modularised)
//!
//! Implements NR41–NR44, envelope, and polynomial counter logic.
//!
//! See: <https://gbdev.io/pandocs/Audio.html#ff20--nr41-channel-4-sound-length-rw>

use crate::apu::Envelope;

/// NR41 - Channel 4 Sound Length (0xFF20)
/// Bits 5-0: Sound length data (0-63)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr41(pub u8);

impl Nr41 {
    pub const fn length(self) -> u8 {
        self.0 & 0x3F
    }
    pub const fn read_reg(self) -> u8 {
        self.0 & 0x3F
    }
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value & 0x3F;
    }
}

/// NR42 - Channel 4 Envelope (0xFF21)
/// Bits 7-4: Initial volume, 3: Envelope direction, 2-0: Period
/// (Reuse Envelope struct)
/// NR43 - Channel 4 Polynomial Counter (0xFF22)
/// Bits 7-4: Shift clock frequency (s)
/// Bit 3: Counter step/width (0=15 bits, 1=7 bits)
/// Bits 2-0: Dividing ratio of frequencies (r)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr43(pub u8);

impl Nr43 {
    pub const fn shift_clock_freq(self) -> u8 {
        (self.0 >> 4) & 0x0F
    }
    pub const fn counter_step_width(self) -> bool {
        (self.0 & 0x08) != 0
    }
    pub const fn dividing_ratio(self) -> u8 {
        self.0 & 0x07
    }
    pub const fn read_reg(self) -> u8 {
        self.0
    }
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR44 - Channel 4 Counter/consecutive; Initial (0xFF23)
/// Bit 7: Initial (trigger, write only)
/// Bit 6: Counter/consecutive selection (length enable)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr44(pub u8);

impl Nr44 {
    pub const fn trigger(self) -> bool {
        (self.0 & 0x80) != 0
    }
    pub const fn length_enable(self) -> bool {
        (self.0 & 0x40) != 0
    }
    pub const fn read_reg(self) -> u8 {
        self.0 & 0xC0
    }
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value & 0xC0;
    }
}

/// Channel 4: Noise channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel4 {
    /// NR41 - Sound length
    pub nr41: Nr41,
    /// NR42 - Envelope
    pub nr42: Envelope,
    /// NR43 - Polynomial counter
    pub nr43: Nr43,
    /// NR44 - Counter/consecutive; initial
    pub nr44: Nr44,
    /// 15-bit LFSR state (noise generator)
    pub lfsr: u16,
    /// Timer for LFSR clocking
    pub sample_timer: u16,
    /// Current output sample (for mixing)
    pub output: u8,
    /// DAC enable flag
    pub dac_enabled: bool,
    /// Length counter (decrements if enabled)
    pub length_counter: u8,
    /// Envelope timer (counts down to envelope step)
    pub envelope_timer: u8,
    /// Envelope volume (current output volume)
    pub envelope_volume: u8,
}

impl Default for Channel4 {
    fn default() -> Self {
        Self {
            nr41: Nr41(0),
            nr42: Envelope::new(),
            nr43: Nr43(0),
            nr44: Nr44(0),
            lfsr: 0x7FFF, // Hardware reset value
            sample_timer: 0,
            output: 0,
            dac_enabled: false,
            length_counter: 0,
            envelope_timer: 0,
            envelope_volume: 0,
        }
    }
}

impl Channel4 {
    /// Read a register by offset (0=NR41, 1=NR42, 2=NR43, 3=NR44)
    pub const fn read_reg(&self, offset: u8) -> u8 {
        match offset {
            0 => self.nr41.read_reg(),
            1 => self.nr42.read_reg(),
            2 => self.nr43.read_reg(),
            3 => self.nr44.read_reg(),
            _ => 0xFF,
        }
    }
    /// Write a register by offset (0=NR41, 1=NR42, 2=NR43, 3=NR44)
    pub fn write_reg(&mut self, offset: u8, value: u8) {
        match offset {
            0 => self.nr41.write_reg(value),
            1 => self.nr42.write_reg(value),
            2 => self.nr43.write_reg(value),
            3 => self.nr44.write_reg(value),
            _ => {}
        }
    }
    /// Trigger the channel (reset LFSR and state)
    pub fn trigger(&mut self) {
        self.lfsr = 0x7FFF; // Hardware: all bits set
        self.sample_timer = 0;
        self.output = 0;
        self.dac_enabled = true;
        self.clock_lfsr(); // Hardware: LFSR is clocked once on trigger
    }
    /// Clock the LFSR (Linear Feedback Shift Register) for noise generation
    /// Updates the LFSR and output fields according to NR43 (15-bit or 7-bit mode)
    pub fn clock_lfsr(&mut self) {
        // XOR bit 0 and bit 1
        let xor = (self.lfsr & 0x01) ^ ((self.lfsr & 0x02) >> 1);
        // Shift right by 1
        self.lfsr >>= 1;
        // Insert XOR result at bit 14 (15-bit mode)
        self.lfsr |= xor << 14;
        // If NR43 bit 3 is set, also insert at bit 6 (7-bit mode)
        if self.nr43.counter_step_width() {
            // Set bit 6 to XOR result
            self.lfsr = (self.lfsr & !(1 << 6)) | (xor << 6);
        }
        // Output is inverted bit 0
        self.output = (!self.lfsr & 0x01) as u8;
    }
}
