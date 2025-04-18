/// core-lib/src/timer/mod.rs
use bitflags::bitflags;

bitflags! {
    #[derive(Default, Debug)]
    struct TacReg: u8 {
        const ENABLE = 0b100;
        const MASK_1 = 0b010;
        const MASK_0 = 0b001;
    }
}

#[derive(Debug, Default)]
pub struct Timer {
    /// Divider register (DIV) at 0xFF04
    pub div: u8,
    /// Timer counter (TIMA) at 0xFF05
    pub tima: u8,
    /// Timer modulo (TMA) at 0xFF06
    pub tma: u8,
    /// Timer control (TAC) at 0xFF07
    tac: TacReg,
    /// Internal counter for DIV register (16-bit)
    div_counter: u16,
    /// Whether TIMA is in overflow state
    pub(crate) tima_overflow: bool,
}

impl Timer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the counter mask based on the current timer frequency
    #[must_use]
    pub const fn get_counter_mask(&self) -> u16 {
        match self.tac.bits() & 0b11 {
            0b00 => 1 << 9, // 4.096 KHz (bit 9)
            0b01 => 1 << 3, // 262.144 KHz (bit 3)
            0b10 => 1 << 5, // 65.536 KHz (bit 5)
            _ => 1 << 7,    // 16.384 KHz (bit 7)
        }
    }

    /// Get the current input bit based on the counter mask
    #[must_use]
    pub const fn get_input(&self) -> bool {
        (self.div_counter & self.get_counter_mask()) != 0
    }

    /// Step the timer forward by the given number of cycles
    /// Returns true if an interrupt should be requested
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn step(&mut self, cycles: u32) -> bool {
        let old_input = self.get_input();

        // We only care about lower 16 bits
        self.div_counter = self.div_counter.wrapping_add(cycles as u16);

        // Update DIV register (upper 8 bits of counter)
        self.div = (self.div_counter >> 8) as u8;

        // Check for falling edge if timer is enabled
        if self.tac.contains(TacReg::ENABLE) {
            let new_input = self.get_input();
            if old_input && !new_input {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.tima_overflow = true;
                }
            }
        }

        // Handle TIMA overflow
        if self.tima_overflow {
            self.tima = self.tma;
            self.tima_overflow = false;
            return true;
        }

        false
    }

    /// Reset DIV register
    pub fn reset_div(&mut self) {
        let old_input = self.get_input();
        self.div_counter = 0;
        self.div = 0;

        // Check if resetting DIV causes a timer increment
        let new_input = self.get_input();
        if old_input && !new_input {
            if self.tima == 0xFF {
                self.tima_overflow = true;
            } else {
                self.tima = self.tima.wrapping_add(1);
            }
        }
    }

    /// Write to a timer register
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.reset_div(),
            0xFF05 => {
                if !self.tima_overflow {
                    self.tima = value;
                }
            }
            0xFF06 => self.tma = value,
            0xFF07 => {
                let old_input = self.get_input();
                self.tac = TacReg::from_bits_truncate(value);

                // Check if changing TAC causes a timer increment
                let new_input = self.get_input();
                if old_input && !new_input {
                    if self.tima == 0xFF {
                        self.tima_overflow = true;
                    } else {
                        self.tima = self.tima.wrapping_add(1);
                    }
                }
            }
            _ => panic!("Invalid timer register write: {addr:#04X}"),
        }
    }

    /// Read from a timer register
    #[must_use]
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac.bits() | 0xF8, // Upper bits always read as 1
            _ => panic!("Invalid timer register read: {addr:#04X}"),
        }
    }
}

#[cfg(test)]
mod tests;
