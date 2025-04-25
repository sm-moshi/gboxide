//! Channel 3: Wave channel (modularised)
//!
//! Implements NR30–NR34, wave RAM access, and output level logic.
//!
//! See: <https://gbdev.io/pandocs/Audio.html#ff1a--nr30-channel-3-sound-onoff-rw>

use crate::helpers::{get_bit, get_bits};

/// NR30 - Channel 3 Sound on/off (0xFF1A)
/// Bit 7 - Sound Channel 3 Off  (0=Stop, 1=Playback)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr30(pub u8);

impl Nr30 {
    pub const fn sound_on(self) -> bool {
        get_bit(self.0, 7)
    }
    pub const fn read_reg(self) -> u8 {
        self.0
    }
    /// Write a value to NR30 register
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value & 0x80; // Only bit 7 is used
    }
}

/// NR31 - Channel 3 Sound Length (0xFF1B)
/// 8 bits - Sound length (0-255)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr31(pub u8);

impl Nr31 {
    pub const fn length(self) -> u8 {
        self.0
    }
    pub const fn read_reg(self) -> u8 {
        self.0
    }
    /// Write a value to NR31 register
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR32 - Channel 3 Select output level (0xFF1C)
/// Bits 6-5 - Output level
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr32(pub u8);

impl Nr32 {
    pub const fn output_level(self) -> u8 {
        get_bits(self.0, 0x03, 5)
    }
    pub const fn read_reg(self) -> u8 {
        self.0 & 0x60 // Only bits 6-5 are used
    }
    /// Write a value to NR32 register
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value & 0x60;
    }
}

/// NR33 - Channel 3 Frequency low (0xFF1D)
/// 8 bits - Frequency low
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr33(pub u8);

impl Nr33 {
    pub const fn freq_lo(self) -> u8 {
        self.0
    }
    pub const fn read_reg(self) -> u8 {
        self.0
    }
    /// Write a value to NR33 register
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR34 - Channel 3 Frequency high/control (0xFF1E)
/// Bit 7 - Initial (1=Restart Sound) (Write Only)
/// Bit 6 - Counter/consecutive selection (1=Stop output when length in NR31 expires)
/// Bits 2-0 - Frequency high bits
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr34(pub u8);

impl Nr34 {
    pub const fn trigger(self) -> bool {
        get_bit(self.0, 7)
    }
    pub const fn length_enable(self) -> bool {
        get_bit(self.0, 6)
    }
    pub const fn freq_high(self) -> u8 {
        get_bits(self.0, 0x07, 0)
    }
    pub const fn read_reg(self) -> u8 {
        self.0 & 0xC7 // Only bits 7,6,2-0 are used
    }
    /// Write a value to NR34 register
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value & 0xC7;
    }
}

/// Channel 3: Wave channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Channel3 {
    /// NR30 - Sound on/off
    pub nr30: Nr30,
    /// NR31 - Sound length
    pub nr31: Nr31,
    /// NR32 - Output level
    pub nr32: Nr32,
    /// NR33 - Frequency low
    pub nr33: Nr33,
    /// NR34 - Frequency high/control
    pub nr34: Nr34,
    /// Wave RAM (32 4-bit samples, stored as 16 bytes)
    pub wave_ram: [u8; 16],
    /// Length counter (decrements each frame sequencer step if enabled)
    pub length_counter: u8,
    /// Channel enabled flag
    pub enabled: bool,
}

impl Default for Channel3 {
    fn default() -> Self {
        Self {
            nr30: Nr30(0),
            nr31: Nr31(0),
            nr32: Nr32(0),
            nr33: Nr33(0),
            nr34: Nr34(0),
            wave_ram: [0; 16],
            length_counter: 0,
            enabled: false,
        }
    }
}

impl Channel3 {
    /// Read a register by offset (0=NR30, 1=NR31, 2=NR32, 3=NR33, 4=NR34)
    pub const fn read_reg(&self, offset: u8) -> u8 {
        match offset {
            0 => self.nr30.read_reg(),
            1 => self.nr31.read_reg(),
            2 => self.nr32.read_reg(),
            3 => self.nr33.read_reg(),
            4 => self.nr34.read_reg(),
            _ => 0xFF,
        }
    }
    /// Write a register by offset (0=NR30, 1=NR31, 2=NR32, 3=NR33, 4=NR34)
    pub const fn write_reg(&mut self, offset: u8, value: u8) {
        match offset {
            0 => self.nr30.write_reg(value),
            1 => self.nr31.write_reg(value),
            2 => self.nr32.write_reg(value),
            3 => self.nr33.write_reg(value),
            4 => self.nr34.write_reg(value),
            _ => {}
        }
    }
    /// Read a byte from wave RAM (0–15)
    pub fn read_wave_ram(&self, index: usize) -> u8 {
        self.wave_ram.get(index).copied().unwrap_or(0xFF)
    }
    /// Writes a value to wave RAM at the given index
    pub const fn write_wave_ram(&mut self, index: usize, val: u8) {
        if index < self.wave_ram.len() {
            self.wave_ram[index] = val;
        }
    }
    /// Trigger the channel (start playback, reload length if zero)
    pub fn trigger(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            // 256 - length (0..=255) always fits in u8, as per hardware
            let val = 256u16 - u16::from(self.nr31.length());
            debug_assert!(
                (1..=256).contains(&val),
                "length_counter must be in 1..=256 (hardware guarantee)"
            );
            self.length_counter = u8::try_from(val).unwrap_or(0); // Safe: val is always in range
        }
    }
    /// Return true if the channel is enabled
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
    /// Return true if length counter is enabled (from NR34)
    pub const fn length_enable(&self) -> bool {
        self.nr34.length_enable()
    }
}
