use crate::helpers::{get_bit, get_bits, set_bit};

#[derive(Debug, Default)]
pub struct Interrupts {
    /// Interrupt Enable (IE) register at 0xFFFF
    pub ie: u8,
    /// Interrupt Flag (IF) register at 0xFF0F
    pub if_: u8,
    /// Interrupt Master Enable flag
    pub ime: bool,
    /// IME scheduled to be enabled (after EI instruction)
    ime_scheduled: bool,
}

/// Interrupt flags that can be set by hardware events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptFlag {
    VBlank,  // Vertical blanking interval
    LcdStat, // LCD status triggers
    Timer,   // Timer overflow
    Serial,  // Serial transfer completion
    Joypad,  // Joypad input
}

impl Interrupts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any enabled interrupts are pending, regardless of IME
    pub const fn pending_regardless_of_ime(&self) -> bool {
        get_bits(self.ie, self.if_, 0) != 0
    }

    /// Check if any enabled interrupts are pending
    pub const fn pending(&self) -> bool {
        self.ime && self.pending_regardless_of_ime()
    }

    /// Get the highest priority pending interrupt
    pub const fn get_interrupt(&self) -> Option<InterruptFlag> {
        if !self.pending() {
            return None;
        }

        let active = self.ie & self.if_;
        Some(if get_bit(active, InterruptFlag::VBlank as u8) {
            InterruptFlag::VBlank
        } else if get_bit(active, InterruptFlag::LcdStat as u8) {
            InterruptFlag::LcdStat
        } else if get_bit(active, InterruptFlag::Timer as u8) {
            InterruptFlag::Timer
        } else if get_bit(active, InterruptFlag::Serial as u8) {
            InterruptFlag::Serial
        } else {
            InterruptFlag::Joypad
        })
    }

    /// Schedule IME to be enabled after the next instruction
    pub const fn schedule_enable_ime(&mut self) {
        self.ime_scheduled = true;
    }

    /// Update IME state (called after each instruction)
    pub const fn update_ime(&mut self) {
        if self.ime_scheduled {
            self.ime = true;
            self.ime_scheduled = false;
        }
    }

    /// Request an interrupt by setting its flag
    pub const fn request(&mut self, flag: InterruptFlag) {
        self.if_ = set_bit(self.if_, flag as u8, true);
    }

    /// Clear an interrupt flag after handling
    pub const fn clear(&mut self, flag: InterruptFlag) {
        self.if_ = set_bit(self.if_, flag as u8, false);
    }

    /// Get the interrupt vector address for a given interrupt
    pub const fn get_vector(flag: InterruptFlag) -> u16 {
        match flag {
            InterruptFlag::VBlank => 0x0040,
            InterruptFlag::LcdStat => 0x0048,
            InterruptFlag::Timer => 0x0050,
            InterruptFlag::Serial => 0x0058,
            InterruptFlag::Joypad => 0x0060,
        }
    }

    /// Read the Interrupt Flag (IF) register
    /// Upper 3 bits always read as 1
    pub const fn read_if(&self) -> u8 {
        self.if_ | 0xE0
    }

    /// Write to the Interrupt Flag (IF) register
    /// Only lower 5 bits can be written
    pub const fn write_if(&mut self, value: u8) {
        self.if_ = get_bits(value, 0x1F, 0);
    }

    /// Read the Interrupt Enable (IE) register
    /// Upper 3 bits always read as 1
    pub const fn read_ie(&self) -> u8 {
        self.ie | 0xE0
    }

    /// Write to the Interrupt Enable (IE) register
    /// Only lower 5 bits can be written
    pub const fn write_ie(&mut self, value: u8) {
        self.ie = get_bits(value, 0x1F, 0);
    }

    /// Disable interrupts immediately
    pub const fn disable_ime(&mut self) {
        self.ime = false;
        self.ime_scheduled = false;
    }

    #[cfg(test)]
    pub const fn set_ie(&mut self, value: u8) {
        self.ie = get_bits(value, 0x1F, 0);
    }

    #[cfg(test)]
    pub const fn set_if(&mut self, value: u8) {
        self.if_ = get_bits(value, 0x1F, 0);
    }

    #[cfg(test)]
    pub const fn set_ime(&mut self, value: bool) {
        self.ime = value;
    }

    #[cfg(test)]
    pub const fn enable(&mut self, flag: InterruptFlag) {
        self.ie = set_bit(self.ie, flag as u8, true);
    }
}

#[cfg(test)]
mod tests;
