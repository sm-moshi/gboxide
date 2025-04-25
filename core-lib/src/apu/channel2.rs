//! Channel 2: Square wave (modularised, manual bitfields)

use crate::apu::envelope::Envelope;
use crate::helpers::{get_bit, get_bits, set_bit, set_bits};

/// NR21 - Channel 2 Sound length/Wave pattern duty (0xFF16)
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr21(pub u8);

#[allow(dead_code)]
impl Nr21 {
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
    pub const fn read_reg(self) -> u8 {
        self.0
    }
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR24 - Channel 2 Frequency high/Control (0xFF19)
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr24(pub u8);

#[allow(dead_code)]
impl Nr24 {
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
    pub const fn read_reg(self) -> u8 {
        self.0
    }
    pub const fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// Channel 2 struct
#[allow(dead_code)]
#[derive(Default)]
pub struct Channel2 {
    pub nr21: Nr21,
    pub nr22: Envelope,
    pub nr23: u8,
    pub nr24: Nr24,
}

#[allow(dead_code)]
impl Channel2 {
    /// Read a register by offset (0=NR21, 1=NR22, 2=NR23, 3=NR24)
    pub const fn read_reg(&self, offset: u8) -> u8 {
        match offset {
            0 => self.nr21.read_reg(),
            1 => self.nr22.read_reg(),
            2 => self.nr23,
            3 => self.nr24.read_reg(),
            _ => 0xFF,
        }
    }
    /// Write a register by offset (0=NR21, 1=NR22, 2=NR23, 3=NR24)
    pub const fn write_reg(&mut self, offset: u8, value: u8) {
        match offset {
            0 => self.nr21.0 = value,
            1 => self.nr22.write_reg(value),
            2 => self.nr23 = value,
            3 => self.nr24.0 = value,
            _ => {}
        }
    }
}
