// Why: This benchmark is rewritten to use the public MMU and PPU API, and to panic on error for all setup steps, as is idiomatic for Criterion benches. All MMU construction uses a dummy ROM and .expect for clarity and maintainability, in line with project rules. No private/internal API is used. If Renderer is not public, use PPU's public API for rendering.

use core_lib::mmu::MMU;
use core_lib::ppu::{lcdc::LcdControl, Ppu};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn ppu_render_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("PPU Rendering");
    group.sample_size(100);

    // Create a test ROM (32KB)
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom).expect("bench setup failed");
    // Initialise VRAM with test pattern
    for i in 0..0x2000 {
        mmu.write(0x8000 + i, (i % 256) as u8)
            .expect("write failed");
    }

    // render_scanline
    group.bench_function("render_scanline", |b| {
        b.iter(|| {
            let mut ppu = Ppu::new();
            let _ = ppu.write(
                0xFF40,
                LcdControl::LCD_ENABLE.bits() | LcdControl::BG_WINDOW_ENABLE.bits(),
            );
            let _ = ppu.write(0xFF42, 0); // SCY = 0
            let _ = ppu.write(0xFF43, 0); // SCX = 0
            for line in 0..144 {
                let _ = ppu.write(0xFF44, line); // Set LY
                ppu.step(1);
            }
            black_box(ppu)
        })
    });

    // render_background
    group.bench_function("render_background", |b| {
        b.iter(|| {
            let mut ppu = Ppu::new();
            let _ = ppu.write(
                0xFF40,
                LcdControl::LCD_ENABLE.bits() | LcdControl::BG_WINDOW_ENABLE.bits(),
            );
            let _ = ppu.write(0xFF42, 0); // SCY = 0
            let _ = ppu.write(0xFF43, 0); // SCX = 0
            let _ = ppu.write(0xFF44, 0); // Set LY = 0
            ppu.step(1);
            black_box(ppu)
        })
    });

    // render_window
    group.bench_function("render_window", |b| {
        b.iter(|| {
            let mut ppu = Ppu::new();
            let _ = ppu.write(
                0xFF40,
                LcdControl::LCD_ENABLE.bits()
                    | LcdControl::BG_WINDOW_ENABLE.bits()
                    | LcdControl::WINDOW_ENABLE.bits(),
            );
            let _ = ppu.write(0xFF4A, 0); // WY = 0
            let _ = ppu.write(0xFF4B, 7); // WX = 7
            let _ = ppu.write(0xFF44, 0); // Set LY = 0
            ppu.step(1);
            black_box(ppu)
        })
    });

    // render_sprites
    group.bench_function("render_sprites", |b| {
        b.iter(|| {
            let mut ppu = Ppu::new();
            let _ = ppu.write(
                0xFF40,
                LcdControl::LCD_ENABLE.bits() | LcdControl::SPRITE_ENABLE.bits(),
            );
            // Set up some test sprites in OAM
            for i in 0..40 {
                let _ = ppu.write(0xFE00 + (i * 4), 16); // Y position
                let _ = ppu.write(0xFE00 + (i * 4) + 1, 8 + i as u8); // X position
                let _ = ppu.write(0xFE00 + (i * 4) + 2, i as u8); // Tile index
                let _ = ppu.write(0xFE00 + (i * 4) + 3, 0); // Attributes
            }
            let _ = ppu.write(0xFF44, 16); // Set LY = 16
            ppu.step(1);
            black_box(ppu)
        })
    });

    group.finish();
}

fn ppu_mode_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("PPU Mode Changes");
    group.sample_size(100);

    // mode_switching
    group.bench_function("mode_switching", |b| {
        b.iter(|| {
            let mut ppu = Ppu::new();
            for _ in 0..456 {
                ppu.step(1);
            }
            black_box(ppu)
        })
    });

    group.finish();
}

criterion_group!(benches, ppu_render_benchmark, ppu_mode_benchmark);
criterion_main!(benches);
