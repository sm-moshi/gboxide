// core/src/mmu/tests.rs

use crate::mmu::MMU;

fn create_mmu_with_rom_data(data: u8) -> MMU {
    let mut mmu = MMU::new();
    mmu.load_rom(vec![data; 0x8000]);
    mmu
}

#[test]
fn test_rom_fixed_read() {
    let mmu = create_mmu_with_rom_data(0x42);
    assert_eq!(mmu.read(0x0000), 0x42);
    assert_eq!(mmu.read(0x3FFF), 0x42);
}

#[test]
fn test_rom_bank_read_default() {
    let mmu = create_mmu_with_rom_data(0x55);
    assert_eq!(mmu.read(0x4000), 0x55);
    assert_eq!(mmu.read(0x7FFF), 0x55);
}

#[test]
fn test_rom_bank_switch() {
    let mut mmu = create_mmu_with_rom_data(0x00);
    mmu.rom[0x4000 * 2 + 0x100] = 0xAB; // ROM bank 2 offset

    mmu.write(0x2000, 0x02); // Set ROM bank to 2
    assert_eq!(mmu.read(0x4100), 0xAB);
}

#[test]
fn test_ram_enable_disable() {
    let mut mmu = create_mmu_with_rom_data(0x00);
    mmu.write(0x0000, 0x0A); // Enable RAM
    mmu.write(0xA000, 0x99);
    assert_eq!(mmu.read(0xA000), 0x99);

    mmu.write(0x0000, 0x00); // Disable RAM
    assert_eq!(mmu.read(0xA000), 0xFF); // RAM is disabled
}

#[test]
fn test_ram_bank_switch() {
    let mut mmu = create_mmu_with_rom_data(0x00);
    mmu.write(0x0000, 0x0A); // Enable RAM

    mmu.write(0x4000, 0x01); // Set RAM bank to 1
    mmu.write(0xA000, 0x11);
    assert_eq!(mmu.read(0xA000), 0x11);

    mmu.write(0x4000, 0x02); // Switch to RAM bank 2
    assert_ne!(mmu.read(0xA000), 0x11); // Should be different
}