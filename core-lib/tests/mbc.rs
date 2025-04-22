//! Unified tests for all MBC controllers (Mbc1, Mbc2, Mbc3, Mbc5)
//! Covers ROM/RAM banking, enable/disable, error handling, RTC stub, and save/load RAM.

use core_lib::mmu::mbc::{Mbc, Mbc1, Mbc2, Mbc3, Mbc5};
use pretty_assertions::assert_eq as pretty_assert_eq;
use proptest::prelude::*;

fn dummy_rom(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i & 0xFF) as u8).collect()
}

#[test]
fn mbc1_rom_banking_basic() {
    let rom = dummy_rom(0x8000); // 32KB
    let mut mbc = Mbc1::new(rom.clone());
    // Bank 0
    pretty_assert_eq!(mbc.read(0x0000).unwrap(), 0);
    // Switch to bank 1
    mbc.write(0x2000, 1).unwrap();
    pretty_assert_eq!(mbc.rom_bank(), 1);
    // Bank 1, address 0x4000
    pretty_assert_eq!(mbc.read(0x4000).unwrap(), 0u8);
}

#[test]
fn mbc2_ram_enable_disable() {
    let rom = dummy_rom(0x40000); // 256KB
    let mut mbc = Mbc2::new(rom);
    // RAM disabled by default
    assert!(mbc.write(0xA000, 0x0F).is_err());
    // Enable RAM
    mbc.write(0x0000, 0x0A).unwrap();
    mbc.write(0xA000, 0x05).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap() & 0x0F, 0x05);
    // Disable RAM
    mbc.write(0x0000, 0x00).unwrap();
    assert!(mbc.write(0xA000, 0x0F).is_err());
}

#[test]
fn mbc3_rtc_stub_access() {
    let rom = dummy_rom(0x200000); // 2MB
    let mut mbc = Mbc3::new(rom);
    // Enable RAM/RTC
    mbc.write(0x0000, 0x0A).unwrap();
    // Select RTC register 0x08 (seconds)
    mbc.write(0x4000, 0x08).unwrap();
    mbc.write(0xA000, 0x12).unwrap();
    // Latch after write
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap(), 0x12);
}

#[test]
fn mbc5_rom_ram_banking() {
    let rom = dummy_rom(0x800000); // 8MB
    let mut mbc = Mbc5::new(rom);
    // Switch to ROM bank 257
    mbc.write(0x2000, 0x01).unwrap();
    mbc.write(0x3000, 0x01).unwrap();
    pretty_assert_eq!(mbc.rom_bank(), 257);
    // Enable RAM, select RAM bank 2
    mbc.write(0x0000, 0x0A).unwrap();
    mbc.write(0x4000, 0x02).unwrap();
    mbc.write(0xA000, 0x55).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap(), 0x55);
}

#[test]
fn mbc3_rtc_registers_read_write() {
    let rom = dummy_rom(0x200000); // 2MB
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A).unwrap(); // Enable RAM/RTC
                                      // Write to all RTC registers
    mbc.write(0x4000, 0x08).unwrap();
    mbc.write(0xA000, 59).unwrap(); // seconds
    mbc.write(0x4000, 0x09).unwrap();
    mbc.write(0xA000, 58).unwrap(); // minutes
    mbc.write(0x4000, 0x0A).unwrap();
    mbc.write(0xA000, 23).unwrap(); // hours
    mbc.write(0x4000, 0x0B).unwrap();
    mbc.write(0xA000, 0xFF).unwrap(); // day low
    mbc.write(0x4000, 0x0C).unwrap();
    mbc.write(0xA000, 0x01).unwrap(); // day high (bit 0)
                                      // Latch after writes
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    // Read back
    mbc.write(0x4000, 0x08).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap(), 59);
    mbc.write(0x4000, 0x09).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap(), 58);
    mbc.write(0x4000, 0x0A).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap(), 23);
    mbc.write(0x4000, 0x0B).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap(), 0xFF);
    mbc.write(0x4000, 0x0C).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap() & 0x01, 0x01); // day high bit 0
}

#[test]
fn mbc3_rtc_halt_and_carry() {
    let rom = dummy_rom(0x200000);
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A).unwrap();
    // Set halt
    mbc.write(0x4000, 0x0C).unwrap();
    mbc.write(0xA000, 0x40).unwrap();
    // Latch after write
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    mbc.write(0x4000, 0x0C).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap() & 0x40, 0x40);
    // Set carry
    mbc.write(0x4000, 0x0C).unwrap();
    mbc.write(0xA000, 0x80).unwrap();
    // Latch after write
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    mbc.write(0x4000, 0x0C).unwrap();
    pretty_assert_eq!(mbc.read(0xA000).unwrap() & 0x80, 0x80);
}

#[test]
fn mbc3_rtc_latch_and_tick() {
    use std::thread::sleep;
    use std::time::Duration;
    let rom = dummy_rom(0x200000);
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A).unwrap();
    // Latch initial
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    mbc.write(0x4000, 0x08).unwrap();
    let initial = mbc.read(0xA000).unwrap();
    // Wait 2 seconds
    sleep(Duration::from_secs(2));
    // Latch again
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    mbc.write(0x4000, 0x08).unwrap();
    let after = mbc.read(0xA000).unwrap();
    assert!(after >= initial + 1 && after <= initial + 2);
}

#[test]
fn mbc3_rtc_persistence() {
    use std::thread::sleep;
    use std::time::Duration;
    let rom = dummy_rom(0x200000);
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A).unwrap();
    // Set RTC to 59 seconds
    mbc.write(0x4000, 0x08).unwrap();
    mbc.write(0xA000, 59).unwrap();
    // Latch and save
    mbc.write(0x6000, 0x00).unwrap();
    mbc.write(0x6000, 0x01).unwrap();
    let saved = mbc.save_ram();
    // Wait 2 seconds
    sleep(Duration::from_secs(2));
    // Load into new MBC3
    let mut mbc2 = Mbc3::new(dummy_rom(0x200000));
    mbc2.load_ram(saved).unwrap();
    mbc2.write(0x0000, 0x0A).unwrap(); // Enable RAM/RTC after load
                                       // Latch and read
    mbc2.write(0x6000, 0x00).unwrap();
    mbc2.write(0x6000, 0x01).unwrap();
    mbc2.write(0x4000, 0x08).unwrap();
    let after = mbc2.read(0xA000).unwrap();
    // Should have advanced by at least 1 second
    assert!(after >= 0 && after <= 1 || after >= 60); // handle wrap
}

proptest! {
    #[test]
    fn mbc1_rom_banking_prop(bank in 1u8..=0x1F) {
        let rom = dummy_rom(0x80000); // 512KB
        let mut mbc = Mbc1::new(rom.clone());
        mbc.write(0x2000, bank).unwrap();
        prop_assert_eq!(mbc.rom_bank(), if bank == 0 { 1 } else { bank as usize });
    }

    #[test]
    fn mbc2_ram_lower_nibble_only(val in 0u8..=0xFF) {
        let rom = dummy_rom(0x40000);
        let mut mbc = Mbc2::new(rom);
        mbc.write(0x0000, 0x0A).unwrap();
        mbc.write(0xA000, val).unwrap();
        let stored = mbc.read(0xA000).unwrap();
        prop_assert_eq!(stored & 0x0F, val & 0x0F);
    }

    #[test]
    fn mbc5_ram_banking_prop(bank in 0u8..=0x0F, val in 0u8..=0xFF) {
        let rom = dummy_rom(0x800000);
        let mut mbc = Mbc5::new(rom);
        mbc.write(0x0000, 0x0A).unwrap();
        mbc.write(0x4000, bank).unwrap();
        mbc.write(0xA000, val).unwrap();
        let read = mbc.read(0xA000).unwrap();
        prop_assert_eq!(read, val);
    }
}

#[test]
fn mbc1_save_load_ram() {
    let rom = dummy_rom(0x80000);
    let mut mbc = Mbc1::new(rom);
    mbc.write(0x0000, 0x0A).unwrap();
    mbc.write(0xA000, 0x42).unwrap();
    let saved = mbc.save_ram();
    let mut mbc2 = Mbc1::new(dummy_rom(0x80000));
    mbc2.load_ram(saved.clone()).unwrap();
    mbc2.write(0x0000, 0x0A).unwrap(); // Enable RAM before reading
    pretty_assert_eq!(mbc2.read(0xA000).unwrap(), 0x42);
}
