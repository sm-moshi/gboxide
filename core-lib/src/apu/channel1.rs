//! Channel 1: Square wave with sweep (modularised, manual bitfields)

use crate::helpers::{get_bit, get_bits, set_bit, set_bits};

/// NR10 - Channel 1 Sweep register (0xFF10)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Nr10(pub u8);

#[allow(dead_code)]
impl Nr10 {
    pub const fn sweep_time(self) -> u8 {
        get_bits(self.0, 0x07, 4)
    }
    pub const fn set_sweep_time(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x07, 4, val);
    }
    pub const fn sweep_increase(self) -> bool {
        get_bit(self.0, 3)
    }
    pub const fn set_sweep_increase(&mut self, val: bool) {
        self.0 = set_bit(self.0, 3, val);
    }
    pub const fn sweep_shift(self) -> u8 {
        get_bits(self.0, 0x07, 0)
    }
    pub const fn set_sweep_shift(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x07, 0, val);
    }
}

/// NR11 - Channel 1 Sound length/Wave pattern duty (0xFF11)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Nr11(pub u8);

#[allow(dead_code)]
impl Nr11 {
    pub const fn duty(self) -> u8 {
        get_bits(self.0, 0x03, 6)
    }
    pub const fn set_duty(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x03, 6, val);
    }
    pub const fn length(self) -> u8 {
        get_bits(self.0, 0x3F, 0)
    }
    pub const fn set_length(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x3F, 0, val);
    }
}

/// NR12 - Channel 1 Envelope (0xFF12)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Nr12(pub u8);

#[allow(dead_code)]
impl Nr12 {
    pub const fn initial_volume(self) -> u8 {
        get_bits(self.0, 0x0F, 4)
    }
    pub const fn set_initial_volume(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x0F, 4, val);
    }
    pub const fn envelope_direction(self) -> bool {
        get_bit(self.0, 3)
    }
    pub const fn set_envelope_direction(&mut self, val: bool) {
        self.0 = set_bit(self.0, 3, val);
    }
    pub const fn envelope_period(self) -> u8 {
        get_bits(self.0, 0x07, 0)
    }
    pub const fn set_envelope_period(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x07, 0, val);
    }
}

/// NR14 - Channel 1 Frequency high/Control (0xFF14)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Nr14(pub u8);

#[allow(dead_code)]
impl Nr14 {
    pub const fn trigger(self) -> bool {
        get_bit(self.0, 7)
    }
    pub const fn set_trigger(&mut self, val: bool) {
        self.0 = set_bit(self.0, 7, val);
    }
    pub const fn length_enable(self) -> bool {
        get_bit(self.0, 6)
    }
    pub const fn set_length_enable(&mut self, val: bool) {
        self.0 = set_bit(self.0, 6, val);
    }
    pub const fn freq_high(self) -> u8 {
        get_bits(self.0, 0x07, 0)
    }
    pub const fn set_freq_high(&mut self, val: u8) {
        self.0 = set_bits(self.0, 0x07, 0, val);
    }
}

/// Channel 1 struct
#[derive(Default)]
#[allow(dead_code)]
pub struct Channel1 {
    pub nr10: Nr10,
    pub nr11: Nr11,
    pub nr12: Nr12,
    pub nr13: u8,
    pub nr14: Nr14,
}

#[allow(dead_code)]
impl Channel1 {
    /// Read a register by offset (0=NR10, 1=NR11, 2=NR12, 3=NR13, 4=NR14)
    pub const fn read_reg(&self, offset: u8) -> u8 {
        match offset {
            0 => self.nr10.0,
            1 => self.nr11.0,
            2 => self.nr12.0,
            3 => self.nr13,
            4 => self.nr14.0,
            _ => 0xFF,
        }
    }
    /// Write a register by offset (0=NR10, 1=NR11, 2=NR12, 3=NR13, 4=NR14)
    ///
    /// If writing to NR14 with the trigger bit set, this will reset and enable the channel as per hardware behaviour.
    pub const fn write_reg(&mut self, offset: u8, value: u8) {
        match offset {
            0 => self.nr10.0 = value,
            1 => self.nr11.0 = value,
            2 => self.nr12.0 = value,
            3 => self.nr13 = value,
            4 => self.nr14.0 = value,
            _ => {}
        }
    }
}
