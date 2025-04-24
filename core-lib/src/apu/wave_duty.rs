//! Wave duty logic for APU channels (modularised)

/// Wave duty patterns for square wave channels
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WaveDuty {
    /// 12.5% duty cycle (00000001)
    Duty12_5,
    /// 25% duty cycle (10000001)
    Duty25,
    /// 50% duty cycle (10000111)
    #[default]
    Duty50,
    /// 75% duty cycle (01111110)
    Duty75,
}

impl WaveDuty {
    /// Returns the 8-bit pattern for the duty cycle
    pub const fn pattern_bits(self) -> u8 {
        match self {
            Self::Duty12_5 => 0b0000_0001,
            Self::Duty25 => 0b1000_0001,
            Self::Duty50 => 0b1000_0111,
            Self::Duty75 => 0b0111_1110,
        }
    }
}
