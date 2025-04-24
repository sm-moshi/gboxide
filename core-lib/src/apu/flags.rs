//! Bitflags for APU channel enables and status

bitflags::bitflags! {
    /// Channel enable/status flags for the APU
    pub struct ChannelFlags: u8 {
        const CH1 = 0b0001;
        const CH2 = 0b0010;
        const CH3 = 0b0100;
        const CH4 = 0b1000;
    }
}
