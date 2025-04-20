/// cli/src/main.rs
use core_lib::{cpu::CPU, mmu::MMU};
use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_path = PathBuf::from("roms/test.gb");
    let rom_data = fs::read(&rom_path)?;

    // Create cartridge and MMU
    let mut mmu = MMU::new(rom_data)?;
    let mut cpu = CPU::new();

    // Write a simple test program (NOP, NOP, HALT)
    mmu.write(0x0100, 0x00); // NOP
    mmu.write(0x0101, 0x00); // NOP
    mmu.write(0x0102, 0x76); // HALT

    // Main emulation loop
    loop {
        let opcode = mmu.read(cpu.regs.pc);
        if opcode == 0x76 {
            // HALT instruction
            break;
        }
        let cycles = cpu.step(&mut mmu);
        mmu.step(u32::from(cycles));
    }

    Ok(())
}
