use core_lib::helpers::{extract_colour_index, tile_data_address, unpack_tile_attributes};
use core_lib::mmu::input::GameBoyButton;
use core_lib::mmu::mbc::{Mbc, MbcError, NoMbc};
use pretty_assertions::assert_eq;

#[test]
fn cover_gameboy_button() {
    // Cover from_index for all valid and invalid indices
    for idx in 0..10 {
        let btn = GameBoyButton::from_index(idx);
        if idx < 8 {
            assert_eq!(btn.unwrap().to_index(), idx);
        } else {
            assert!(btn.is_none());
        }
    }
    // Cover to_index for all variants
    let all = [
        GameBoyButton::Right,
        GameBoyButton::Left,
        GameBoyButton::Up,
        GameBoyButton::Down,
        GameBoyButton::A,
        GameBoyButton::B,
        GameBoyButton::Select,
        GameBoyButton::Start,
    ];
    for btn in all.iter() {
        let idx = btn.to_index();
        assert!(idx < 8);
    }
}

#[test]
fn cover_helpers() {
    // unpack_tile_attributes
    let (pal, vram, x, y, prio) = unpack_tile_attributes(0b1011_0111);
    assert_eq!(pal, 0b111);
    assert_eq!(vram, 0b0);
    assert!(x);
    assert!(!y);
    assert!(prio);
    // tile_data_address
    let addr1 = tile_data_address(0x8000, 5, false);
    let addr2 = tile_data_address(0x9000, 128, true);
    assert!(addr1 > 0);
    assert!(addr2 > 0);
    // extract_colour_index
    let idx = extract_colour_index(0b1010_1010, 0b0101_0101, 3);
    assert!(idx <= 3);
}

#[test]
fn cover_no_mbc() {
    let mut mbc = NoMbc::new(vec![0xAA; 0x8000]);
    // ROM read
    assert_eq!(mbc.read(0x0000).unwrap(), 0xAA);
    // RAM disabled
    assert!(matches!(mbc.read(0xA000), Err(MbcError::RamDisabled)));
    // Enable RAM
    mbc.write(0x0000, 0x0A).unwrap();
    // RAM write and read
    assert!(mbc.write(0xA000, 0x55).is_ok());
    assert_eq!(mbc.read(0xA000).unwrap(), 0x55);
    // Out of range
    assert!(mbc.read(0xC000).is_err());
    assert!(mbc.write(0xC000, 0xFF).is_err());
    // Save/load RAM
    let ram = mbc.save_ram();
    assert!(mbc.load_ram(ram).is_ok());
    // Invalid RAM size
    assert!(mbc.load_ram(vec![0; 1]).is_err());
    // Trait methods
    assert_eq!(mbc.rom_bank(), 0);
    assert_eq!(mbc.ram_bank(), 0);
    assert!(mbc.is_ram_enabled());
}
