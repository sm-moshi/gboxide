use bitflags::bitflags;

bitflags! {
    /// LCD Status Register (STAT) at 0xFF41
    pub struct LcdStatus: u8 {
        /// Bit 6 - LYC=LY Coincidence Interrupt (1=Enable)
        const LYC_INTERRUPT = 0b0100_0000;
        
        /// Bit 5 - Mode 2 OAM Interrupt (1=Enable)
        const OAM_INTERRUPT = 0b0010_0000;
        
        /// Bit 4 - Mode 1 V-Blank Interrupt (1=Enable)
        const VBLANK_INTERRUPT = 0b0001_0000;
        
        /// Bit 3 - Mode 0 H-Blank Interrupt (1=Enable)
        const HBLANK_INTERRUPT = 0b0000_1000;
        
        /// Bit 2 - Coincidence Flag (0=LYC≠LY, 1=LYC=LY)
        const LYC_EQUAL_LY = 0b0000_0100;
        
        /// Mode Flag (bits 0-1)
        /// 00: H-Blank
        /// 01: V-Blank
        /// 10: Searching OAM
        /// 11: Transferring Data to LCD Driver
        const MODE_FLAG_MASK = 0b0000_0011;
        
        /// Mode 0 - H-Blank
        const MODE_HBLANK = 0b0000_0000;
        
        /// Mode 1 - V-Blank
        const MODE_VBLANK = 0b0000_0001;
        
        /// Mode 2 - Searching OAM
        const MODE_OAM = 0b0000_0010;
        
        /// Mode 3 - Transferring Data to LCD Driver
        const MODE_TRANSFER = 0b0000_0011;
    }
}
