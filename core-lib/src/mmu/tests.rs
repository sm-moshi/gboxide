/// core-lib/src/mmu/tests.rs
#[cfg(test)]
// Acceptable in tests - we want tests to fail loudly
// #[allow(clippy::cast_possible_truncation)] // Intentional truncation in test data
use super::MMU;
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

/// Tests reading from the fixed ROM bank (0x0000-0x3FFF)
#[test]
fn test_rom_fixed_read() {
    let mmu = create_test_mmu();
    assert_eq!(mmu.read(0x0000), 0x00);
    assert_eq!(mmu.read(0x3FFF), 0x00);
}

/// Tests reading from the switchable ROM bank (0x4000-0x7FFF) in its default state
#[test]
fn test_rom_bank_read_default() {
    let mmu = create_test_mmu();
    assert_eq!(mmu.read(0x4000), 0x00);
    assert_eq!(mmu.read(0x7FFF), 0x00);
}

/// Tests ROM bank switching functionality with MBC1
/// Verifies that data can be read from different ROM banks after switching
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

/// Tests RAM enable/disable functionality
/// Verifies that RAM is disabled by default, can be enabled with 0x0A,
/// and disabled with 0x00
#[test]
fn test_ram_enable_disable() {
    let mut mmu = create_test_mmu();

    // RAM should be disabled by default
    assert_eq!(mmu.read(0xA000), 0xFF);

    // Enable RAM
    let _ = mmu.write(0x0000, 0x0A);
    let _ = mmu.write(0xA000, 0x42);
    assert_eq!(mmu.read(0xA000), 0x42);

    // Disable RAM
    let _ = mmu.write(0x0000, 0x00);
    assert_eq!(mmu.read(0xA000), 0xFF);
}

/// Tests RAM bank switching functionality in mode 1
/// Verifies that data written to different RAM banks persists independently
#[test]
fn test_ram_bank_switch() {
    let mut mmu = create_test_mmu();

    // Enable RAM
    let _ = mmu.write(0x0000, 0x0A);

    // Write to RAM bank 0
    let _ = mmu.write(0xA000, 0x42);
    assert_eq!(mmu.read(0xA000), 0x42);

    // Switch to RAM bank 1 (requires setting mode 1)
    let _ = mmu.write(0x6000, 0x01); // Set mode 1
    let _ = mmu.write(0x4000, 0x01); // Set RAM bank 1
    let _ = mmu.write(0xA000, 0x24);
    assert_eq!(mmu.read(0xA000), 0x24);

    // Switch back to bank 0 and verify data
    let _ = mmu.write(0x4000, 0x00);
    assert_eq!(mmu.read(0xA000), 0x42);
}

/// Tests basic VRAM read/write functionality (0x8000-0x9FFF)
#[test]
fn test_vram_access() {
    let mut mmu = create_test_mmu();
    let _ = mmu.write(0x8000, 0x42);
    assert_eq!(mmu.read(0x8000), 0x42);
}

/// Tests basic WRAM read/write functionality (0xC000-0xDFFF)
#[test]
fn test_wram_access() {
    let mut mmu = create_test_mmu();
    let _ = mmu.write(0xC000, 0x42);
    assert_eq!(mmu.read(0xC000), 0x42);
}

/// Tests that Echo RAM (0xE000-0xFDFF) mirrors WRAM (0xC000-0xDDFF)
#[test]
fn test_echo_ram() {
    let mut mmu = create_test_mmu();
    let _ = mmu.write(0xC000, 0x42);
    assert_eq!(mmu.read(0xE000), 0x42);
}

/// Tests basic OAM read/write functionality (0xFE00-0xFE9F)
#[test]
fn test_oam_access() {
    let mut mmu = create_test_mmu();
    let _ = mmu.write(0xFE00, 0x42);
    assert_eq!(mmu.read(0xFE00), 0x42);
}

/// Tests basic HRAM read/write functionality (0xFF80-0xFFFE)
#[test]
fn test_hram_access() {
    let mut mmu = create_test_mmu();
    let _ = mmu.write(0xFF80, 0x42);
    assert_eq!(mmu.read(0xFF80), 0x42);
}

/// Tests saving and loading of RAM state
/// Verifies that RAM contents can be saved and restored correctly
#[test]
fn test_save_load_ram() {
    let mut mmu = create_test_mmu();

    // Enable RAM and write some data
    let _ = mmu.write(0x0000, 0x0A);
    let _ = mmu.write(0xA000, 0x42);

    // Save RAM state
    let ram_state = mmu.save_ram();

    // Change RAM contents
    let _ = mmu.write(0xA000, 0x24);
    assert_eq!(mmu.read(0xA000), 0x24);

    // Load RAM state and verify
    mmu.load_ram(ram_state).unwrap();
    assert_eq!(mmu.read(0xA000), 0x42);
}

/// Tests that the MMU correctly rejects invalid cartridge data
/// (ROM size too small)
#[test]
fn test_invalid_cartridge() {
    let result = MMU::new(vec![0; 0x100]); // Too small ROM
    assert!(result.is_err());
}

/// Tests behavior of ROM-only cartridges (type 0x00)
/// Verifies that writes to ROM are ignored
#[test]
fn test_rom_only_cartridge() {
    let mut rom = vec![0; 0x8000];
    rom[0x147] = 0x00; // ROM ONLY
    let mut mmu = MMU::new(rom).unwrap();

    // Try writing to ROM (should be ignored)
    let _ = mmu.write(0x0000, 0x42);
    assert_eq!(mmu.read(0x0000), 0x00);
}

/// Tests MBC1 cartridge with RAM and battery functionality
/// Verifies that RAM can be enabled and state can be saved
#[test]
fn test_mbc1_ram_battery() {
    let mut rom = vec![0; 0x8000];
    rom[0x147] = 0x03; // MBC1+RAM+BATTERY
    let mut mmu = MMU::new(rom).unwrap();

    // Enable RAM and write
    let _ = mmu.write(0x0000, 0x0A);
    let _ = mmu.write(0xA000, 0x42);

    // Save state should work
    let ram_state = mmu.save_ram();
    assert!(!ram_state.is_empty());
}

/// Tests basic read/write functionality across various memory regions
/// Verifies that writes to ROM are ignored and other regions work correctly
#[test]
fn test_mmu_write_read() {
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom).unwrap();

    // Test writing and reading from various memory regions
    let _ = mmu.write(0x0000, 0x42); // ROM bank 0 (write should be ignored)
    assert_eq!(mmu.read(0x0000), 0x00);

    let _ = mmu.write(0x8000, 0x42); // VRAM
    assert_eq!(mmu.read(0x8000), 0x42);

    let _ = mmu.write(0xC000, 0x42); // WRAM
    assert_eq!(mmu.read(0xC000), 0x42);

    let _ = mmu.write(0xFF80, 0x42); // HRAM
    assert_eq!(mmu.read(0xFF80), 0x42);
}

/// Tests that writes to ROM banks are ignored
/// Verifies ROM write protection for both fixed and switchable banks
#[test]
fn test_read_write_rom() {
    let mut mmu = create_test_mmu();

    // Test writing to ROM (should be ignored)
    let _ = mmu.write(0x0000, 0x42);
    pretty_assertions::assert_eq!(mmu.read(0x0000), 0x00);

    // Test writing to switchable ROM bank (should be ignored)
    let _ = mmu.write(0x4000, 0x42);
    pretty_assertions::assert_eq!(mmu.read(0x4000), 0x00);
}

/// Tests DMA transfers from different memory regions (ROM, VRAM, WRAM)
/// Verifies that DMA correctly copies data from various source regions to OAM
/// Each test writes unique patterns to verify correct transfer
#[test]
fn test_dma_from_various_sources() {
    let mut mmu = create_test_mmu();

    // Test DMA from ROM
    let _ = mmu.write(0xFF46, 0x00); // Source: 0x0000
    mmu.step(160);

    // Test DMA from VRAM
    for i in 0..0xA0 {
        let _ = mmu.write(0x8000 + i, u8::try_from(i + 1).unwrap());
    }
    let _ = mmu.write(0xFF46, 0x80); // Source: 0x8000
    mmu.step(160);

    for i in 0..0xA0 {
        assert_eq!(mmu.read(0xFE00 + i), u8::try_from(i + 1).unwrap());
    }

    // Test DMA from WRAM
    for i in 0..0xA0 {
        let _ = mmu.write(0xC000 + i, u8::try_from(i + 2).unwrap());
    }
    let _ = mmu.write(0xFF46, 0xC0); // Source: 0xC000
    mmu.step(160);

    for i in 0..0xA0 {
        assert_eq!(mmu.read(0xFE00 + i), u8::try_from(i + 2).unwrap());
    }
}

/// Tests reading from the DMA register (0xFF46)
/// Verifies that reading the DMA register returns the last written value
/// This is important for software that needs to track DMA state
#[test]
fn test_dma_register_read() {
    let mut mmu = create_test_mmu();

    let _ = mmu.write(0xFF46, 0x42); // Start DMA from 0x4200
    assert_eq!(mmu.read(0xFF46), 0x42);
}
