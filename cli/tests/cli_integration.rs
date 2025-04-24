/// Integration test for the gboxide CLI
///
/// This test checks that the CLI can run a real .gb ROM file using the 'run' subcommand.
/// It verifies error handling for missing files and correct output for valid ROMs.
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

#[test]
fn run_real_rom_headless() -> Result<(), Box<dyn std::error::Error>> {
    let rom_path = Path::new("test-roms/cpu_instrs/cpu_instrs.gb");
    if !rom_path.exists() {
        eprintln!("Test ROM not found: {}. Skipping test.", rom_path.display());
        return Ok(());
    }
    let mut cmd = Command::cargo_bin("cli")?;
    cmd.arg("run").arg(rom_path).arg("--headless");
    cmd.assert().success();
    Ok(())
}

#[test]
fn error_on_missing_rom() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("cli")?;
    cmd.arg("run").arg("nonexistent.gb").arg("--headless");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("ROM file not found"));
    Ok(())
}

#[test]
fn run_with_debug_flag() -> Result<(), Box<dyn std::error::Error>> {
    let rom_path = Path::new("test-roms/cpu_instrs/cpu_instrs.gb");
    if !rom_path.exists() {
        eprintln!("Test ROM not found: {}. Skipping test.", rom_path.display());
        return Ok(());
    }
    let mut cmd = Command::cargo_bin("cli")?;
    cmd.arg("run")
        .arg(rom_path)
        .arg("--headless")
        .arg("--debug");
    cmd.assert().success();
    Ok(())
}
