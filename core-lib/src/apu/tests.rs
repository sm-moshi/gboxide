use super::channel1::*;
use super::channel2::*;
use super::channel3::*;
use super::channel4::*;
use super::envelope::*;
use super::{
    Apu, NR10_ADDR, NR11_ADDR, NR12_ADDR, NR13_ADDR, NR14_ADDR, NR30_ADDR, NR31_ADDR, NR32_ADDR,
    NR33_ADDR, NR34_ADDR, NR41_ADDR, NR42_ADDR, NR43_ADDR, NR44_ADDR, WAVE_RAM_START,
};
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use tracing::info;

static TRACING: Lazy<()> = Lazy::new(|| {
    let _ = tracing_subscriber::fmt::try_init();
});

/// Test Channel1 register read/write and bitfield behaviour
#[test]
fn test_channel1_registers() {
    let mut ch1 = Channel1::default();

    // Test NR10 sweep register
    ch1.nr10.set_sweep_time(0b101);
    ch1.nr10.set_sweep_increase(true);
    ch1.nr10.set_sweep_shift(0b011);
    assert_eq!(ch1.nr10.sweep_time(), 0b101);
    assert_eq!(ch1.nr10.sweep_increase(), true);
    assert_eq!(ch1.nr10.sweep_shift(), 0b011);

    // Test NR11 duty/length
    ch1.nr11.set_duty(0b10);
    ch1.nr11.set_length(0b10_1010);
    assert_eq!(ch1.nr11.duty(), 0b10);
    assert_eq!(ch1.nr11.length(), 0b10_1010);

    // Test NR12 envelope
    ch1.nr12.set_initial_volume(0xF);
    ch1.nr12.set_envelope_direction(true);
    ch1.nr12.set_envelope_period(0b101);
    assert_eq!(ch1.nr12.initial_volume(), 0xF);
    assert_eq!(ch1.nr12.envelope_direction(), true);
    assert_eq!(ch1.nr12.envelope_period(), 0b101);

    // Test NR13 (raw u8)
    ch1.nr13 = 0xAB;
    assert_eq!(ch1.nr13, 0xAB);

    // Test NR14
    ch1.nr14.set_trigger(true);
    ch1.nr14.set_length_enable(true);
    ch1.nr14.set_freq_high(0b101);
    assert_eq!(ch1.nr14.trigger(), true);
    assert_eq!(ch1.nr14.length_enable(), true);
    assert_eq!(ch1.nr14.freq_high(), 0b101);

    // Test read_reg/write_reg
    ch1.write_reg(0, 0x77);
    assert_eq!(ch1.read_reg(0), 0x77);
    ch1.write_reg(1, 0x88);
    assert_eq!(ch1.read_reg(1), 0x88);
    ch1.write_reg(2, 0x99);
    assert_eq!(ch1.read_reg(2), 0x99);
    ch1.write_reg(3, 0xAA);
    assert_eq!(ch1.read_reg(3), 0xAA);
    ch1.write_reg(4, 0xBB);
    assert_eq!(ch1.read_reg(4), 0xBB);
}

/// Test APU memory-mapped I/O for Channel 1
#[test]
fn test_apu_mmio_channel1() {
    println!(
        "[TRACE] Running test_apu_mmio_channel1 from {}:{}",
        file!(),
        line!()
    );
    tracing::info!(
        "[TRACE] Running test_apu_mmio_channel1 from {}:{}",
        file!(),
        line!()
    );
    let mut apu = Apu {
        enabled: true,
        ..Default::default()
    };

    // Write to each Channel 1 register via MMIO
    apu.write(NR10_ADDR, 0x12);
    apu.write(NR11_ADDR, 0x34);
    apu.write(NR12_ADDR, 0x56);
    apu.write(NR13_ADDR, 0x78);
    apu.write(NR14_ADDR, 0x9A);

    // Read back and check values
    assert_eq!(apu.read(NR10_ADDR), 0x12);
    assert_eq!(apu.read(NR11_ADDR), 0x34);
    assert_eq!(apu.read(NR12_ADDR), 0x56);
    // NR13 is write-only; reads always return 0xFF per hardware spec
    assert_eq!(apu.read(NR13_ADDR), 0xFF);
    assert_eq!(apu.read(NR14_ADDR), 0x9A);

    // Unmapped address returns 0xFF
    assert_eq!(apu.read(0xFF15), 0xFF);
    // Mapped address (NR41) is write-only, always returns 0xFF
    apu.write(0xFF20, 0xCD);
    assert_eq!(apu.read(0xFF20), 0xFF); // Write-only register

    // Writes to unmapped addresses do not panic or affect state
    apu.write(0xFF15, 0xAB);
    apu.write(0xFF20, 0xCD);
    // Still returns 0xFF
    assert_eq!(apu.read(0xFF15), 0xFF);
    assert_eq!(apu.read(0xFF20), 0xFF);
}

#[test]
/// Test Channel2 register read/write and envelope behaviour (Mooneye style)
fn test_channel2_registers() {
    *TRACING;
    println!("[TRACE] test_channel2_registers: start");
    info!("[TRACE] test_channel2_registers: start");
    let mut ch2 = Channel2::default();
    // NR21
    println!("[TRACE] before NR21 write_reg");
    ch2.nr21.write_reg(0b1100_0011);
    println!("[TRACE] after NR21 write_reg, before assert 1");
    info!("[TRACE] after NR21 write_reg, before assert 1");
    assert_eq!(ch2.nr21.read_reg() >> 6, 0b11);
    println!("[TRACE] after assert 1, before assert 2");
    info!("[TRACE] after assert 1, before assert 2");
    assert_eq!(ch2.nr21.read_reg() & 0b0011_1111, 0b00_0011);
    // NR22 (Envelope)
    println!("[TRACE] before NR22 write_reg");
    ch2.nr22.write_reg(0b1011_0111);
    assert_eq!(
        ch2.nr22.increasing, false,
        "increasing should be false after write_reg(0b1011_0111)"
    );
    info!("Channel2 NR22 raw: {:08b}", ch2.nr22.read_reg());
    info!(
        "Channel2 NR22 increasing: {} (should be false)",
        ch2.nr22.increasing
    );
    info!("Channel2 NR22 period: {:03b}", ch2.nr22.period);
    info!("Channel2 NR22 volume: {:04b}", ch2.nr22.volume);
    println!("[TRACE] after NR22 write_reg, before assert 3");
    assert_eq!(ch2.nr22.volume, 0b1011);
    println!("[TRACE] after assert 3, before assert 4");
    assert_eq!(ch2.nr22.increasing, false);
    println!("[TRACE] after assert 4, before assert 5");
    assert_eq!(ch2.nr22.period, 0b111);
    println!("[TRACE] after assert 5, before assert 6");
    assert_eq!(ch2.nr22.read_reg(), 0b1011_0111);
    // Positive test: bit 3 set
    ch2.nr22.write_reg(0b1011_1111);
    assert_eq!(
        ch2.nr22.increasing, true,
        "increasing should be true after write_reg(0b1011_1111)"
    );
    // NR23
    ch2.nr23 = 0x5A;
    assert_eq!(ch2.nr23, 0x5A);
    // NR24
    ch2.nr24.write_reg(0b1100_0111);
    assert_eq!(ch2.nr24.read_reg(), 0b1100_0111);
    // Offset read/write
    ch2.write_reg(0, 0x12);
    assert_eq!(ch2.read_reg(0), 0x12);
    ch2.write_reg(1, 0x34);
    assert_eq!(ch2.read_reg(1), 0x34);
    ch2.write_reg(2, 0x56);
    assert_eq!(ch2.read_reg(2), 0x56);
    ch2.write_reg(3, 0x78);
    assert_eq!(ch2.read_reg(3), 0x78);
    // Out-of-bounds
    assert_eq!(ch2.read_reg(4), 0xFF);
}

#[test]
/// Minimal test for Envelope bitfield and read/write methods
fn test_envelope_manual_getter() {
    *TRACING;
    println!("[TRACE] test_envelope_manual_getter: start");
    info!("[TRACE] test_envelope_manual_getter: start");
    let mut env = Envelope::new();
    env.write_reg(0b1011_0111);
    assert_eq!(
        env.increasing, false,
        "increasing should be false after write_reg(0b1011_0111)"
    );
    info!("Envelope raw: {:08b}", env.read_reg());
    info!("Envelope increasing: {} (should be false)", env.increasing);
    info!("Envelope period: {:03b}", env.period);
    info!("Envelope volume: {:04b}", env.volume);
    println!("[TRACE] after write_reg, before assert 1");
    assert_eq!(env.volume, 0b1011);
    println!("[TRACE] after assert 1, before assert 2");
    assert_eq!(env.increasing, false);
    println!("[TRACE] after assert 2, before assert 3");
    assert_eq!(env.period, 0b111);
    println!("[TRACE] after assert 3, before assert 4");
    assert_eq!(env.read_reg(), 0b1011_0111);
    // Positive test: bit 3 set
    env.write_reg(0b1011_1111);
    assert_eq!(
        env.increasing, true,
        "increasing should be true after write_reg(0b1011_1111)"
    );
}

#[test]
/// Direct bitwise operation sanity check (should always pass)
fn test_direct_bitwise() {
    println!("[TRACE] test_direct_bitwise: start");
    assert_eq!((0b1011_1111 & 0b0000_1000) != 0, true);
}

/// Test Channel4 register read/write and bitfield behaviour
#[test]
fn test_channel4_registers() {
    let mut ch4 = Channel4::default();

    // NR41: Sound length (6 bits)
    ch4.nr41.write_reg(0b1010_1010);
    assert_eq!(ch4.nr41.length(), 0b10_1010);
    assert_eq!(ch4.nr41.read_reg(), 0b10_1010);

    // NR42: Envelope (reuse Envelope struct)
    ch4.nr42.write_reg(0b1011_0111);
    assert_eq!(ch4.nr42.volume, 0b1011);
    assert_eq!(ch4.nr42.increasing, false);
    assert_eq!(ch4.nr42.period, 0b111);
    assert_eq!(ch4.nr42.read_reg(), 0b1011_0111);

    // NR43: Polynomial counter
    ch4.nr43.write_reg(0b1101_1011);
    assert_eq!(ch4.nr43.shift_clock_freq(), 0b1101);
    assert_eq!(ch4.nr43.counter_step_width(), true);
    assert_eq!(ch4.nr43.dividing_ratio(), 0b011);
    assert_eq!(ch4.nr43.read_reg(), 0b1101_1011);

    // NR44: Trigger/length enable
    ch4.nr44.write_reg(0b1100_0000);
    assert_eq!(ch4.nr44.trigger(), true);
    assert_eq!(ch4.nr44.length_enable(), true);
    assert_eq!(ch4.nr44.read_reg(), 0b1100_0000);

    // Offset read/write
    ch4.write_reg(0, 0x12);
    assert_eq!(ch4.read_reg(0), 0x12 & 0x3F);
    ch4.write_reg(1, 0x34);
    assert_eq!(ch4.read_reg(1), 0x34);
    ch4.write_reg(2, 0x56);
    assert_eq!(ch4.read_reg(2), 0x56);
    ch4.write_reg(3, 0x78);
    assert_eq!(ch4.read_reg(3), 0x78 & 0xC0);
    // Out-of-bounds
    assert_eq!(ch4.read_reg(4), 0xFF);
}

/// Test APU memory-mapped I/O for Channel 4
#[test]
fn test_apu_mmio_channel4() {
    println!(
        "[TRACE] Running test_apu_mmio_channel4 from {}:{}",
        file!(),
        line!()
    );
    tracing::info!(
        "[TRACE] Running test_apu_mmio_channel4 from {}:{}",
        file!(),
        line!()
    );
    let mut apu = Apu {
        enabled: true,
        ..Default::default()
    };

    // Write to each Channel 4 register via MMIO
    apu.write(NR41_ADDR, 0x21);
    // NR41 is write-only; reads always return 0xFF per hardware spec
    assert_eq!(apu.read(NR41_ADDR), 0xFF);
    apu.write(NR42_ADDR, 0x43);
    assert_eq!(apu.read(NR42_ADDR), 0x43);
    apu.write(NR43_ADDR, 0x65);
    assert_eq!(apu.read(NR43_ADDR), 0x65);
    apu.write(NR44_ADDR, 0x87);
    assert_eq!(apu.read(NR44_ADDR), 0x87 & 0xC0);

    // Unmapped address returns 0xFF
    assert_eq!(apu.read(0xFF24), 0xFF);
    // Writes to unmapped addresses do not panic or affect state
    apu.write(0xFF24, 0xAB);
    assert_eq!(apu.read(0xFF24), 0xFF);
}

/// Test Channel3 register read/write and bitfield behaviour
#[test]
fn test_channel3_registers() {
    let mut ch3 = Channel3::default();

    // NR30: Sound on/off (bit 7)
    ch3.nr30.write_reg(0b1000_0000);
    assert_eq!(ch3.nr30.sound_on(), true);
    assert_eq!(ch3.nr30.read_reg(), 0b1000_0000);
    ch3.nr30.write_reg(0b0000_0000);
    assert_eq!(ch3.nr30.sound_on(), false);

    // NR31: Sound length (8 bits)
    ch3.nr31.write_reg(0xAB);
    assert_eq!(ch3.nr31.length(), 0xAB);
    assert_eq!(ch3.nr31.read_reg(), 0xAB);

    // NR32: Output level (bits 6-5)
    ch3.nr32.write_reg(0b0110_0000);
    assert_eq!(ch3.nr32.output_level(), 0b11);
    assert_eq!(ch3.nr32.read_reg(), 0b0110_0000);

    // NR33: Frequency low (8 bits)
    ch3.nr33.write_reg(0xCD);
    assert_eq!(ch3.nr33.freq_lo(), 0xCD);
    assert_eq!(ch3.nr33.read_reg(), 0xCD);

    // NR34: Frequency high/control (bits 7,6,2-0)
    ch3.nr34.write_reg(0b1100_0111);
    assert_eq!(ch3.nr34.trigger(), true);
    assert_eq!(ch3.nr34.length_enable(), true);
    assert_eq!(ch3.nr34.freq_high(), 0b111);
    assert_eq!(ch3.nr34.read_reg(), 0b1100_0111);

    // Offset read/write
    ch3.write_reg(0, 0x12);
    assert_eq!(ch3.read_reg(0), 0x12 & 0x80);
    ch3.write_reg(1, 0x34);
    assert_eq!(ch3.read_reg(1), 0x34);
    ch3.write_reg(2, 0x60);
    assert_eq!(ch3.read_reg(2), 0x60);
    ch3.write_reg(3, 0x56);
    assert_eq!(ch3.read_reg(3), 0x56);
    ch3.write_reg(4, 0xC7);
    assert_eq!(ch3.read_reg(4), 0xC7);
    // Out-of-bounds
    assert_eq!(ch3.read_reg(5), 0xFF);
}

/// Test Channel3 wave RAM read/write
#[test]
fn test_channel3_wave_ram() {
    let mut ch3 = Channel3::default();
    // Write to all 16 bytes
    for i in 0..16 {
        let Ok(val) = u8::try_from(i) else {
            panic!("i out of range for u8")
        };
        ch3.write_wave_ram(i, val);
    }
    // Read back and check
    for i in 0..16 {
        let Ok(val) = u8::try_from(i) else {
            panic!("i out of range for u8")
        };
        assert_eq!(ch3.read_wave_ram(i), val);
    }
    // Out-of-bounds
    assert_eq!(ch3.read_wave_ram(16), 0xFF);
}

/// Test APU memory-mapped I/O for Channel 3 (registers and wave RAM)
#[test]
fn test_apu_mmio_channel3() {
    let mut apu = Apu {
        enabled: true,
        ..Default::default()
    };
    // Write to each Channel 3 register via MMIO
    apu.write(NR30_ADDR, 0x80);
    // NR31 is write-only, always returns 0xFF
    apu.write(NR31_ADDR, 0x12);
    assert_eq!(apu.read(NR31_ADDR), 0xFF);
    apu.write(NR32_ADDR, 0x60);
    assert_eq!(apu.read(NR32_ADDR), 0x60);
    // NR33 is write-only, always returns 0xFF
    apu.write(NR33_ADDR, 0x34);
    assert_eq!(apu.read(NR33_ADDR), 0xFF);
    apu.write(NR34_ADDR, 0xC7);
    assert_eq!(apu.read(NR34_ADDR), 0xC7);
    // Read back and check values
    assert_eq!(apu.read(NR30_ADDR), 0x80);
    // Write/read wave RAM
    for i in 0..16 {
        let Ok(v) = u16::try_from(i) else {
            panic!("i out of range for u16")
        };
        let addr = WAVE_RAM_START + v;
        let Ok(val) = u8::try_from(i) else {
            panic!("i out of range for u8")
        };
        apu.write(addr, val);
        assert_eq!(apu.read(addr), val);
    }
    // Out-of-bounds wave RAM
    assert_eq!(apu.ch3.read_wave_ram(16), 0xFF);
    // Unmapped address returns 0xFF
    assert_eq!(apu.read(0xFF40), 0xFF);
    // Writes to unmapped addresses do not panic or affect state
    apu.write(0xFF40, 0xAB);
    assert_eq!(apu.read(0xFF40), 0xFF);
}

#[cfg(test)]
mod wave_duty_tests {
    use super::super::wave_duty::WaveDuty;

    #[test]
    fn test_pattern_bits() {
        assert_eq!(WaveDuty::Duty12_5.pattern_bits(), 0b0000_0001);
        assert_eq!(WaveDuty::Duty25.pattern_bits(), 0b1000_0001);
        assert_eq!(WaveDuty::Duty50.pattern_bits(), 0b1000_0111);
        assert_eq!(WaveDuty::Duty75.pattern_bits(), 0b0111_1110);
    }

    #[test]
    fn test_enum_variants() {
        let d1 = WaveDuty::Duty12_5;
        let d2 = WaveDuty::Duty25;
        let d3 = WaveDuty::Duty50;
        let d4 = WaveDuty::Duty75;
        assert!(matches!(d1, WaveDuty::Duty12_5));
        assert!(matches!(d2, WaveDuty::Duty25));
        assert!(matches!(d3, WaveDuty::Duty50));
        assert!(matches!(d4, WaveDuty::Duty75));
    }
}

#[cfg(test)]
mod sweep_tests {
    use super::super::sweep::Sweep;

    #[test]
    fn test_clock_increase_no_overflow() {
        let mut sweep = Sweep {
            enabled: true,
            period: 2,
            negate: false,
            shift: 1,
            timer: 2,
            shadow_freq: 1000,
        };
        sweep.clock();
        assert_eq!(sweep.shadow_freq, 1500);
        assert!(sweep.enabled);
    }

    #[test]
    fn test_clock_decrease_no_overflow() {
        let mut sweep = Sweep {
            enabled: true,
            period: 2,
            negate: true,
            shift: 1,
            timer: 2,
            shadow_freq: 1000,
        };
        sweep.clock();
        assert_eq!(sweep.shadow_freq, 500);
        assert!(sweep.enabled);
    }

    #[test]
    fn test_clock_overflow() {
        let mut sweep = Sweep {
            enabled: true,
            period: 2,
            negate: false,
            shift: 1,
            timer: 2,
            shadow_freq: 2048,
        };
        sweep.clock();
        // On overflow, enabled should be false
        assert!(!sweep.enabled);
    }

    #[test]
    fn test_clock_noop_when_disabled() {
        let mut sweep = Sweep {
            enabled: false,
            period: 2,
            negate: false,
            shift: 1,
            timer: 2,
            shadow_freq: 1000,
        };
        sweep.clock();
        assert_eq!(sweep.shadow_freq, 1000);
        assert!(!sweep.enabled);
    }

    #[test]
    fn test_trigger_enables_and_initial_calc() {
        let mut sweep = Sweep {
            enabled: false,
            period: 2,
            negate: false,
            shift: 1,
            timer: 0,
            shadow_freq: 1000,
        };
        sweep.trigger(1000);
        assert!(sweep.enabled);
        assert_eq!(sweep.shadow_freq, 1000);
    }

    #[test]
    fn test_trigger_overflow_disables() {
        let mut sweep = Sweep {
            enabled: false,
            period: 2,
            negate: false,
            shift: 1,
            timer: 0,
            shadow_freq: 2048,
        };
        sweep.trigger(2048);
        assert!(!sweep.enabled);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut sweep = Sweep {
            enabled: true,
            period: 2,
            negate: false,
            shift: 1,
            timer: 2,
            shadow_freq: 1000,
        };
        sweep.reset();
        assert!(!sweep.enabled);
        assert_eq!(sweep.timer, 0);
        assert_eq!(sweep.shadow_freq, 0);
    }
}

#[cfg(test)]
mod channel2_bitfield_tests {
    use super::super::channel2::{Channel2, Nr21, Nr24};

    #[test]
    fn nr21_duty_and_length_roundtrip() {
        for duty in 0..=3 {
            for length in 0..=0x3F {
                let mut nr21 = Nr21(0);
                nr21.set_duty(duty);
                nr21.set_length(length);
                assert_eq!(nr21.duty(), duty);
                assert_eq!(nr21.length(), length);
                // Check raw register value
                let reg = (duty << 6) | length;
                assert_eq!(nr21.read_reg(), reg);
            }
        }
    }

    #[test]
    fn nr21_write_and_read_reg() {
        let mut nr21 = Nr21(0);
        nr21.write_reg(0b1010_1100);
        assert_eq!(nr21.read_reg(), 0b1010_1100);
        assert_eq!(nr21.duty(), 0b10);
        assert_eq!(nr21.length(), 0b10_1100);
    }

    #[test]
    fn nr24_trigger_length_enable_freq_high_roundtrip() {
        let mut nr24 = Nr24(0);
        nr24.set_trigger(true);
        assert!(nr24.trigger());
        nr24.set_trigger(false);
        assert!(!nr24.trigger());
        nr24.set_length_enable(true);
        assert!(nr24.length_enable());
        nr24.set_length_enable(false);
        assert!(!nr24.length_enable());
        for freq in 0..=0x07 {
            nr24.set_freq_high(freq);
            assert_eq!(nr24.freq_high(), freq);
        }
    }

    #[test]
    fn nr24_write_and_read_reg() {
        let mut nr24 = Nr24(0);
        nr24.write_reg(0b1100_0111);
        assert_eq!(nr24.read_reg(), 0b1100_0111);
        assert!(nr24.trigger());
        assert!(nr24.length_enable());
        assert_eq!(nr24.freq_high(), 0b111);
    }

    #[test]
    fn channel2_out_of_bounds_register_access() {
        let ch2 = Channel2::default();
        // Out-of-bounds read returns 0xFF
        assert_eq!(ch2.read_reg(4), 0xFF);
        assert_eq!(ch2.read_reg(255), 0xFF);
        // Out-of-bounds write does not panic or affect state
        let mut ch2 = Channel2::default();
        ch2.write_reg(4, 0xAB);
        ch2.write_reg(255, 0xCD);
        // Registers remain at default
        assert_eq!(ch2.read_reg(0), 0);
        assert_eq!(ch2.read_reg(1), 0);
        assert_eq!(ch2.read_reg(2), 0);
        assert_eq!(ch2.read_reg(3), 0);
    }
}

#[cfg(test)]
mod apu_mod_coverage {
    use crate::apu::{
        Apu, Channel1, Channel2, FrameSequencer, NR10_ADDR, NR21_ADDR, NR30_ADDR, NR42_ADDR,
        WAVE_RAM_START,
    };
    // Use standard assert_eq!
    #[test]
    fn frame_sequencer_tick_all_steps_and_wraparound() {
        let mut seq = FrameSequencer::default();
        let mut steps = Vec::new();
        for _ in 0..(FrameSequencer::NUM_STEPS * 2) {
            let step = seq.tick(FrameSequencer::STEP_CYCLES);
            if let Some(s) = step {
                steps.push(s);
            }
        }
        assert_eq!(steps.len(), 2 * FrameSequencer::NUM_STEPS as usize);
        assert_eq!(steps[0], 1);
        assert_eq!(steps[7], 0);
        assert_eq!(steps[15], 0);
    }

    #[test]
    fn apu_tick_dispatches_to_length_sweep_envelope() {
        let mut apu = Apu::default();
        // Set up channels with nonzero length/envelope
        apu.ch1.length_counter = 2;
        apu.ch2.length_counter = 2;
        apu.ch3.length_counter = 2;
        apu.ch4.length_counter = 2;
        apu.ch1.envelope_timer = 1;
        apu.ch2.envelope_timer = 1;
        apu.ch4.envelope_timer = 1;
        apu.ch1.sweep.enabled = true;
        apu.ch1.sweep.timer = 1;
        // Set length_enable for all channels
        apu.ch1.nr14.write_reg(0x40); // bit 6 = 1
        apu.ch2.nr24.write_reg(0x40);
        apu.ch3.nr34.write_reg(0x40);
        apu.ch4.nr44.write_reg(0x40);
        // Record initial values
        let initial_ch1 = apu.ch1.length_counter;
        let initial_ch2 = apu.ch2.length_counter;
        let initial_ch3 = apu.ch3.length_counter;
        let initial_ch4 = apu.ch4.length_counter;
        // Simulate enough cycles to hit all steps
        for _ in 0..8 {
            apu.tick(FrameSequencer::STEP_CYCLES);
        }
        // All length counters should not increase and should decrement if initially > 0
        assert!(apu.ch1.length_counter <= initial_ch1);
        assert!(apu.ch2.length_counter <= initial_ch2);
        assert!(apu.ch3.length_counter <= initial_ch3);
        assert!(apu.ch4.length_counter <= initial_ch4);
        // If initially > 0, should have decremented at least once
        if initial_ch1 > 0 {
            assert!(apu.ch1.length_counter < initial_ch1);
        }
        if initial_ch2 > 0 {
            assert!(apu.ch2.length_counter < initial_ch2);
        }
        if initial_ch3 > 0 {
            assert!(apu.ch3.length_counter < initial_ch3);
        }
        if initial_ch4 > 0 {
            assert!(apu.ch4.length_counter < initial_ch4);
        }
    }

    #[test]
    fn apu_mmio_read_write_all_channels_and_wave_ram() {
        let mut apu = Apu::default();
        apu.enabled = true;
        // Write and read NR10 (Channel 1)
        apu.write(NR10_ADDR, 0x12);
        assert_eq!(apu.read(NR10_ADDR), 0x12);
        // Write and read NR21 (Channel 2)
        apu.write(NR21_ADDR, 0x34);
        assert_eq!(apu.read(NR21_ADDR), 0x34);
        // Write and read NR30 (Channel 3)
        apu.write(NR30_ADDR, 0x80);
        assert_eq!(apu.read(NR30_ADDR), 0x80);
        // Write and read NR42 (Channel 4)
        apu.write(NR42_ADDR, 0x56);
        assert_eq!(apu.read(NR42_ADDR), 0x56);
        // Write and read wave RAM
        for i in 0..16 {
            apu.write(WAVE_RAM_START + i, i as u8);
            assert_eq!(apu.read(WAVE_RAM_START + i), i as u8);
        }
        // Unmapped address returns 0xFF
        assert_eq!(apu.read(0xFF00), 0xFF);
        // Write to unmapped address does not panic
        apu.write(0xFF00, 0xAB);
    }

    #[test]
    fn apu_mmio_disabled_returns_ff_and_ignores_write() {
        let mut apu = Apu::default();
        apu.enabled = false;
        // All reads except NR52 return 0xFF
        assert_eq!(apu.read(NR10_ADDR), 0xFF);
        // Writes do nothing
        apu.write(NR10_ADDR, 0x12);
        assert_eq!(apu.ch1.nr10.read_reg(), 0);
    }

    #[test]
    fn channel1_trigger_and_enable_disable_logic() {
        let mut ch1 = Channel1::default();
        ch1.nr11.write_reg(0x00); // length bits = 0
        ch1.nr12.write_reg(0xF0); // initial volume = 0xF
        ch1.nr14.write_reg(0xC0); // trigger + length enable
        ch1.trigger();
        assert!(ch1.enabled);
        assert_eq!(ch1.length_counter, 64);
        assert_eq!(ch1.envelope_volume, 0xF);
        // Test tick_length disables channel
        ch1.length_counter = 1;
        ch1.tick_length();
        assert!(!ch1.enabled);
    }

    #[test]
    fn channel2_tick_length_and_envelope() {
        let mut ch2 = Channel2::default();
        ch2.length_counter = 2;
        ch2.nr24.write_reg(0x40); // length enable
        ch2.tick_length();
        assert_eq!(ch2.length_counter, 1);
        ch2.tick_length();
        assert_eq!(ch2.length_counter, 0);
        // Envelope
        ch2.envelope_timer = 1;
        ch2.nr22.write_reg(0b0000_1001); // increasing, period=1
        ch2.envelope_volume = 0x0E;
        ch2.tick_envelope();
        assert_eq!(ch2.envelope_volume, 0x0F);
        ch2.tick_envelope();
        assert_eq!(ch2.envelope_volume, 0x0F); // capped at 0x0F
    }
}
