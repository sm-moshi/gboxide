use super::*;

#[test]
fn test_valid_romonly_cartridge_construction() {
    // Minimal valid ROM: 0x150 bytes, type 0x00 (ROM ONLY), ROM size 0x00 (32KB), RAM size 0x00 (None)
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0x00; // ROM ONLY
    rom[0x148] = 0x00; // 32KB ROM
    rom[0x149] = 0x00; // No RAM
    let cart = Cartridge::new(rom);
    assert!(cart.is_ok());
    let cart = cart.unwrap();
    assert_eq!(cart.cart_type, CartridgeType::RomOnly);
    assert_eq!(cart.rom_size, RomSize::Size32KB);
    assert_eq!(cart.ram_size, RamSize::None);
    assert!(!cart.has_battery);
}

#[test]
fn test_invalid_rom_too_small() {
    let rom = vec![0; 0x100]; // Too small
    let cart = Cartridge::new(rom);
    assert!(matches!(cart, Err(CartridgeError::InvalidSize(0x100))));
}

#[test]
fn test_unsupported_cartridge_type() {
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0xAA; // Invalid/unsupported type
    rom[0x148] = 0x00;
    rom[0x149] = 0x00;
    let cart = Cartridge::new(rom);
    assert!(matches!(
        cart,
        Err(CartridgeError::UnsupportedCartridgeType(0xAA))
    ));
}

#[test]
fn test_invalid_rom_size_header() {
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0x00;
    rom[0x148] = 0xFF; // Invalid ROM size
    rom[0x149] = 0x00;
    let cart = Cartridge::new(rom);
    assert!(matches!(cart, Err(CartridgeError::InvalidSize(0xFF))));
}

#[test]
fn test_invalid_ram_size_header() {
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0x00;
    rom[0x148] = 0x00;
    rom[0x149] = 0xFF; // Invalid RAM size
    let cart = Cartridge::new(rom);
    assert!(matches!(cart, Err(CartridgeError::InvalidSize(0xFF))));
}

#[test]
fn test_mbc1_battery_flag() {
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0x03; // MBC1 + RAM + BATTERY
    rom[0x148] = 0x00;
    rom[0x149] = 0x00;
    let cart = Cartridge::new(rom).unwrap();
    assert!(cart.has_battery);
    assert!(matches!(
        cart.cart_type,
        CartridgeType::Mbc1 { battery: true, .. }
    ));
}

#[test]
fn test_title_parsing_old_and_new() {
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0x00;
    rom[0x148] = 0x00;
    rom[0x149] = 0x00;
    // Old style: 0x134..=0x143
    let title = b"TESTTITLE";
    rom[0x134..0x134 + title.len()].copy_from_slice(title);
    let cart = Cartridge::new(rom.clone()).unwrap();
    assert_eq!(cart.title(), "TESTTITLE");
    // New style: 0x14B == 0x33, 0x134..0x13F
    let mut rom_new = rom.clone();
    rom_new[0x14B] = 0x33;
    let cart_new = Cartridge::new(rom_new).unwrap();
    assert_eq!(cart_new.title(), "TESTTITLE");
}

#[test]
fn test_read_and_write_rom_and_ram() {
    let mut rom = vec![0; 0x2000 * 2]; // 2 ROM banks
    rom.resize(0x150, 0); // Ensure header is present
    rom[0x147] = 0x00;
    rom[0x148] = 0x00;
    rom[0x149] = 0x00;
    let mut cart = Cartridge::new(rom).unwrap();
    // ROM read
    assert_eq!(cart.read(0x0000).unwrap(), 0x00);
    // Write to RAM region (should be 0xFF if not enabled)
    assert_eq!(cart.read(0xA000).unwrap(), 0xFF);
    // Enable RAM and write
    cart.write(0x0000, 0x0A).unwrap();
    let _ = cart.write(0xA000, 0x42);
    // Now RAM is enabled, but since data is not large enough, should error
    assert!(cart.read(0xA000).is_err());
}

#[test]
fn test_invalid_address_read_write() {
    let mut rom = vec![0; 0x150];
    rom[0x147] = 0x00;
    rom[0x148] = 0x00;
    rom[0x149] = 0x00;
    let mut cart = Cartridge::new(rom).unwrap();
    // Out of bounds
    assert!(matches!(
        cart.read(0xC000),
        Err(CartridgeError::InvalidAddress(0xC000))
    ));
    assert!(matches!(
        cart.write(0xC000, 0x12),
        Err(CartridgeError::InvalidAddress(0xC000))
    ));
}
