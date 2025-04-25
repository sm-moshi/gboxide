// Integration tests for core Game Boy emulator features
// Why: Ensure all major subsystems work together as expected (CPU, MMU, Timer, PPU, Input)
//
// These tests use the public API to simulate real usage and verify integration, not just unit logic.

use anyhow::Result;
use core_lib::cpu::CPU;
use core_lib::mmu::{GameBoyButton, MMU};
use pretty_assertions::assert_eq;

#[test]
fn integration_cpu_mmu_basic_rom_execution() -> Result<()> {
    // Use a valid ROM size (0x8000 bytes), with test program at 0x0100 (entry point)
    let mut rom = vec![0; 0x8000];
    rom[0x0100] = 0x00; // NOP
    rom[0x0101] = 0x00; // NOP
    rom[0x0102] = 0x76; // HALT
    let mut mmu = MMU::new(rom)?;
    let mut cpu = CPU::new();
    cpu.regs.pc = 0x0100; // Start at entry point
    let mut steps = 0;
    // Run until HALT
    loop {
        let opcode = mmu.read(cpu.regs.pc);
        if opcode == 0x76 {
            break;
        }
        let _ = cpu.step(&mut mmu);
        steps += 1;
        assert!(steps <= 10, "CPU did not halt as expected");
    }
    Ok(())
}

#[test]
fn integration_dma_and_oam_transfer() -> Result<()> {
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom)?;
    // Fill WRAM with pattern
    for i in 0..0xA0 {
        // Safe: i always fits in u8 for test data
        let _ = mmu.write(0xC000 + i, u8::try_from(i).unwrap_or(0));
    }
    // Start DMA from WRAM
    let _ = mmu.write(0xFF46, 0xC0); // Source: 0xC000
    mmu.step(160); // DMA cycles
                   // Step additional cycles to ensure PPU is in HBlank/VBlank (OAM accessible)
    mmu.step(456 * 2); // Two scanlines
    for i in 0..0xA0 {
        // Safe: i always fits in u8 for test data
        assert_eq!(mmu.read(0xFE00 + i), u8::try_from(i).unwrap_or(0));
    }
    Ok(())
}

#[test]
fn integration_timer_interrupt() -> Result<()> {
    // Initialise tracing for debug output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom)?;
    // Enable timer and set modulo
    let _ = mmu.write(0xFF07, 0x05); // TAC: enable, freq 4096Hz
    let _ = mmu.write(0xFF06, 0xAB); // TMA
    let _ = mmu.write(0xFF05, 0xFF); // TIMA (will overflow)
                                     // Step enough cycles to guarantee timer overflow and interrupt request (hardware: 4 cycles delay after overflow, so step extra for safety)
    let mut if_reg = 0;
    for _ in 0..32 {
        // Increased to 32 to ensure timer can reach Reloading and set IF
        mmu.step(1);
        if_reg = mmu.read(0xFF0F);
        if (if_reg & 0b100) != 0 {
            break;
        }
    }
    assert!(
        (if_reg & 0b100) != 0,
        "Timer interrupt not set in IF register"
    );
    assert_eq!(mmu.timer.read(0xFF05)?, 0xAB); // TIMA reloaded
    Ok(())
}

#[test]
fn integration_ppu_sprite_rendering() -> Result<()> {
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom)?;
    // Set up a sprite in OAM
    let _ = mmu.write(0xFE00, 16); // Y
    let _ = mmu.write(0xFE01, 16); // X
    let _ = mmu.write(0xFE02, 1); // Tile
    let _ = mmu.write(0xFE03, 0); // Attr
                                  // Set up tile data
    let _ = mmu.write(0x8010, 0b0111_1110);
    let _ = mmu.write(0x8011, 0b1000_0001);
    // Enable LCD, sprites, BG
    let _ = mmu.write(0xFF40, 0x83);
    // Set scanline to 0 by writing to LY (not writable, so step until LY==0)
    // Instead, just step one scanline and check LY
    mmu.step(456);
    let ly = mmu.read(0xFF44);
    // Check pixel in framebuffer (use public get_frame_buffer)
    let pixel_index = ly as usize * 160 + 8; // SCREEN_WIDTH = 160
    let expected = core_lib::ppu::color::Color::WHITE;
    let fb = mmu.ppu.get_frame_buffer();
    assert_eq!(fb[pixel_index], expected);
    Ok(())
}

#[test]
fn integration_input_joypad_interrupt() -> Result<()> {
    let rom = vec![0; 0x8000];
    let mut mmu = MMU::new(rom)?;
    // Select action buttons (set bit 5 low)
    let _ = mmu.write(0xFF00, 0x20); // Select action (A/B/Start/Select)
                                     // Simulate releasing A (start with released state)
    mmu.update_joypad(GameBoyButton::A, false);
    // Simulate pressing A (falling edge triggers interrupt)
    mmu.update_joypad(GameBoyButton::A, true);
    // Step a few cycles to allow the interrupt to be registered (hardware-accurate edge detection)
    for _ in 0..4 {
        mmu.step(1);
    }
    // Should trigger joypad interrupt (bit 4 in IF register)
    let if_reg = mmu.read(0xFF0F);
    assert!(
        (if_reg & 0b10000) != 0,
        "Joypad interrupt not set in IF register"
    );
    // Simulate releasing A again
    mmu.update_joypad(GameBoyButton::A, false);
    // Interrupt should not be re-triggered (bit should remain set until cleared by CPU)
    Ok(())
}
