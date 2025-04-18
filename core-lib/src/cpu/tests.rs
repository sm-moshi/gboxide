/// core-lib/src/cpu/tests.rs
use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::mmu::{MemoryBusTrait, MMU};

#[cfg(test)]
use pretty_assertions::assert_eq;

/// Creates a new MMU instance with a ROM containing the test program
fn create_test_mmu(program: &[u8]) -> MMU {
    let mut rom = vec![0; 0x8000]; // 32KB ROM
    rom[0x100..0x100 + program.len()].copy_from_slice(program);
    MMU::new(rom).expect("Failed to create test MMU")
}

#[test]
fn test_ld_b_immediate() {
    let mut cpu = CPU::new();
    let program = [0x06, 0x42]; // LD B, 0x42
    let mut mmu = create_test_mmu(&program);
    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);
    assert_eq!(cpu.regs.b, 0x42);
}

#[test]
fn test_add_a_n() {
    let mut cpu = CPU::new();
    let program = [0xC6, 0x05]; // ADD A, 5
    let mut mmu = create_test_mmu(&program);
    cpu.regs.a = 3;
    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);
    assert_eq!(cpu.regs.a, 8);
    assert_eq!(cpu.regs.f & 0x80, 0x00); // not zero
}

#[test]
fn test_add_a_b() {
    let mut cpu = CPU::new();
    let program = [0x80]; // ADD A, B
    let mut mmu = create_test_mmu(&program);
    cpu.regs.a = 5;
    cpu.regs.b = 7;
    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);
    assert_eq!(cpu.regs.a, 12);
    assert_eq!(cpu.regs.f & 0x80, 0x00); // not zero
}

#[test]
fn test_inc_b() {
    let mut cpu = CPU::new();
    let program = [0x04]; // INC B
    let mut mmu = create_test_mmu(&program);
    cpu.regs.b = 0x0F;
    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);
    assert_eq!(cpu.regs.b, 0x10);
    assert_eq!(cpu.regs.f & 0x20, 0x20); // H flag set
}

#[test]
fn test_add_hl_bc() {
    let mut cpu = CPU::new();
    let program = [0x09]; // ADD HL, BC
    let mut mmu = create_test_mmu(&program);

    // Set initial flag state
    cpu.regs.f = 0x80; // Set Z flag initially to verify it's preserved

    // Set registers for addition
    cpu.regs.set_hl(0x1000);
    cpu.regs.set_bc(0x2000);

    println!(
        "Before ADD HL, BC: HL = {:#06X}, BC = {:#06X}, F = {:#04X}",
        cpu.regs.hl(),
        cpu.regs.bc(),
        cpu.regs.f
    );

    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);

    println!(
        "After ADD HL, BC: HL = {:#06X}, F = {:#04X}",
        cpu.regs.hl(),
        cpu.regs.f
    );

    // Verify results
    assert_eq!(cpu.regs.hl(), 0x3000, "HL should be 0x3000");
    assert_eq!(cpu.regs.f & 0x80, 0x80, "Z flag should be preserved");
    assert_eq!(cpu.regs.f & 0x40, 0x00, "N flag should be reset");
    assert_eq!(
        cpu.regs.f & 0x20,
        0x00,
        "H flag should not be set (no half carry)"
    );
    assert_eq!(
        cpu.regs.f & 0x10,
        0x00,
        "C flag should not be set (no carry)"
    );
}

#[test]
fn test_instruction_timing() {
    let mut cpu = CPU::new();
    let program = [
        0x00, // NOP (4 cycles)
        0x06, 0x42, // LD B, n (8 cycles)
        0x80, // ADD A, B (4 cycles)
        0xCB, 0x11, // RL C (8 cycles - CB prefix)
    ];
    let mut mmu = create_test_mmu(&program);
    cpu.regs.pc = 0x0100;

    // Test NOP timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 4, "NOP should take 4 cycles");
    assert_eq!(cpu.get_cycles(), 4);

    // Test LD B, n timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 8, "LD B, n should take 8 cycles");
    assert_eq!(cpu.get_cycles(), 12);

    // Test ADD A, B timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 4, "ADD A, B should take 4 cycles");
    assert_eq!(cpu.get_cycles(), 16);

    // Test CB prefix instruction timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 8, "CB prefix instruction should take 8 cycles");
    assert_eq!(cpu.get_cycles(), 24);
}

#[test]
fn test_conditional_jump_timing() {
    let mut cpu = CPU::new();
    let program = [
        0x3E, 0x00, // LD A, 0 (8 cycles)
        0x20, 0x02, // JR NZ, 2 (12 cycles)
        0x00, // NOP (4 cycles)
        0x00, // NOP (4 cycles)
    ];
    let mut mmu = create_test_mmu(&program);
    cpu.regs.pc = 0x0100;

    // Execute LD A, 0
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 8, "LD A, 0 should take 8 cycles");

    // Execute JR NZ, 2 (should not jump as Z flag is set)
    cpu.regs.f |= 0x80; // Set Z flag
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 12, "JR NZ (not taken) should take 12 cycles");

    // Execute same jump with Z flag clear (should jump)
    cpu.regs.pc = 0x0102; // Reset PC to jump instruction
    cpu.regs.f &= !0x80; // Clear Z flag
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 12, "JR NZ (taken) should take 12 cycles");
}

#[test]
fn test_halt_timing() {
    let mut cpu = CPU::new();
    let program = [0x76]; // HALT
    let mut mmu = create_test_mmu(&program);
    cpu.regs.pc = 0x0100;

    // Execute HALT
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 4, "HALT should take 4 cycles");
    assert!(cpu.halted, "CPU should be halted");

    // Check cycles while halted
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 4, "Halted CPU should take 4 cycles per step");
}

#[test]
fn test_memory_operation_timing() {
    let mut cpu = CPU::new();
    let program = [
        0x3E, 0x42, // LD A, n (8 cycles)
        0xEA, 0x00, 0xC0, // LD (nn), A (16 cycles)
        0xFA, 0x00, 0xC0, // LD A, (nn) (16 cycles)
    ];
    let mut mmu = create_test_mmu(&program);
    cpu.regs.pc = 0x0100;

    // Test LD A, n timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 8, "LD A, n should take 8 cycles");

    // Test LD (nn), A timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 16, "LD (nn), A should take 16 cycles");

    // Test LD A, (nn) timing
    let cycles = cpu.step(&mut mmu);
    assert_eq!(cycles, 16, "LD A, (nn) should take 16 cycles");
}
