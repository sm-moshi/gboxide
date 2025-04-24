//! Unified tests for all MBC controllers (Mbc1, Mbc2, Mbc3, Mbc5)
//! Covers ROM/RAM banking, enable/disable, error handling, RTC stub, and save/load RAM.

use anyhow::Result;
use core_lib::mmu::mbc::{Mbc, Mbc1, Mbc2, Mbc3, Mbc5};
use pretty_assertions::assert_eq as pretty_assert_eq;
use proptest::prelude::*;

fn dummy_rom(size: usize) -> Vec<u8> {
    // Safe: (i & 0xFF) always fits in u8 for test data
    (0..size)
        .map(|i| u8::try_from(i & 0xFF).unwrap_or(0))
        .collect()
}

#[test]
fn mbc1_rom_banking_basic() -> Result<()> {
    let rom = dummy_rom(0x80_0000); // 32KB
    let mut mbc = Mbc1::new(rom);
    // Bank 0
    pretty_assert_eq!(mbc.read(0x0000)?, 0);
    // Switch to bank 1
    mbc.write(0x2000, 1)?;
    pretty_assert_eq!(mbc.rom_bank(), 1);
    // Bank 1, address 0x4000
    pretty_assert_eq!(mbc.read(0x4000)?, 0u8);
    Ok(())
}

#[test]
fn mbc2_ram_enable_disable() -> Result<()> {
    let rom = dummy_rom(0x40000); // 256KB
    let mut mbc = Mbc2::new(rom);
    // RAM disabled by default
    assert!(mbc.write(0xA000, 0x0F).is_err());
    // Enable RAM
    mbc.write(0x0000, 0x0A)?;
    mbc.write(0xA000, 0x05)?;
    pretty_assert_eq!(mbc.read(0xA000)? & 0x0F, 0x05);
    // Disable RAM
    mbc.write(0x0000, 0x00)?;
    assert!(mbc.write(0xA000, 0x0F).is_err());
    Ok(())
}

#[test]
fn mbc3_rtc_stub_access() -> Result<()> {
    let rom = dummy_rom(0x20_0000); // 2MB
    let mut mbc = Mbc3::new(rom);
    // Enable RAM/RTC
    mbc.write(0x0000, 0x0A)?;
    // Select RTC register 0x08 (seconds)
    mbc.write(0x4000, 0x08)?;
    mbc.write(0xA000, 0x12)?;
    // Latch after write
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    pretty_assert_eq!(mbc.read(0xA000)?, 0x12);
    Ok(())
}

#[test]
fn mbc5_rom_ram_banking() -> Result<()> {
    let rom = dummy_rom(0x80_0000); // 8MB
    let mut mbc = Mbc5::new(rom);
    // Switch to ROM bank 257
    mbc.write(0x2000, 0x01)?;
    mbc.write(0x3000, 0x01)?;
    pretty_assert_eq!(mbc.rom_bank(), 257);
    // Enable RAM, select RAM bank 2
    mbc.write(0x0000, 0x0A)?;
    mbc.write(0x4000, 0x02)?;
    mbc.write(0xA000, 0x55)?;
    pretty_assert_eq!(mbc.read(0xA000)?, 0x55);
    Ok(())
}

#[test]
fn mbc3_rtc_registers_read_write() -> Result<()> {
    let rom = dummy_rom(0x20_0000); // 2MB
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A)?; // Enable RAM/RTC
                              // Write to all RTC registers
    mbc.write(0x4000, 0x08)?;
    mbc.write(0xA000, 59)?; // seconds
    mbc.write(0x4000, 0x09)?;
    mbc.write(0xA000, 58)?; // minutes
    mbc.write(0x4000, 0x0A)?;
    mbc.write(0xA000, 23)?; // hours
    mbc.write(0x4000, 0x0B)?;
    mbc.write(0xA000, 0xFF)?; // day low
    mbc.write(0x4000, 0x0C)?;
    mbc.write(0xA000, 0x01)?; // day high (bit 0)
                              // Latch after writes
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    // Read back
    mbc.write(0x4000, 0x08)?;
    pretty_assert_eq!(mbc.read(0xA000)?, 59);
    mbc.write(0x4000, 0x09)?;
    pretty_assert_eq!(mbc.read(0xA000)?, 58);
    mbc.write(0x4000, 0x0A)?;
    pretty_assert_eq!(mbc.read(0xA000)?, 23);
    mbc.write(0x4000, 0x0B)?;
    pretty_assert_eq!(mbc.read(0xA000)?, 0xFF);
    mbc.write(0x4000, 0x0C)?;
    pretty_assert_eq!(mbc.read(0xA000)? & 0x01, 0x01); // day high bit 0
    Ok(())
}

#[test]
fn mbc3_rtc_halt_and_carry() -> Result<()> {
    let rom = dummy_rom(0x20_0000);
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A)?;
    // Set halt
    mbc.write(0x4000, 0x0C)?;
    mbc.write(0xA000, 0x40)?;
    // Latch after write
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    mbc.write(0x4000, 0x0C)?;
    pretty_assert_eq!(mbc.read(0xA000)? & 0x40, 0x40);
    // Set carry
    mbc.write(0x4000, 0x0C)?;
    mbc.write(0xA000, 0x80)?;
    // Latch after write
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    mbc.write(0x4000, 0x0C)?;
    pretty_assert_eq!(mbc.read(0xA000)? & 0x80, 0x80);
    Ok(())
}

#[test]
fn mbc3_rtc_latch_and_tick() -> Result<()> {
    use std::thread::sleep;
    use std::time::Duration;
    let rom = dummy_rom(0x20_0000);
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A)?;
    // Latch initial
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    mbc.write(0x4000, 0x08)?;
    let initial = mbc.read(0xA000)?;
    // Wait 2 seconds
    sleep(Duration::from_secs(2));
    // Latch again
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    mbc.write(0x4000, 0x08)?;
    let after = mbc.read(0xA000)?;
    assert!(
        (after > initial) && (after <= initial + 2),
        "RTC did not tick as expected"
    );
    Ok(())
}

#[test]
fn mbc3_rtc_persistence() -> Result<()> {
    use std::thread::sleep;
    use std::time::Duration;
    let rom = dummy_rom(0x20_0000);
    let mut mbc = Mbc3::new(rom);
    mbc.write(0x0000, 0x0A)?;
    // Set RTC to 59 seconds
    mbc.write(0x4000, 0x08)?;
    mbc.write(0xA000, 59)?;
    // Latch and save
    mbc.write(0x6000, 0x00)?;
    mbc.write(0x6000, 0x01)?;
    let saved = mbc.save_ram();
    // Wait 3 seconds (increased from 2 for robustness)
    sleep(Duration::from_secs(3));
    // Load into new Mbc3
    let mut mbc2 = Mbc3::new(dummy_rom(0x20_0000));
    mbc2.load_ram(saved)?;
    mbc2.write(0x0000, 0x0A)?; // Enable RAM/RTC after load
                               // Latch and read
    mbc2.write(0x6000, 0x00)?;
    mbc2.write(0x6000, 0x01)?;
    mbc2.write(0x4000, 0x08)?;
    let after = mbc2.read(0xA000)?;
    // Should have advanced by at least 1 second
    if !((0..=3).contains(&after) || after >= 60) {
        eprintln!("[mbc3_rtc_persistence] RTC seconds after reload: {after}");
    }
    assert!(
        (0..=3).contains(&after) || after >= 60,
        "RTC did not wrap as expected"
    );
    Ok(())
}

proptest! {
    #[test]
    fn mbc1_rom_banking_prop(bank in 1u8..=0x1F) {
        let rom = dummy_rom(0x80000); // 512KB
        let mut mbc = Mbc1::new(rom);
        let res = mbc.write(0x2000, bank);
        prop_assert!(res.is_ok());
        prop_assert_eq!(mbc.rom_bank(), if bank == 0 { 1 } else { bank as usize });
    }

    #[test]
    fn mbc2_ram_lower_nibble_only(val in 0u8..=0xFF) {
        let rom = dummy_rom(0x40000);
        let mut mbc = Mbc2::new(rom);
        let res1 = mbc.write(0x0000, 0x0A);
        let res2 = mbc.write(0xA000, val);
        let stored = mbc.read(0xA000);
        prop_assert!(res1.is_ok());
        prop_assert!(res2.is_ok());
        match stored {
            Ok(stored_val) => prop_assert_eq!(stored_val & 0x0F, val & 0x0F),
            Err(_) => prop_assert!(false, "read(0xA000) failed"),
        }
    }

    #[test]
    fn mbc5_ram_banking_prop(bank in 0u8..=0x0F, val in 0u8..=0xFF) {
        let rom = dummy_rom(0x80_0000);
        let mut mbc = Mbc5::new(rom);
        let res1 = mbc.write(0x0000, 0x0A);
        let res2 = mbc.write(0x4000, bank);
        let res3 = mbc.write(0xA000, val);
        let read = mbc.read(0xA000);
        prop_assert!(res1.is_ok());
        prop_assert!(res2.is_ok());
        prop_assert!(res3.is_ok());
        match read {
            Ok(read_val) => prop_assert_eq!(read_val, val),
            Err(_) => prop_assert!(false, "read(0xA000) failed"),
        }
    }
}

#[test]
fn mbc1_save_load_ram() -> Result<()> {
    let rom = dummy_rom(0x80_0000);
    let mut mbc = Mbc1::new(rom);
    mbc.write(0x0000, 0x0A)?;
    mbc.write(0xA000, 0x42)?;
    let saved = mbc.save_ram();
    let mut mbc2 = Mbc1::new(dummy_rom(0x80000));
    mbc2.load_ram(saved)?;
    mbc2.write(0x0000, 0x0A)?; // Enable RAM before reading
    pretty_assert_eq!(mbc2.read(0xA000)?, 0x42);
    Ok(())
}
