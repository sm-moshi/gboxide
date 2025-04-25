// Why: This benchmark is rewritten to use the current struct-based Flags design (not bitflags),
// and to use the public API for CPU and MMU construction. All MMU construction uses a dummy ROM
// and panics on error, as is idiomatic for Criterion benches. All flag operations use explicit
// struct initialisation for clarity and maintainability, in line with project rules.

use core_lib::cpu::{Flags, CPU};
use core_lib::mmu::MMU;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn cpu_step_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU Operations");
    group.sample_size(100);

    // Setup
    let mut cpu = CPU::new();
    let mut mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");

    // Basic NOP instruction
    group.bench_function("NOP instruction", |b| {
        b.iter(|| {
            cpu = CPU::new();
            mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");
            mmu.write(0x0000, 0x00).expect("write failed"); // NOP
            black_box(cpu.step(&mut mmu))
        })
    });

    // LD r,r instructions
    group.bench_function("LD r,r instructions", |b| {
        b.iter(|| {
            cpu = CPU::new();
            mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");
            mmu.write(0x0000, 0x7F).expect("write failed"); // LD A,A
            black_box(cpu.step(&mut mmu))
        })
    });

    // ADD A,r instructions
    group.bench_function("ADD A,r instructions", |b| {
        b.iter(|| {
            cpu = CPU::new();
            mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");
            cpu.set_reg_a(0x12);
            mmu.write(0x0000, 0x87).expect("write failed"); // ADD A,A
            black_box(cpu.step(&mut mmu))
        })
    });

    // Flag operations
    group.bench_function("Flag operations", |b| {
        b.iter(|| {
            cpu = CPU::new();
            let flags = Flags {
                zero: true,
                subtract: true,
                half_carry: false,
                carry: false,
            };
            cpu.regs.set_flags(flags);
            black_box(cpu.regs.flags())
        })
    });

    // Memory operations
    group.bench_function("Memory operations", |b| {
        b.iter(|| {
            cpu = CPU::new();
            mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");
            mmu.write(0x0000, 0x3E).expect("write failed"); // LD A,d8
            mmu.write(0x0001, 0x42).expect("write failed");
            black_box(cpu.step(&mut mmu))
        })
    });

    group.finish();
}

fn cpu_interrupt_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU Interrupts");
    group.sample_size(100);

    let mut cpu = CPU::new();
    let mut mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");

    // Interrupt handling
    group.bench_function("Interrupt handling", |b| {
        b.iter(|| {
            cpu = CPU::new();
            mmu = MMU::new(vec![0; 0x8000]).expect("bench setup failed");
            cpu.ime = true;
            mmu.write(0xFF0F, 0x01).expect("write failed"); // Request VBlank
            black_box(cpu.handle_interrupts(&mut mmu))
        })
    });

    group.finish();
}

criterion_group!(benches, cpu_step_benchmark, cpu_interrupt_benchmark);
criterion_main!(benches);
