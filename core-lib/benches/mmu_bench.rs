// Why: This benchmark is rewritten to use the public MMU API and to panic on error for all setup steps, as is idiomatic for Criterion benches. All MMU construction uses a dummy ROM and .expect for clarity and maintainability, in line with project rules. No bitflags or private/internal API is used.

use core_lib::mmu::MMU;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn mmu_access_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("MMU Memory Access");
    group.sample_size(100);

    // Create a test ROM (32KB)
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom).expect("bench setup failed");

    // Basic memory operations
    group.bench_function("read_write_rom", |b| {
        b.iter(|| {
            mmu.write(0x0000, 0x42).expect("write failed");
            black_box(mmu.read(0x0000))
        })
    });

    group.bench_function("read_write_wram", |b| {
        b.iter(|| {
            mmu.write(0xC000, 0x42).expect("write failed");
            black_box(mmu.read(0xC000))
        })
    });

    group.bench_function("read_write_hram", |b| {
        b.iter(|| {
            mmu.write(0xFF80, 0x42).expect("write failed");
            black_box(mmu.read(0xFF80))
        })
    });

    // DMA operations
    group.bench_function("dma_transfer", |b| {
        b.iter(|| {
            // Setup source data
            for i in 0..0xA0 {
                mmu.write(0xC000 + i, i as u8).expect("write failed");
            }
            mmu.write(0xFF46, 0xC0).expect("write failed"); // Start DMA from 0xC000
            black_box(mmu.step(160)) // DMA takes 160 cycles
        })
    });

    group.finish();
}

fn mmu_banking_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("MMU Banking");
    group.sample_size(100);

    // Create a test ROM with MBC1 (32KB)
    let mut rom = vec![0; 0x8000];
    rom[0x147] = 0x01; // MBC1
    rom[0x148] = 0x01; // 64KB ROM
    rom[0x149] = 0x02; // 8KB RAM
    let mut mmu = MMU::new(rom).expect("bench setup failed");

    // ROM banking
    group.bench_function("rom_banking", |b| {
        b.iter(|| {
            // Switch ROM banks
            mmu.write(0x2000, 0x01).expect("write failed"); // Select ROM bank 1
            black_box(mmu.read(0x4000)) // Read from banked area
        })
    });

    // RAM banking
    group.bench_function("ram_banking", |b| {
        b.iter(|| {
            // Switch RAM banks
            mmu.write(0x4000, 0x01).expect("write failed"); // Select RAM bank 1
                                                            // Enable RAM before writing (required by MBC1)
            mmu.write(0x0000, 0x0A).expect("enable RAM failed");
            mmu.write(0xA000, 0x42).expect("write failed"); // Write to external RAM
            black_box(mmu.read(0xA000)) // Read from external RAM
        })
    });

    group.finish();
}

criterion_group!(benches, mmu_access_benchmark, mmu_banking_benchmark);
criterion_main!(benches);
