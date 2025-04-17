use super::*;
use crate::bus::MemoryBus;

fn load_mock_rom(size: usize) -> Vec<u8> {
    let mut rom = vec![0; size];
    for i in 0..size {
        rom[i] = (i & 0xFF) as u8;
    }
    rom
}

#[test]
fn test_rom_read_fixed_bank() {
    let rom = load_mock_rom(0x8000); // 32KB ROM
    let mut mmu = MMU::new(rom);

    assert_eq!(mmu.read(0x0000), 0x00); // First byte of ROM
    assert_eq!(mmu.read(0x3FFF), 0xFF); // Last byte of bank 0
}

#[test]
fn test_rom_bank_switching() {
    let rom = load_mock_rom(0x8000); // 32KB ROM
    let mut mmu = MMU::new(rom);

    mmu.write(0x2000, 0x02); // Switch to ROM bank 2
    assert_eq!(mmu.rom_bank, 2);

    let addr = 0x4000; // Banked ROM region
    let expected = rom[0x4000 * 2]; // 2nd bank starts at 0x8000 in full rom
    assert_eq!(mmu.read(addr), expected);
}

#[test]
fn test_ram_enable_disable() {
    let rom = load_mock_rom(0x8000);
    let mut mmu = MMU::new(rom);

    mmu.write(0x0000, 0x0A); // Enable RAM
    assert!(mmu.ram_enabled);

    mmu.write(0xA000, 0x55); // Write to ext RAM
    assert_eq!(mmu.read(0xA000), 0x55);

    mmu.write(0x0000, 0x00); // Disable RAM
    assert!(!mmu.ram_enabled);

    assert_eq!(mmu.read(0xA000), 0xFF); // Should return open bus value
}

#[test]
fn test_ram_bank_switching() {
    let rom = load_mock_rom(0x8000);
    let mut mmu = MMU::new(rom);
    mmu.write(0x0000, 0x0A); // Enable RAM

    mmu.write(0x6000, 0x01); // Select RAM banking mode
    mmu.write(0x4000, 0x01); // Switch to RAM bank 1
    mmu.write(0xA000, 0x77);
    assert_eq!(mmu.ram_banks[1][0], 0x77);

    mmu.write(0x4000, 0x00); // Switch to RAM bank 0
    mmu.write(0xA000, 0x99);
    assert_eq!(mmu.ram_banks[0][0], 0x99);
}

#[test]
fn test_bank_number_wrapping() {
    let rom = load_mock_rom(0x8000);
    let mut mmu = MMU::new(rom);

    mmu.write(0x2000, 0x00); // Should wrap to 1
    assert_eq!(mmu.rom_bank, 1);

    mmu.write(0x2000, 0x1F); // Valid
    assert_eq!(mmu.rom_bank, 0x1F);

    mmu.write(0x2000, 0x20); // Invalid in MBC1, should mask lower bits
    assert_eq!(mmu.rom_bank, 0x00); // Wrap to 1 internally during access
}
