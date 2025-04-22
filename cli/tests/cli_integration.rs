use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

/// Integration test for the gboxide CLI
///
/// This test checks that the CLI can run a real .gb ROM file using the 'run' subcommand.
/// It verifies error handling for missing files and correct output for valid ROMs.
#[test]
fn run_real_rom_headless() {
    let rom_path = Path::new("test-roms/cpu_instrs/cpu_instrs.gb");
    if !rom_path.exists() {
        eprintln!("Test ROM not found: {}. Skipping test.", rom_path.display());
        return;
    }
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.arg("run").arg(rom_path).arg("--headless");
    cmd.assert().success();
}

#[test]
fn error_on_missing_rom() {
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.arg("run").arg("nonexistent.gb").arg("--headless");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("ROM file not found"));
}

#[test]
fn run_with_debug_flag() {
    let rom_path = Path::new("test-roms/cpu_instrs/cpu_instrs.gb");
    if !rom_path.exists() {
        eprintln!("Test ROM not found: {}. Skipping test.", rom_path.display());
        return;
    }
    let mut cmd = Command::cargo_bin("cli").unwrap();
    cmd.arg("run")
        .arg(rom_path)
        .arg("--headless")
        .arg("--debug");
    cmd.assert().success();
}
