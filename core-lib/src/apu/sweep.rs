//! Sweep logic for APU channels (modularised)

/// Frequency sweep logic for Channel 1
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Sweep {
    /// Sweep enabled flag
    pub enabled: bool,
    /// Sweep period (in sequencer steps)
    pub period: u8,
    /// Sweep negate flag (true = decrease, false = increase)
    pub negate: bool,
    /// Sweep shift amount
    pub shift: u8,
    /// Sweep timer
    pub timer: u8,
    /// Shadow frequency (used for calculations)
    pub shadow_freq: u16,
}

impl Sweep {
    /// Clock the sweep unit (hardware-accurate)
    pub const fn clock(&mut self) {
        // Only run sweep if enabled and shift > 0
        if self.enabled && self.shift > 0 {
            // Calculate new frequency
            let delta = self.shadow_freq >> self.shift;
            let new_freq = if self.negate {
                self.shadow_freq.wrapping_sub(delta)
            } else {
                self.shadow_freq.wrapping_add(delta)
            };
            // Overflow detection: if new_freq > 2047, disable sweep
            if new_freq > 0x7FF {
                self.enabled = false;
            } else if self.shift > 0 {
                // Update shadow frequency and (in real hardware, would update channel freq)
                self.shadow_freq = new_freq;
                // (Caller must update channel frequency register)
            }
        }
    }
    /// Trigger the sweep (reset state and perform initial calculation if needed)
    pub const fn trigger(&mut self, freq: u16) {
        self.enabled = self.period != 0 || self.shift != 0;
        self.timer = if self.period == 0 { 8 } else { self.period };
        self.shadow_freq = freq;
        // Initial sweep calculation (hardware: overflow disables channel immediately)
        if self.shift > 0 {
            let delta = self.shadow_freq >> self.shift;
            let new_freq = if self.negate {
                self.shadow_freq.wrapping_sub(delta)
            } else {
                self.shadow_freq.wrapping_add(delta)
            };
            if new_freq > 0x7FF {
                self.enabled = false;
            }
        }
    }
    /// Reset the sweep state
    pub const fn reset(&mut self) {
        self.enabled = false;
        self.timer = 0;
        self.shadow_freq = 0;
    }
}
