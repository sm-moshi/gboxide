use bitflags::bitflags;

bitflags! {
    /// LCD Control Register (LCDC) at 0xFF40
    pub struct LcdControl: u8 {
        /// Bit 7 - LCD Display Enable (0=Off, 1=On)
        const LCD_ENABLE = 0b1000_0000;
        
        /// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        const WINDOW_TILEMAP = 0b0100_0000;
        
        /// Bit 5 - Window Display Enable (0=Off, 1=On)
        const WINDOW_ENABLE = 0b0010_0000;
        
        /// Bit 4 - BG & Window Tile Data Select (0=8800-97FF, 1=8000-8FFF)
        const BG_WINDOW_TILE_DATA = 0b0001_0000;
        
        /// Bit 3 - BG Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        const BG_TILEMAP = 0b0000_1000;
        
        /// Bit 2 - OBJ (Sprite) Size (0=8x8, 1=8x16)
        const SPRITE_SIZE = 0b0000_0100;
        
        /// Bit 1 - OBJ (Sprite) Display Enable (0=Off, 1=On)
        const SPRITE_ENABLE = 0b0000_0010;
        
        /// Bit 0 - BG Display (0=Off, 1=On)
        const BG_WINDOW_ENABLE = 0b0000_0001;
    }
}
