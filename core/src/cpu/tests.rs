use crate::bus::MemoryBus;
use crate::cpu::CPU;
use crate::mmu::MMU;

#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn test_ld_b_immediate() {
    let mut cpu = CPU::new();
    let mut mmu = MMU::new();

    // LD B, 0x42; HALT
    mmu.write(0x0100, 0x06); // opcode
    mmu.write(0x0101, 0x42); // immediate value
    mmu.write(0x0102, 0x76); // HALT

    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);
    assert_eq!(cpu.regs.b, 0x42);
}

#[test]
fn test_add_a_n() {
    let mut cpu = CPU::new();
    let mut mmu = MMU::new();
    mmu.write(0x0100, 0xC6); // ADD A, n
    mmu.write(0x0101, 0x05); // 5
    cpu.regs.a = 3;
    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);
    assert_eq!(cpu.regs.a, 8);
    assert_eq!(cpu.regs.f & 0x80, 0x00); // not zero
}

#[test]
fn test_add_a_b() {
    let mut cpu = CPU::new();
    let mut mmu = MMU::new();

    cpu.regs.a = 5;
    cpu.regs.b = 7;

    mmu.write(0x0100, 0x80); // ADD A, B
    cpu.regs.pc = 0x0100;
    cpu.step(&mut mmu);

    assert_eq!(cpu.regs.a, 12);
    assert_eq!(cpu.regs.f & 0x80, 0x00); // not zero
}

#[test]
fn test_inc_b() {
    let mut cpu = CPU::new();
    let mut mmu = MMU::new();

    cpu.regs.b = 0x0F;
    mmu.write(0x0100, 0x04); // INC B
    cpu.regs.pc = 0x0100;

    cpu.step(&mut mmu);

    assert_eq!(cpu.regs.b, 0x10);
    assert_eq!(cpu.regs.f & 0x20, 0x20); // H flag set
}

#[test]
fn test_add_hl_bc() {
    let mut cpu = CPU::new();
    let mut mmu = MMU::new();

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

    // Execute ADD HL, BC
    mmu.write(0x0100, 0x09);
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
