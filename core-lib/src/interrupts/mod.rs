/// core-lib/src/interrupts/mod.rs
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

/// Interrupt bit positions
#[derive(Debug, Clone, Copy)]
pub enum InterruptFlag {
    VBlank = 0,
    LcdStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

impl Interrupts {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any enabled interrupts are pending, regardless of IME
    #[must_use]
    pub const fn pending_regardless_of_ime(&self) -> bool {
        (self.ie & self.if_) != 0
    }

    /// Check if any enabled interrupts are pending
    #[must_use]
    pub const fn pending(&self) -> bool {
        self.ime && self.pending_regardless_of_ime()
    }

    /// Get the highest priority pending interrupt
    #[must_use]
    pub fn get_interrupt(&self) -> Option<InterruptFlag> {
        if !self.pending() {
            return None;
        }

        let active = self.ie & self.if_;
        Some(if active & (1 << InterruptFlag::VBlank as u8) != 0 {
            InterruptFlag::VBlank
        } else if active & (1 << InterruptFlag::LcdStat as u8) != 0 {
            InterruptFlag::LcdStat
        } else if active & (1 << InterruptFlag::Timer as u8) != 0 {
            InterruptFlag::Timer
        } else if active & (1 << InterruptFlag::Serial as u8) != 0 {
            InterruptFlag::Serial
        } else {
            InterruptFlag::Joypad
        })
    }

    /// Schedule IME to be enabled after the next instruction
    pub fn schedule_enable_ime(&mut self) {
        self.ime_scheduled = true;
    }

    /// Update IME state (called after each instruction)
    pub fn update_ime(&mut self) {
        if self.ime_scheduled {
            self.ime = true;
            self.ime_scheduled = false;
        }
    }

    /// Request an interrupt by setting its flag
    pub fn request(&mut self, flag: InterruptFlag) {
        self.if_ |= 1 << flag as u8;
    }

    /// Clear an interrupt flag after handling
    pub fn clear(&mut self, flag: InterruptFlag) {
        self.if_ &= !(1 << flag as u8);
    }

    /// Get the interrupt vector address for a given interrupt
    #[must_use]
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
    #[must_use]
    pub const fn read_if(&self) -> u8 {
        self.if_ | 0xE0
    }

    /// Write to the Interrupt Flag (IF) register
    /// Only lower 5 bits can be written
    pub fn write_if(&mut self, value: u8) {
        self.if_ = value & 0x1F;
    }

    /// Read the Interrupt Enable (IE) register
    /// Upper 3 bits always read as 1
    #[must_use]
    pub const fn read_ie(&self) -> u8 {
        self.ie | 0xE0
    }

    /// Write to the Interrupt Enable (IE) register
    /// Only lower 5 bits can be written
    pub fn write_ie(&mut self, value: u8) {
        self.ie = value & 0x1F;
    }

    /// Disable interrupts immediately
    pub fn disable_ime(&mut self) {
        self.ime = false;
        self.ime_scheduled = false;
    }

    #[cfg(test)]
    pub fn set_ie(&mut self, value: u8) {
        self.ie = value & 0x1F;
    }

    #[cfg(test)]
    pub fn set_if(&mut self, value: u8) {
        self.if_ = value & 0x1F;
    }

    #[cfg(test)]
    pub fn set_ime(&mut self, value: bool) {
        self.ime = value;
    }

    #[cfg(test)]
    pub fn enable(&mut self, flag: InterruptFlag) {
        self.ie |= 1 << flag as u8;
    }
}

#[cfg(test)]
mod tests;
