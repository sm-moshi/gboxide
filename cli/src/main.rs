use core::{MemoryBus, CPU, MMU};

fn main() {
    // Set up MMU and inject test program
    let mut mmu = MMU::new();

    // Program: NOP (0x00), NOP (0x00), HALT (0x76)
    mmu.write(0x0100, 0x00);
    mmu.write(0x0101, 0x00);
    mmu.write(0x0102, 0x76);

    // Set up CPU and set initial PC to 0x0100 (Game Boy entry point)
    let mut cpu = CPU::new();
    cpu.regs.pc = 0x0100;

    // Execute 3 instructions
    for _ in 0..3 {
        let opcode = mmu.read(cpu.regs.pc);
        println!(
            "Executing opcode: 0x{:02X} at PC=0x{:04X}",
            opcode, cpu.regs.pc
        );
        let cycles = cpu.step(&mut mmu);
        println!("-> Took {} cycles", cycles);
        if cpu.halted {
            println!("CPU entered HALT state.");
            break;
        }
    }
}
