//! Game Boy (DMG/CGB) Audio Processing Unit (APU)
//!
//! Implements all 4 sound channels, mixing, frame sequencer, and memory-mapped I/O.
//!
//! - DMG: 4 channels (2x Square, Wave, Noise)
//! - CGB: Stereo panning, advanced features
//!
//! See: [`https://gbdev.io/pandocs/Audio.html`]

mod channel1;
mod channel2;
mod channel3;
mod channel4;
mod envelope;
mod flags;
mod sweep;
mod wave_duty;

pub use flags::*;
pub use sweep::*;
pub use wave_duty::*;

use channel3::Channel3;
use channel4::Channel4;

/// APU register addresses
pub const NR10_ADDR: u16 = 0xFF10;
pub const NR11_ADDR: u16 = 0xFF11;
pub const NR12_ADDR: u16 = 0xFF12;
pub const NR13_ADDR: u16 = 0xFF13;
pub const NR14_ADDR: u16 = 0xFF14;
pub const NR21_ADDR: u16 = 0xFF16;
pub const NR22_ADDR: u16 = 0xFF17;
pub const NR23_ADDR: u16 = 0xFF18;
pub const NR24_ADDR: u16 = 0xFF19;
pub const NR30_ADDR: u16 = 0xFF1A;
pub const NR31_ADDR: u16 = 0xFF1B;
pub const NR32_ADDR: u16 = 0xFF1C;
pub const NR33_ADDR: u16 = 0xFF1D;
pub const NR34_ADDR: u16 = 0xFF1E;
pub const NR41_ADDR: u16 = 0xFF20;
pub const NR42_ADDR: u16 = 0xFF21;
pub const NR43_ADDR: u16 = 0xFF22;
pub const NR44_ADDR: u16 = 0xFF23;
pub const NR50_ADDR: u16 = 0xFF24;
pub const NR51_ADDR: u16 = 0xFF25;
pub const NR52_ADDR: u16 = 0xFF26;
pub const WAVE_RAM_START: u16 = 0xFF30;
pub const WAVE_RAM_END: u16 = 0xFF3F;

/// Top-level APU struct
///
/// Holds all channel state, frame sequencer, and output buffer.
#[derive(Default)]
pub struct Apu {
    /// Channel 1: Square wave with sweep
    pub ch1: Channel1,
    /// Channel 2: Square wave
    pub ch2: Channel2,
    /// Channel 3: Wave channel
    pub ch3: Channel3,
    /// Channel 4: Noise channel
    pub ch4: Channel4,
    /// Frame sequencer state
    pub sequencer: FrameSequencer,
    /// Wave RAM (Channel 3)
    pub wave_ram: [u8; 16],
    /// Output buffer (to be filled by mixing logic)
    pub output_buffer: Vec<f32>,
    /// Master enable (`NR52`)
    pub enabled: bool,
}

/// Channel 1: Square wave with sweep
pub struct Channel1 {
    /// NR10 - Sweep register (0xFF10)
    pub nr10: Nr10,
    /// NR11 - Sound length/wave pattern duty (0xFF11)
    pub nr11: Nr11,
    /// NR12 - Envelope (0xFF12)
    pub nr12: Nr12,
    /// NR13 - Frequency low (0xFF13)
    pub nr13: u8,
    /// NR14 - Frequency high/control (0xFF14)
    pub nr14: Nr14,
    /// Length counter (decrements if enabled)
    pub length_counter: u8,
    /// Envelope timer (counts down to envelope step)
    pub envelope_timer: u8,
    /// Envelope volume (current output volume)
    pub envelope_volume: u8,
    /// Sweep state
    pub sweep: Sweep,
    /// Channel enabled flag
    pub enabled: bool,
}

impl Default for Channel1 {
    fn default() -> Self {
        Self {
            nr10: Nr10(0),
            nr11: Nr11(0),
            nr12: Nr12(0),
            nr13: 0,
            nr14: Nr14(0),
            length_counter: 0,
            envelope_timer: 0,
            envelope_volume: 0,
            sweep: Sweep::default(),
            enabled: false,
        }
    }
}

/// NR10 - Channel 1 Sweep register (0xFF10)
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | - | - | S | S | S | N | P | P |
/// Bits 6-4: Sweep time, 3: Sweep increase/decrease, 2-0: Sweep shift
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr10(pub u8);

#[allow(dead_code)]
impl Nr10 {
    pub const fn read_reg(&self) -> u8 {
        self.0
    }
    /// Write a register by offset (0=NR10)
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR11 - Channel 1 Sound length/Wave pattern duty (0xFF11)
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | D | D | D | D | L | L | L | L |
/// Bits 7-6: Duty, 5-0: Length
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr11(pub u8);

#[allow(dead_code)]
impl Nr11 {
    pub const fn read_reg(&self) -> u8 {
        self.0
    }
    /// Write a register by offset (0=NR11)
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR12 - Channel 1 Envelope (0xFF12)
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | V | V | V | D | P | P | P | P |
/// Bits 7-4: Initial volume, 3: Envelope direction, 2-0: Period
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr12(pub u8);

#[allow(dead_code)]
impl Nr12 {
    pub const fn read_reg(&self) -> u8 {
        self.0
    }
    /// Write a register by offset (0=NR12)
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR14 - Channel 1 Frequency high/Control (0xFF14)
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | I | - | - | - | - | F | F | F |
/// Bit 7: Trigger, 6: Length enable, 2-0: Frequency high bits
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr14(pub u8);

#[allow(dead_code)]
impl Nr14 {
    pub const fn read_reg(&self) -> u8 {
        self.0
    }
    /// Write a register by offset (0=NR14)
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

impl Channel1 {
    /// Read a register by offset (0=NR10, 1=NR11, 2=NR12, 3=NR13, 4=NR14)
    pub const fn read_reg(&self, offset: u8) -> u8 {
        match offset {
            0 => self.nr10.read_reg(),
            1 => self.nr11.read_reg(),
            2 => self.nr12.read_reg(),
            3 => self.nr13,
            4 => self.nr14.read_reg(),
            _ => 0xFF,
        }
    }
    /// Write a register by offset (0=NR10, 1=NR11, 2=NR12, 3=NR13, 4=NR14)
    pub fn write_reg(&mut self, offset: u8, value: u8) {
        match offset {
            0 => self.nr10.write_reg(value),
            1 => self.nr11.write_reg(value),
            2 => self.nr12.write_reg(value),
            3 => self.nr13 = value,
            4 => {
                self.nr14.write_reg(value);
                if value & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => {}
        }
    }
    /// Hardware-accurate trigger logic for Channel 1 (see Pandocs)
    ///
    /// - Enables the channel
    /// - Resets length counter if zero
    /// - Resets envelope timer and volume
    /// - Resets sweep state
    /// - Sets sweep timer and shadow frequency
    pub fn trigger(&mut self) {
        // 1. Enable the channel
        self.enabled = true;
        // 2. If length counter is zero, set to 64 - (NR11 length bits 0-5)
        let length_bits = self.nr11.0 & 0x3F; // bits 0-5
        if self.length_counter == 0 {
            let len = 64u8.saturating_sub(length_bits);
            self.length_counter = if len == 0 { 64 } else { len };
        }
        // 3. Envelope: set volume and timer
        // Initial volume: bits 4-7 of NR12
        self.envelope_volume = (self.nr12.0 >> 4) & 0x0F;
        // Envelope period: bits 0-2 of NR12
        let period = self.nr12.0 & 0x07;
        self.envelope_timer = if period == 0 { 8 } else { period };
        // 4. Sweep: set state from NR10 and current frequency
        // Sweep period: bits 4-6 of NR10
        self.sweep.period = (self.nr10.0 >> 4) & 0x07;
        // Sweep negate: bit 3 of NR10 (0 = increase, 1 = decrease)
        self.sweep.negate = (self.nr10.0 & 0x08) == 0;
        // Sweep shift: bits 0-2 of NR10
        self.sweep.shift = self.nr10.0 & 0x07;
        self.sweep.timer = if self.sweep.period == 0 {
            8
        } else {
            self.sweep.period
        };
        // Shadow frequency is the current 11-bit frequency
        let freq = ((u16::from(self.nr14.0 & 0x07)) << 8) | u16::from(self.nr13);
        self.sweep.shadow_freq = freq;
        // Sweep enabled if period or shift is nonzero
        self.sweep.enabled = self.sweep.period != 0 || self.sweep.shift != 0;
        // If sweep shift is nonzero, perform initial calculation and disable if overflow
        if self.sweep.shift != 0 {
            let delta = self.sweep.shadow_freq >> self.sweep.shift;
            let new_freq = if self.sweep.negate {
                self.sweep.shadow_freq.wrapping_sub(delta)
            } else {
                self.sweep.shadow_freq.wrapping_add(delta)
            };
            if new_freq > 0x7FF {
                self.enabled = false;
            }
        }
        // (Frequency timer reset is handled elsewhere if needed)
    }
    /// Tick the length counter (frame sequencer steps 0,2,4,6)
    /// This disables the channel when the length counter reaches zero, as per hardware.
    pub fn tick_length(&mut self) {
        if self.nr14.0 & 0x40 != 0 && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
    /// Tick the envelope (frame sequencer step 7)
    /// This updates the envelope volume according to the envelope period and direction.
    pub fn tick_envelope(&mut self) {
        let period = self.nr12.0 & 0x07;
        if period != 0 {
            if self.envelope_timer > 0 {
                self.envelope_timer -= 1;
            }
            if self.envelope_timer == 0 {
                self.envelope_timer = period;
                let direction = (self.nr12.0 & 0x08) != 0;
                if direction && self.envelope_volume < 0x0F {
                    self.envelope_volume += 1;
                } else if !direction && self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                }
            }
        }
    }
    /// Tick the sweep unit (frame sequencer steps 2,6)
    /// This clocks the sweep logic, updating frequency and disabling the channel on overflow.
    pub fn tick_sweep(&mut self) {
        self.sweep.clock();
        // If sweep disables itself, also disable the channel
        if !self.sweep.enabled {
            self.enabled = false;
        }
    }
}

/// Envelope register for Channel 2 (NR22)
/// Follows the Mooneye GB approach: explicit fields for each logical part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Envelope {
    pub volume: u8,       // bits 7-4
    pub increasing: bool, // bit 3
    pub period: u8,       // bits 2-0
}

impl Envelope {
    pub const fn new() -> Self {
        Self {
            volume: 0,
            increasing: false,
            period: 0,
        }
    }
    /// Read the envelope register as a raw u8 (NR22 format)
    pub const fn read_reg(&self) -> u8 {
        (self.volume << 4) | (if self.increasing { 1 << 3 } else { 0 }) | (self.period & 0x07)
    }
    /// Write a raw u8 value to the envelope register (NR22 format)
    pub fn write_reg(&mut self, value: u8) {
        self.volume = (value >> 4) & 0x0F;
        self.increasing = (value & 0x08) != 0;
        self.period = value & 0x07;
    }
}

/// Channel 2: Square wave
pub struct Channel2 {
    /// NR21 - Sound length/wave pattern duty (0xFF16)
    pub nr21: Nr21,
    /// NR22 - Envelope (0xFF17)
    pub nr22: Envelope,
    /// NR23 - Frequency low (0xFF18)
    pub nr23: u8,
    /// NR24 - Frequency high/control (0xFF19)
    pub nr24: Nr24,
    /// Length counter (decrements if enabled)
    pub length_counter: u8,
    /// Envelope timer (counts down to envelope step)
    pub envelope_timer: u8,
    /// Envelope volume (current output volume)
    pub envelope_volume: u8,
    /// Channel enabled flag
    pub enabled: bool,
}

impl Default for Channel2 {
    fn default() -> Self {
        Self {
            nr21: Nr21(0),
            nr22: Envelope::new(),
            nr23: 0,
            nr24: Nr24(0),
            length_counter: 0,
            envelope_timer: 0,
            envelope_volume: 0,
            enabled: false,
        }
    }
}

/// NR21 - Channel 2 Sound length/Wave pattern duty (0xFF16)
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | D | D | D | D | L | L | L | L |
/// Bits 7-6: Duty, 5-0: Length
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr21(pub u8);

impl Nr21 {
    pub const fn read_reg(&self) -> u8 {
        self.0
    }
    /// Write a register by offset (0=NR21)
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

/// NR24 - Channel 2 Frequency high/Control (0xFF19)
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | I | - | - | - | - | F | F | F |
/// Bit 7: Trigger, 6: Length enable, 2-0: Frequency high bits
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nr24(pub u8);

impl Nr24 {
    pub const fn read_reg(&self) -> u8 {
        self.0
    }
    /// Write a register by offset (0=NR24)
    pub fn write_reg(&mut self, value: u8) {
        self.0 = value;
    }
}

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
    pub fn write_reg(&mut self, offset: u8, value: u8) {
        match offset {
            0 => self.nr21.write_reg(value),
            1 => self.nr22.write_reg(value),
            2 => self.nr23 = value,
            3 => self.nr24.write_reg(value),
            _ => {}
        }
    }
    /// Tick the length counter (frame sequencer steps 0,2,4,6)
    pub fn tick_length(&mut self) {
        if self.nr24.0 & 0x40 != 0 && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
    /// Tick the envelope (frame sequencer step 7)
    pub fn tick_envelope(&mut self) {
        let period = self.nr22.period;
        if period != 0 {
            if self.envelope_timer > 0 {
                self.envelope_timer -= 1;
            }
            if self.envelope_timer == 0 {
                self.envelope_timer = period;
                let direction = self.nr22.increasing;
                if direction && self.envelope_volume < 0x0F {
                    self.envelope_volume += 1;
                } else if !direction && self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                }
            }
        }
    }
}

/// Frame sequencer (timing for envelopes, length, sweep)
#[derive(Default)]
pub struct FrameSequencer {
    /// Number of CPU cycles until the next frame sequencer step (8192 cycles per step)
    pub cycles_until_step: u32,
    /// Current step index (0..=7)
    pub step: u8,
}

impl FrameSequencer {
    /// Number of CPU cycles per frame sequencer step (512 Hz)
    pub const STEP_CYCLES: u32 = 8192;
    /// Total number of steps in the frame sequencer
    pub const NUM_STEPS: u8 = 8;

    /// Advance the frame sequencer by a number of CPU cycles.
    /// Returns Some(step) if a new step occurred, or None otherwise.
    pub fn tick(&mut self, mut cycles: u32) -> Option<u8> {
        let mut step_triggered = None;
        while cycles > 0 {
            if self.cycles_until_step == 0 {
                self.cycles_until_step = Self::STEP_CYCLES;
                self.step = (self.step + 1) % Self::NUM_STEPS;
                step_triggered = Some(self.step);
            }
            let advance = cycles.min(self.cycles_until_step);
            self.cycles_until_step -= advance;
            cycles -= advance;
        }
        step_triggered
    }
}

#[allow(dead_code)]
impl Apu {
    /// Tick the APU by a number of CPU cycles, advancing the frame sequencer.
    ///
    /// This should be called from the main emulator loop, passing the number of CPU cycles elapsed.
    /// The frame sequencer advances every 8192 cycles (512 Hz), triggering length, envelope, and sweep events.
    pub fn tick(&mut self, cycles: u32) {
        let step = self.sequencer.tick(cycles);
        if let Some(step) = step {
            // Frame sequencer event dispatch (see: https://gbdev.io/pandocs/Audio.html#frame-sequencer)
            // Steps 0,2,4,6: Length counter
            // Steps 2,6: Sweep (Channel 1)
            // Step 7: Envelope (Channels 1,2,4)
            match step {
                0 | 2 | 4 | 6 => {
                    self.ch1.tick_length();
                    self.ch2.tick_length();
                    self.ch3.tick_length();
                    self.ch4.tick_length();
                }
                _ => {}
            }
            match step {
                2 | 6 => {
                    self.ch1.tick_sweep();
                }
                _ => {}
            }
            if step == 7 {
                self.ch1.tick_envelope();
                self.ch2.tick_envelope();
                self.ch4.tick_envelope();
            }
        }
    }

    /// Read from an APU register by address (memory-mapped I/O)
    ///
    /// Now supports Channel 3 (0xFF1A–0xFF1E, 0xFF30–0xFF3F). Returns 0xFF for unmapped addresses.
    pub fn read(&self, addr: u16) -> u8 {
        if addr != NR52_ADDR && !self.enabled {
            return 0xFF;
        }
        match addr {
            NR13_ADDR | NR23_ADDR | NR31_ADDR | NR33_ADDR | NR41_ADDR => 0xFF,
            NR10_ADDR => self.ch1.read_reg(0),
            NR11_ADDR => self.ch1.read_reg(1),
            NR12_ADDR => self.ch1.read_reg(2),
            NR14_ADDR => self.ch1.read_reg(4),
            NR21_ADDR => self.ch2.read_reg(0),
            NR22_ADDR => self.ch2.read_reg(1),
            NR24_ADDR => self.ch2.read_reg(3),
            // Channel 3 registers
            NR30_ADDR => self.ch3.read_reg(0),
            NR32_ADDR => self.ch3.read_reg(2),
            NR34_ADDR => self.ch3.read_reg(4),
            // Channel 3 wave RAM (FF30–FF3F)
            WAVE_RAM_START..=WAVE_RAM_END => {
                let index = (addr - WAVE_RAM_START) as usize;
                self.ch3.read_wave_ram(index)
            }
            // Channel 4 registers
            NR42_ADDR => self.ch4.read_reg(1),
            NR43_ADDR => self.ch4.read_reg(2),
            NR44_ADDR => self.ch4.read_reg(3),
            _ => 0xFF,
        }
    }

    /// Write to an APU register by address (memory-mapped I/O)
    ///
    /// Now supports Channel 3 (0xFF1A–0xFF1E, 0xFF30–0xFF3F). Writes to unmapped addresses are ignored.
    pub fn write(&mut self, addr: u16, value: u8) {
        if addr != NR52_ADDR && !self.enabled {
            return;
        }
        match addr {
            NR10_ADDR => self.ch1.write_reg(0, value),
            NR11_ADDR => self.ch1.write_reg(1, value),
            NR12_ADDR => self.ch1.write_reg(2, value),
            NR14_ADDR => self.ch1.write_reg(4, value),
            NR21_ADDR => self.ch2.write_reg(0, value),
            NR22_ADDR => self.ch2.write_reg(1, value),
            NR24_ADDR => self.ch2.write_reg(3, value),
            // Channel 3 registers
            NR30_ADDR => self.ch3.write_reg(0, value),
            NR32_ADDR => self.ch3.write_reg(2, value),
            NR34_ADDR => self.ch3.write_reg(4, value),
            // Channel 3 wave RAM (FF30–FF3F)
            WAVE_RAM_START..=WAVE_RAM_END => {
                let index = (addr - WAVE_RAM_START) as usize;
                self.ch3.write_wave_ram(index, value);
            }
            // Channel 4 registers
            NR42_ADDR => self.ch4.write_reg(1, value),
            NR43_ADDR => self.ch4.write_reg(2, value),
            NR44_ADDR => self.ch4.write_reg(3, value),
            _ => {}
        }
    }
}

impl Channel3 {
    /// Tick the length counter (frame sequencer steps 0,2,4,6)
    pub fn tick_length(&mut self) {
        if self.nr34.length_enable() && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }
}

impl Channel4 {
    /// Tick the length counter (frame sequencer steps 0,2,4,6)
    pub fn tick_length(&mut self) {
        if self.nr44.0 & 0x40 != 0 && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.dac_enabled = false;
            }
        }
    }
    /// Tick the envelope (frame sequencer step 7)
    pub fn tick_envelope(&mut self) {
        let period = self.nr42.period;
        if period != 0 {
            if self.envelope_timer > 0 {
                self.envelope_timer -= 1;
            }
            if self.envelope_timer == 0 {
                self.envelope_timer = period;
                let direction = self.nr42.increasing;
                if direction && self.nr42.volume < 0x0F {
                    self.nr42.volume += 1;
                } else if !direction && self.nr42.volume > 0 {
                    self.nr42.volume -= 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
