/// core-lib/src/mmu/tests.rs
use super::*;
use pretty_assertions::assert_eq;

/// Creates a new MMU instance with a dummy ROM for testing
fn create_test_mmu() -> MMU {
    let mut dummy_rom = vec![0; 0x8000]; // 32KB ROM
                                         // Set cartridge type to MBC1
    dummy_rom[0x147] = 0x01;
    // Set ROM size to 32KB
    dummy_rom[0x148] = 0x00;
    // Set RAM size to 8KB
    dummy_rom[0x149] = 0x02;
    MMU::new(dummy_rom).unwrap()
}

#[test]
fn test_rom_fixed_read() {
    let mmu = create_test_mmu();
    assert_eq!(mmu.read(0x0000), 0x00);
    assert_eq!(mmu.read(0x3FFF), 0x00);
}

#[test]
fn test_rom_bank_read_default() {
    let mmu = create_test_mmu();
    assert_eq!(mmu.read(0x4000), 0x00);
    assert_eq!(mmu.read(0x7FFF), 0x00);
}

#[test]
fn test_rom_bank_switch() {
    let mut rom = vec![0; 0x20000]; // 128KB ROM
    rom[0x147] = 0x01; // MBC1
    rom[0x148] = 0x02; // 128KB ROM
    rom[0x4000 * 2 + 0x100] = 0xAB; // Data in ROM bank 2

    let mut mmu = MMU::new(rom).unwrap();
    mmu.write(0x2000, 0x02); // Set ROM bank to 2
    assert_eq!(mmu.read(0x4100), 0xAB);
}

#[test]
fn test_ram_enable_disable() {
    let mut mmu = create_test_mmu();

    // RAM should be disabled by default
    assert_eq!(mmu.read(0xA000), 0xFF);

    // Enable RAM
    mmu.write(0x0000, 0x0A);
    mmu.write(0xA000, 0x42);
    assert_eq!(mmu.read(0xA000), 0x42);

    // Disable RAM
    mmu.write(0x0000, 0x00);
    assert_eq!(mmu.read(0xA000), 0xFF);
}

#[test]
fn test_ram_bank_switch() {
    let mut mmu = create_test_mmu();

    // Enable RAM
    mmu.write(0x0000, 0x0A);

    // Write to RAM bank 0
    mmu.write(0xA000, 0x42);
    assert_eq!(mmu.read(0xA000), 0x42);

    // Switch to RAM bank 1 (requires setting mode 1)
    mmu.write(0x6000, 0x01); // Set mode 1
    mmu.write(0x4000, 0x01); // Set RAM bank 1
    mmu.write(0xA000, 0x24);
    assert_eq!(mmu.read(0xA000), 0x24);

    // Switch back to bank 0 and verify data
    mmu.write(0x4000, 0x00);
    assert_eq!(mmu.read(0xA000), 0x42);
}

#[test]
fn test_vram_access() {
    let mut mmu = create_test_mmu();
    mmu.write(0x8000, 0x42);
    assert_eq!(mmu.read(0x8000), 0x42);
}

#[test]
fn test_wram_access() {
    let mut mmu = create_test_mmu();
    mmu.write(0xC000, 0x42);
    assert_eq!(mmu.read(0xC000), 0x42);
}

#[test]
fn test_echo_ram() {
    let mut mmu = create_test_mmu();
    mmu.write(0xC000, 0x42);
    assert_eq!(mmu.read(0xE000), 0x42);
}

#[test]
fn test_oam_access() {
    let mut mmu = create_test_mmu();
    mmu.write(0xFE00, 0x42);
    assert_eq!(mmu.read(0xFE00), 0x42);
}

#[test]
fn test_hram_access() {
    let mut mmu = create_test_mmu();
    mmu.write(0xFF80, 0x42);
    assert_eq!(mmu.read(0xFF80), 0x42);
}

#[test]
fn test_save_load_ram() {
    let mut mmu = create_test_mmu();

    // Enable RAM and write some data
    mmu.write(0x0000, 0x0A);
    mmu.write(0xA000, 0x42);

    // Save RAM state
    let ram_state = mmu.save_ram();

    // Change RAM contents
    mmu.write(0xA000, 0x24);
    assert_eq!(mmu.read(0xA000), 0x24);

    // Load RAM state and verify
    mmu.load_ram(ram_state).unwrap();
    assert_eq!(mmu.read(0xA000), 0x42);
}

#[test]
fn test_invalid_cartridge() {
    let result = MMU::new(vec![0; 0x100]); // Too small ROM
    assert!(result.is_err());
}

#[test]
fn test_rom_only_cartridge() {
    let mut rom = vec![0; 0x8000];
    rom[0x147] = 0x00; // ROM ONLY
    let mut mmu = MMU::new(rom).unwrap();

    // Try writing to ROM (should be ignored)
    mmu.write(0x0000, 0x42);
    assert_eq!(mmu.read(0x0000), 0x00);
}

#[test]
fn test_mbc1_ram_battery() {
    let mut rom = vec![0; 0x8000];
    rom[0x147] = 0x03; // MBC1+RAM+BATTERY
    let mut mmu = MMU::new(rom).unwrap();

    // Enable RAM and write
    mmu.write(0x0000, 0x0A);
    mmu.write(0xA000, 0x42);

    // Save state should work
    let ram_state = mmu.save_ram();
    assert!(!ram_state.is_empty());
}

#[test]
fn test_mmu_write_read() {
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom).unwrap();

    // Test writing and reading from various memory regions
    mmu.write(0x0000, 0x42); // ROM bank 0 (write should be ignored)
    assert_eq!(mmu.read(0x0000), 0x00);

    mmu.write(0x8000, 0x42); // VRAM
    assert_eq!(mmu.read(0x8000), 0x42);

    mmu.write(0xC000, 0x42); // WRAM
    assert_eq!(mmu.read(0xC000), 0x42);

    mmu.write(0xFF80, 0x42); // HRAM
    assert_eq!(mmu.read(0xFF80), 0x42);
}
