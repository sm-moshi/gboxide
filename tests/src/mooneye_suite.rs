// This file is part of Mooneye GB.
// Copyright (C) 2014-2020 Joonas Javanainen <joonas.javanainen@gmail.com>
//
// Mooneye GB is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Mooneye GB is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Mooneye GB.  If not, see <http://www.gnu.org/licenses/>.

//! Mooneye GB Test Suite Integration for gboxide
//!
//! This module adapts the Mooneye GB test suite for use with the gboxide emulator.
//! It provides a comprehensive set of hardware-accuracy tests for the Game Boy platform.
//!
//! # Purpose
//! - To validate emulator correctness against the Mooneye GB test ROMs.
//! - To serve as a development roadmap: unimplemented features are marked with `TODO`/`TO IMPLEMENT` comments.
//!
//! # Usage
//! - Test ROMs must be present in `assets/mooneye-test-suite`.
//! - Run with `cargo test --test mooneye_suite`.
//! - Failing or ignored tests indicate missing or incomplete emulator features.
//!
//! # Roadmap
//! - Hardware model selection (DMG, CGB, etc.) is not yet implemented. See `TODO` comments.
//! - Boot ROM support is not yet implemented. See `TODO` comments.
//! - This file should be updated as emulator support improves.
//!
//! # Attribution
//! - Original test logic and macros © Joonas Javanainen, adapted for gboxide.
//! - See license above.

use std::path::PathBuf;
use std::sync::Once;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use core_lib::cartridge::Cartridge;
use core_lib::cpu::CPU;
use core_lib::mmu::MMU;
use pretty_assertions::assert_eq;
use tracing::{debug, error, info};

// Ensure tracing is only initialised once per test run
static TRACING_INIT: Once = Once::new();

/// Initialise tracing for debug output
fn init_tracing() {
    TRACING_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt::try_init();
    });
}

// WARNING: Mooneye GB Test Suite is DEACTIVATED
// ------------------------------------------------
// Running these tests may freeze your system or cause a kernel panic due to incomplete emulator features.
// All tests are now marked as #[ignore].
//
// To re-enable, remove #[ignore] and ensure all emulator safety and feature requirements are met.
// ------------------------------------------------

macro_rules! testcases {
    (
        $name:ident($path:expr, all);
        $( $t_name:ident($t_path:expr, all); )*
    ) => {
        mod $name {
            use super::run_test;
            #[test]
            #[ignore]
            fn default() {
                super::init_tracing();
                if let Err(e) = run_test($path) {
                    panic!("Test failed: {}\n{:#}", $path, e);
                }
            }
        }
        $(
            mod $t_name {
                use super::run_test;
                #[test]
                #[ignore]
                fn default() {
                    super::init_tracing();
                    if let Err(e) = run_test($t_path) {
                        panic!("Test failed: {}\n{:#}", $t_path, e);
                    }
                }
            }
        )*
    };
}

// --- Mooneye GB Acceptance Test Suite ---
testcases! {
    add_sp_e_timing("acceptance/add_sp_e_timing", all);
    boot_div_dmg0("acceptance/boot_div-dmg0", all);
    boot_div_dmg_abc_mgb("acceptance/boot_div-dmgABCmgb", all);
    boot_div_s("acceptance/boot_div-S", all);
    boot_div2_s("acceptance/boot_div2-S", all);
    boot_hwio_dmg0("acceptance/boot_hwio-dmg0", all);
    boot_hwio_dmg_abc_mgb("acceptance/boot_hwio-dmgABCmgb", all);
    boot_hwio_s("acceptance/boot_hwio-S", all);
    boot_regs_dmg0("acceptance/boot_regs-dmg0", all);
    boot_regs_dmg_abc("acceptance/boot_regs-dmgABC", all);
    boot_regs_mgb("acceptance/boot_regs-mgb", all);
    boot_regs_sgb("acceptance/boot_regs-sgb", all);
    boot_regs_sgb2("acceptance/boot_regs-sgb2", all);
    call_cc_timing("acceptance/call_cc_timing", all);
    call_cc_timing2("acceptance/call_cc_timing2", all);
    call_timing("acceptance/call_timing", all);
    call_timing2("acceptance/call_timing2", all);
    di_timing_gs("acceptance/di_timing-GS", all);
    div_timing("acceptance/div_timing", all);
    ei_sequence("acceptance/ei_sequence", all);
    ei_timing("acceptance/ei_timing", all);
    halt_ime0_ei("acceptance/halt_ime0_ei", all);
    halt_ime0_nointr_timing("acceptance/halt_ime0_nointr_timing", all);
    halt_ime1_timing("acceptance/halt_ime1_timing", all);
    halt_ime1_timing2_gs("acceptance/halt_ime1_timing2-GS", all);
    if_ie_registers("acceptance/if_ie_registers", all);
    intr_timing("acceptance/intr_timing", all);
    jp_cc_timing("acceptance/jp_cc_timing", all);
    jp_timing("acceptance/jp_timing", all);
    ld_hl_sp_e_timing("acceptance/ld_hl_sp_e_timing", all);
    oam_dma_restart("acceptance/oam_dma_restart", all);
    oam_dma_start("acceptance/oam_dma_start", all);
    oam_dma_timing("acceptance/oam_dma_timing", all);
    pop_timing("acceptance/pop_timing", all);
    push_timing("acceptance/push_timing", all);
    rapid_di_ei("acceptance/rapid_di_ei", all);
    ret_timing("acceptance/ret_timing", all);
    reti_timing("acceptance/reti_timing", all);
    ret_cc_timing("acceptance/ret_cc_timing", all);
    reti_intr_timing("acceptance/reti_intr_timing", all);
    rst_timing("acceptance/rst_timing", all);
    // ... Add more as needed from the acceptance folder ...
}

// --- Placeholder for Common Tests ---
// testcases! {
//     common_test1("common/test1", all);
//     // ... Add more as needed ...
// }

// --- Future Test Types ---
// Property-based tests: to be implemented with proptest 🦀
// Snapshot tests: to be implemented with insta 🦀
// Performance benchmarks: to be implemented with criterion 🦀

/// Runs a single Mooneye GB test ROM using the gboxide emulator core.
///
/// # Arguments
/// * `name` - Relative path to the test ROM within `assets/mooneye-test-suite/`.
///
/// # Returns
/// * `Result<(), anyhow::Error>` - Ok if the test passes, Err with context otherwise.
///
/// # Panics
/// Panics if the test fails or the ROM cannot be loaded.
pub fn run_test(name: &str) -> Result<()> {
    info!("Starting test: {}", name);
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let test_rom_path = project_root
        .join("assets/mooneye-test-suite")
        .join(format!("{}.gb", name));
    let rom_data = std::fs::read(&test_rom_path)
        .with_context(|| format!("Failed to read test ROM: {}", test_rom_path.display()))?;
    debug!(
        "ROM loaded: {} ({} bytes)",
        test_rom_path.display(),
        rom_data.len()
    );
    let mut mmu = MMU::new(rom_data).context("Failed to create MMU")?;
    let mut cpu = CPU::new();

    let max_duration = Duration::from_secs(120);
    let start_time = Instant::now();
    let mut test_passed = false;

    // TODO: Implement proper event/flag signalling for test completion
    // For now, check register values after a fixed number of cycles
    for _ in 0..10_000_000 {
        if start_time.elapsed() > max_duration {
            error!("Test timed out: {}", name);
            break;
        }
        let _cycles = cpu.step(&mut mmu);
        // TODO: Check for test completion (e.g., via memory-mapped flag or register)
        // For now, check if CPU is halted or a specific register value is set
        // Example: if cpu.regs.a == 0 { test_passed = true; break; }
    }

    // TODO: Implement proper pass/fail detection
    // For now, always fail to indicate incomplete implementation
    assert_eq!(test_passed, true, "Test did not finish or pass: {}", name);
    info!("Test completed: {}", name);
    Ok(())
}
