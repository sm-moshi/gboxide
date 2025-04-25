//! Main entry point for the CPU opcodes module.
//!
//! This module re-exports all opcode types, helpers, and submodules for use in the CPU implementation.
//!
//! Modularising opcodes improves clarity, maintainability, and testability.
use once_cell::sync::Lazy;
use pastey::paste;

pub mod alu;
pub mod cb;
pub mod jump;
pub mod load_store;
pub mod types;

pub use alu::*;
pub use cb::CB_OPCODES;
pub use jump::*;
pub use load_store::*;
pub use types::*;

use crate::cpu::CPU;
use crate::mmu::MemoryBusTrait;

/// The main opcode table for the CPU (0x00..=0xFF).
///
/// This table is constructed using macros and logic from the submodules.
pub static OPCODES: Lazy<[Opcode; 256]> = Lazy::new(|| {
    let mut table: [Opcode; 256] = std::array::from_fn(|_| Opcode {
        mnemonic: "UNUSED",
        base_cycles: 0,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            Err(anyhow::anyhow!(
                "Unimplemented opcode: {:02X} at PC={:04X}",
                bus.read(cpu.regs.pc),
                cpu.regs.pc
            ))
        }),
    });

    // --- ALU Macros ---
    alu_add!(table, 0x81, c);
    alu_add!(table, 0x82, d);
    alu_add!(table, 0x83, e);
    alu_add!(table, 0x84, h);
    alu_add!(table, 0x85, l);
    alu_add!(table, 0x87, a);
    alu_sub!(table, 0x90, b);
    alu_sub!(table, 0x91, c);
    alu_sub!(table, 0x92, d);
    alu_sub!(table, 0x93, e);
    alu_sub!(table, 0x94, h);
    alu_sub!(table, 0x95, l);
    alu_sub!(table, 0x97, a);
    alu_and!(table, 0xA0, b);
    alu_and!(table, 0xA1, c);
    alu_and!(table, 0xA2, d);
    alu_and!(table, 0xA3, e);
    alu_and!(table, 0xA4, h);
    alu_and!(table, 0xA5, l);
    alu_and!(table, 0xA7, a);
    alu_xor!(table, 0xA8, b);
    alu_xor!(table, 0xA9, c);
    alu_xor!(table, 0xAA, d);
    alu_xor!(table, 0xAB, e);
    alu_xor!(table, 0xAC, h);
    alu_xor!(table, 0xAD, l);
    alu_xor!(table, 0xAF, a);
    alu_or!(table, 0xB0, b);
    alu_or!(table, 0xB1, c);
    alu_or!(table, 0xB2, d);
    alu_or!(table, 0xB3, e);
    alu_or!(table, 0xB4, h);
    alu_or!(table, 0xB5, l);
    alu_or!(table, 0xB7, a);
    alu_cp!(table, 0xB8, b);
    alu_cp!(table, 0xB9, c);
    alu_cp!(table, 0xBA, d);
    alu_cp!(table, 0xBB, e);
    alu_cp!(table, 0xBC, h);
    alu_cp!(table, 0xBD, l);
    alu_cp!(table, 0xBF, a);
    inc_r!(table, 0x0C, c);
    inc_r!(table, 0x14, d);
    inc_r!(table, 0x1C, e);
    inc_r!(table, 0x24, h);
    inc_r!(table, 0x2C, l);
    inc_r!(table, 0x3C, a);
    dec_r!(table, 0x05, b);
    dec_r!(table, 0x0D, c);
    dec_r!(table, 0x15, d);
    dec_r!(table, 0x1D, e);
    dec_r!(table, 0x25, h);
    dec_r!(table, 0x2D, l);
    dec_r!(table, 0x3D, a);

    // --- Load/Store Macros ---
    ld_r_r!(table, 0x40, b, b);
    ld_r_r!(table, 0x41, b, c);
    ld_r_r!(table, 0x42, b, d);
    ld_r_r!(table, 0x43, b, e);
    ld_r_r!(table, 0x44, b, h);
    ld_r_r!(table, 0x45, b, l);
    ld_r_r!(table, 0x47, b, a);
    ld_r_r!(table, 0x48, c, b);
    ld_r_r!(table, 0x49, c, c);
    ld_r_r!(table, 0x4A, c, d);
    ld_r_r!(table, 0x4B, c, e);
    ld_r_r!(table, 0x4C, c, h);
    ld_r_r!(table, 0x4D, c, l);
    ld_r_r!(table, 0x4F, c, a);
    ld_r_r!(table, 0x50, d, b);
    ld_r_r!(table, 0x51, d, c);
    ld_r_r!(table, 0x52, d, d);
    ld_r_r!(table, 0x53, d, e);
    ld_r_r!(table, 0x54, d, h);
    ld_r_r!(table, 0x55, d, l);
    ld_r_r!(table, 0x57, d, a);
    ld_r_r!(table, 0x58, e, b);
    ld_r_r!(table, 0x59, e, c);
    ld_r_r!(table, 0x5A, e, d);
    ld_r_r!(table, 0x5B, e, e);
    ld_r_r!(table, 0x5C, e, h);
    ld_r_r!(table, 0x5D, e, l);
    ld_r_r!(table, 0x5F, e, a);
    ld_r_r!(table, 0x60, h, b);
    ld_r_r!(table, 0x61, h, c);
    ld_r_r!(table, 0x62, h, d);
    ld_r_r!(table, 0x63, h, e);
    ld_r_r!(table, 0x64, h, h);
    ld_r_r!(table, 0x65, h, l);
    ld_r_r!(table, 0x67, h, a);
    ld_r_r!(table, 0x68, l, b);
    ld_r_r!(table, 0x69, l, c);
    ld_r_r!(table, 0x6A, l, d);
    ld_r_r!(table, 0x6B, l, e);
    ld_r_r!(table, 0x6C, l, h);
    ld_r_r!(table, 0x6D, l, l);
    ld_r_r!(table, 0x6F, l, a);
    ld_r_r!(table, 0x78, a, b);
    ld_r_r!(table, 0x79, a, c);
    ld_r_r!(table, 0x7A, a, d);
    ld_r_r!(table, 0x7B, a, e);
    ld_r_r!(table, 0x7C, a, h);
    ld_r_r!(table, 0x7D, a, l);
    ld_r_r!(table, 0x7F, a, a);
    ld_r_hl!(table, 0x46, b);
    ld_r_hl!(table, 0x4E, c);
    ld_r_hl!(table, 0x56, d);
    ld_r_hl!(table, 0x5E, e);
    ld_r_hl!(table, 0x66, h);
    ld_r_hl!(table, 0x6E, l);
    ld_r_hl!(table, 0x7E, a);
    ld_hl_r!(table, 0x70, b);
    ld_hl_r!(table, 0x71, c);
    ld_hl_r!(table, 0x72, d);
    ld_hl_r!(table, 0x73, e);
    ld_hl_r!(table, 0x74, h);
    ld_hl_r!(table, 0x75, l);
    ld_hl_r!(table, 0x77, a);
    inc_rr!(table, 0x03, bc);
    inc_rr!(table, 0x13, de);
    inc_rr!(table, 0x23, hl);
    inc_rr!(table, 0x33, sp);
    dec_rr!(table, 0x0B, bc);
    dec_rr!(table, 0x1B, de);
    dec_rr!(table, 0x2B, hl);
    dec_rr!(table, 0x3B, sp);
    ld_rr_nn!(table, 0x01, bc);
    ld_rr_nn!(table, 0x11, de);
    ld_rr_nn!(table, 0x21, hl);
    ld_rr_nn!(table, 0x31, sp);
    push_rr!(table, 0xC5, bc);
    push_rr!(table, 0xD5, de);
    push_rr!(table, 0xE5, hl);
    push_rr!(table, 0xF5, af);
    pop_rr!(table, 0xC1, bc);
    pop_rr!(table, 0xD1, de);
    pop_rr!(table, 0xE1, hl);
    pop_rr!(table, 0xF1, af);

    // --- Jump/Call/Return Macros ---
    jp_cc_nn!(table, 0xC2, "NZ", 0x80, 0x00);
    jp_cc_nn!(table, 0xCA, "Z", 0x80, 0x80);
    jp_cc_nn!(table, 0xD2, "NC", 0x10, 0x00);
    jp_cc_nn!(table, 0xDA, "C", 0x10, 0x10);
    jr_cc_e!(table, 0x20, "NZ", 0x80, 0x00);
    jr_cc_e!(table, 0x28, "Z", 0x80, 0x80);
    jr_cc_e!(table, 0x30, "NC", 0x10, 0x00);
    jr_cc_e!(table, 0x38, "C", 0x10, 0x10);
    call_cc_nn!(table, 0xC4, "NZ", 0x80, 0x00);
    call_cc_nn!(table, 0xCC, "Z", 0x80, 0x80);
    call_cc_nn!(table, 0xD4, "NC", 0x10, 0x00);
    call_cc_nn!(table, 0xDC, "C", 0x10, 0x10);
    ret_cc!(table, 0xC0, "NZ", 0x80, 0x00);
    ret_cc!(table, 0xC8, "Z", 0x80, 0x80);
    ret_cc!(table, 0xD0, "NC", 0x10, 0x00);
    ret_cc!(table, 0xD8, "C", 0x10, 0x10);

    // --- Additional Main Opcodes (modularised from legacy) ---
    // Arithmetic 16-bit
    add_hl_rr!(table, 0x09, bc); // ADD HL,BC
    add_hl_rr!(table, 0x19, de); // ADD HL,DE
    add_hl_rr!(table, 0x29, hl); // ADD HL,HL
    add_hl_rr!(table, 0x39, sp); // ADD HL,SP

    // JP/JR/CALL/RET/RETI/RST
    jp_nn!(table, 0xC3); // JP nn
    jp_hl!(table, 0xE9); // JP (HL)
    jr_e!(table, 0x18); // JR e
    call_nn!(table, 0xCD); // CALL nn
    ret!(table, 0xC9); // RET
    reti!(table, 0xD9); // RETI
    rst!(table, 0xC7, 0x00); // RST 00H
    rst!(table, 0xCF, 0x08); // RST 08H
    rst!(table, 0xD7, 0x10); // RST 10H
    rst!(table, 0xDF, 0x18); // RST 18H
    rst!(table, 0xE7, 0x20); // RST 20H
    rst!(table, 0xEF, 0x28); // RST 28H
    rst!(table, 0xF7, 0x30); // RST 30H
    rst!(table, 0xFF, 0x38); // RST 38H

    // Misc control
    di!(table, 0xF3); // DI
    ei!(table, 0xFB); // EI
    halt!(table, 0x76); // HALT
    stop!(table, 0x10); // STOP

    // LD (nn),SP
    ld_nn_sp!(table, 0x08);
    // LD A,(BC), LD A,(DE), LD (BC),A, LD (DE),A
    ld_a_bc!(table, 0x0A);
    ld_a_de!(table, 0x1A);
    ld_bc_a!(table, 0x02);
    ld_de_a!(table, 0x12);
    // LD A,(nn), LD (nn),A
    ld_a_nn!(table, 0xFA);
    ld_nn_a!(table, 0xEA);
    // LD A,(C), LD (C),A
    ld_a_c!(table, 0xF2);
    ld_c_a!(table, 0xE2);
    // LD A,(HLD), LD (HLD),A, LD A,(HLI), LD (HLI),A
    ld_a_hld!(table, 0x3A);
    ld_hld_a!(table, 0x32);
    ld_a_hli!(table, 0x2A);
    ld_hli_a!(table, 0x22);
    // LD SP,HL
    ld_sp_hl!(table, 0xF9);
    // LD HL,SP+e
    ld_hl_sp_e!(table, 0xF8);
    // LD (FF00+n),A; LD A,(FF00+n)
    ld_ff00_n_a!(table, 0xE0);
    ld_a_ff00_n!(table, 0xF0);
    // LD (a16),A; LD A,(a16)
    ld_a_a16!(table, 0xFA);
    ld_a16_a!(table, 0xEA);
    // Immediate ALU/memory variants (ADD A,n; SUB n; AND n; OR n; XOR n; CP n; etc.)
    alu_add_n!(table, 0xC6);
    alu_sub_n!(table, 0xD6);
    alu_and_n!(table, 0xE6);
    alu_or_n!(table, 0xF6);
    alu_xor_n!(table, 0xEE);
    alu_cp_n!(table, 0xFE);
    alu_adc_n!(table, 0xCE);
    alu_sbc_n!(table, 0xDE);

    // --- Direct implementations for missing opcodes ---
    table[0x00] = Opcode {
        mnemonic: "NOP",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|_, _| Ok(false)),
    };
    table[0x04] = Opcode {
        mnemonic: "INC B",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let val = cpu.regs.b;
            let result = val.wrapping_add(1);
            cpu.regs.f &= 0x10; // Preserve C
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (val & 0x0F) + 1 > 0x0F {
                cpu.regs.f |= 0x20;
            }
            cpu.regs.b = result;
            Ok(false)
        }),
    };
    table[0x06] = Opcode {
        mnemonic: "LD B, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.b = n;
            Ok(false)
        }),
    };
    table[0x07] = Opcode {
        mnemonic: "RLCA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let carry = (a & 0x80) != 0;
            cpu.regs.a = (a << 1) | u8::from(carry);
            cpu.regs.f = 0;
            if carry {
                cpu.regs.f |= 0x10;
            }
            Ok(false)
        }),
    };
    table[0x0E] = Opcode {
        mnemonic: "LD C, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.c = n;
            Ok(false)
        }),
    };
    table[0x0F] = Opcode {
        mnemonic: "RRCA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let carry = (a & 0x01) != 0;
            cpu.regs.a = (a >> 1) | u8::from(carry) << 7;
            cpu.regs.f = 0;
            if carry {
                cpu.regs.f |= 0x10;
            }
            Ok(false)
        }),
    };
    table[0x16] = Opcode {
        mnemonic: "LD D, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.d = n;
            Ok(false)
        }),
    };
    table[0x17] = Opcode {
        mnemonic: "RLA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (a & 0x80) != 0;
            cpu.regs.a = (a << 1) | u8::from(old_carry);
            cpu.regs.f = 0;
            if new_carry {
                cpu.regs.f |= 0x10;
            }
            Ok(false)
        }),
    };
    table[0x1E] = Opcode {
        mnemonic: "LD E, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.e = n;
            Ok(false)
        }),
    };
    table[0x1F] = Opcode {
        mnemonic: "RRA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (a & 0x01) != 0;
            cpu.regs.a = (a >> 1) | (u8::from(old_carry) << 7);
            cpu.regs.f = 0;
            if new_carry {
                cpu.regs.f |= 0x10;
            }
            Ok(false)
        }),
    };
    table[0x26] = Opcode {
        mnemonic: "LD H, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.h = n;
            Ok(false)
        }),
    };
    table[0x27] = Opcode {
        mnemonic: "DAA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let mut a = cpu.regs.a;
            let mut adjust = 0;
            let carry = (cpu.regs.f & 0x10) != 0;
            let h_carry = (cpu.regs.f & 0x20) != 0;
            let n_flag = (cpu.regs.f & 0x40) != 0;
            if h_carry || (!n_flag && (a & 0x0F) > 9) {
                adjust |= 0x06;
            }
            if carry || (!n_flag && a > 0x99) {
                adjust |= 0x60;
                cpu.regs.f |= 0x10;
            }
            if n_flag {
                a = a.wrapping_sub(adjust);
            } else {
                a = a.wrapping_add(adjust);
            }
            cpu.regs.f &= 0x70; // Keep N, H, C flags
            if a == 0 {
                cpu.regs.f |= 0x80;
            }
            cpu.regs.a = a;
            Ok(false)
        }),
    };
    table[0x2E] = Opcode {
        mnemonic: "LD L, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.l = n;
            Ok(false)
        }),
    };
    table[0x2F] = Opcode {
        mnemonic: "CPL",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            cpu.regs.a = !cpu.regs.a;
            cpu.regs.f |= 0x60; // Set N and H flags
            Ok(false)
        }),
    };
    table[0x34] = Opcode {
        mnemonic: "INC (HL)",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let addr = cpu.regs.hl();
            let val = bus.read(addr);
            let result = val.wrapping_add(1);
            let mut f = cpu.regs.f & 0x10; // Preserve C
            if result == 0 {
                f |= 0x80;
            }
            if (val & 0x0F) + 1 > 0x0F {
                f |= 0x20;
            }
            cpu.regs.f = f;
            let _ = bus.write(addr, result);
            Ok(false)
        }),
    };
    table[0x35] = Opcode {
        mnemonic: "DEC (HL)",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let addr = cpu.regs.hl();
            let val = bus.read(addr);
            let result = val.wrapping_sub(1);
            let mut f = cpu.regs.f & 0x10; // Preserve C
            f |= 0x40; // N
            if result == 0 {
                f |= 0x80;
            }
            if (val & 0x0F) == 0 {
                f |= 0x20;
            }
            cpu.regs.f = f;
            let _ = bus.write(addr, result);
            Ok(false)
        }),
    };
    table[0x36] = Opcode {
        mnemonic: "LD (HL), n",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = cpu.regs.hl();
            let _ = bus.write(addr, n);
            Ok(false)
        }),
    };
    table[0x37] = Opcode {
        mnemonic: "SCF",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            cpu.regs.f &= 0x80; // Keep Z flag
            cpu.regs.f |= 0x10; // Set C flag
            Ok(false)
        }),
    };
    table[0x3E] = Opcode {
        mnemonic: "LD A, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a = n;
            Ok(false)
        }),
    };
    table[0x3F] = Opcode {
        mnemonic: "CCF",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let carry = cpu.regs.f & 0x10;
            cpu.regs.f &= 0x80; // Keep Z flag
            cpu.regs.f |= carry ^ 0x10; // Toggle C flag
            Ok(false)
        }),
    };
    // ALU and memory opcodes
    alu_add!(table, 0x80, b);
    table[0x86] = Opcode {
        mnemonic: "ADD A, (HL)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let a = cpu.regs.a;
            let v = bus.read(cpu.regs.hl());
            let result = a.wrapping_add(v);
            cpu.regs.f = 0;
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) + (v & 0xF) > 0xF {
                cpu.regs.f |= 0x20;
            }
            if result < a {
                cpu.regs.f |= 0x10;
            }
            cpu.regs.a = result;
            Ok(false)
        }),
    };
    // ADC A, r
    macro_rules! adc_a_r {
        ($code:expr, $reg:ident) => {
            table[$code] = Opcode {
                mnemonic: concat!("ADC A, ", stringify!($reg)),
                base_cycles: 4,
                conditional_cycles: 0,
                exec: Box::new(|cpu, _| {
                    let a = cpu.regs.a;
                    let v = cpu.regs.$reg;
                    let c = u8::from(cpu.regs.f & 0x10 != 0);
                    let result = a.wrapping_add(v).wrapping_add(c);
                    cpu.regs.f = 0;
                    if result == 0 {
                        cpu.regs.f |= 0x80;
                    }
                    if (a & 0xF) + (v & 0xF) + c > 0xF {
                        cpu.regs.f |= 0x20;
                    }
                    if u16::from(a) + u16::from(v) + u16::from(c) > 0xFF {
                        cpu.regs.f |= 0x10;
                    }
                    cpu.regs.a = result;
                    Ok(false)
                }),
            };
        };
    }
    adc_a_r!(0x88, c);
    adc_a_r!(0x89, d);
    adc_a_r!(0x8A, e);
    adc_a_r!(0x8B, h);
    adc_a_r!(0x8C, l);
    adc_a_r!(0x8E, a);
    // SBC A, r
    macro_rules! sbc_a_r {
        ($code:expr, $reg:ident) => {
            table[$code] = Opcode {
                mnemonic: concat!("SBC A, ", stringify!($reg)),
                base_cycles: 4,
                conditional_cycles: 0,
                exec: Box::new(|cpu, _| {
                    let a = cpu.regs.a;
                    let v = cpu.regs.$reg;
                    let c = u8::from(cpu.regs.f & 0x10 != 0);
                    let result = a.wrapping_sub(v).wrapping_sub(c);
                    cpu.regs.f = 0x40;
                    if result == 0 {
                        cpu.regs.f |= 0x80;
                    }
                    if (a & 0xF) < ((v & 0xF) + c) {
                        cpu.regs.f |= 0x20;
                    }
                    if u16::from(v) + u16::from(c) > u16::from(a) {
                        cpu.regs.f |= 0x10;
                    }
                    cpu.regs.a = result;
                    Ok(false)
                }),
            };
        };
    }
    sbc_a_r!(0x98, c);
    sbc_a_r!(0x99, d);
    sbc_a_r!(0x9A, e);
    sbc_a_r!(0x9B, h);
    sbc_a_r!(0x9C, l);
    sbc_a_r!(0x9E, a);
    // AND (HL)
    table[0xA6] = Opcode {
        mnemonic: "AND (HL)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            cpu.regs.a &= bus.read(cpu.regs.hl());
            cpu.regs.f = 0x20;
            if cpu.regs.a == 0 {
                cpu.regs.f |= 0x80;
            }
            Ok(false)
        }),
    };
    // XOR (HL)
    table[0xAE] = Opcode {
        mnemonic: "XOR (HL)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            cpu.regs.a ^= bus.read(cpu.regs.hl());
            cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 };
            Ok(false)
        }),
    };
    // OR (HL)
    table[0xB6] = Opcode {
        mnemonic: "OR (HL)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            cpu.regs.a |= bus.read(cpu.regs.hl());
            cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 };
            Ok(false)
        }),
    };
    // CP (HL)
    table[0xBE] = Opcode {
        mnemonic: "CP (HL)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let a = cpu.regs.a;
            let v = bus.read(cpu.regs.hl());
            cpu.regs.f = 0x40;
            if a == v {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) < (v & 0xF) {
                cpu.regs.f |= 0x20;
            }
            if v > a {
                cpu.regs.f |= 0x10;
            }
            Ok(false)
        }),
    };
    // CB prefix
    table[0xCB] = Opcode {
        mnemonic: "PREFIX CB",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let opcode = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let cb_op = &CB_OPCODES[opcode as usize];
            let _ = (cb_op.exec)(cpu, bus);
            Ok(false)
        }),
    };
    // ADD SP, e
    table[0xE8] = Opcode {
        mnemonic: "ADD SP, e",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
            // These casts are required for hardware-accurate signed offset addition (Game Boy CPU).
            let e = i16::from(bus.read(cpu.regs.pc) as i8);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let sp = cpu.regs.sp;
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let result = (sp as i16).wrapping_add(e) as u16;
            cpu.regs.sp = result;
            cpu.regs.f = 0;
            #[allow(clippy::cast_sign_loss)]
            if ((sp & 0xF) + (e as u16 & 0xF)) > 0xF {
                cpu.regs.f |= 0x20;
            }
            #[allow(clippy::cast_sign_loss)]
            if ((sp & 0xFF) + (e as u16 & 0xFF)) > 0xFF {
                cpu.regs.f |= 0x10;
            }
            Ok(false)
        }),
    };
    // Final missing opcodes
    // ADC A, L
    table[0x8D] = Opcode {
        mnemonic: "ADC A, L",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let v = cpu.regs.l;
            let c = u8::from(cpu.regs.f & 0x10 != 0);
            let result = a.wrapping_add(v).wrapping_add(c);
            cpu.regs.f = 0;
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) + (v & 0xF) + c > 0xF {
                cpu.regs.f |= 0x20;
            }
            if u16::from(a) + u16::from(v) + u16::from(c) > 0xFF {
                cpu.regs.f |= 0x10;
            }
            cpu.regs.a = result;
            Ok(false)
        }),
    };
    // ADC A, A
    table[0x8F] = Opcode {
        mnemonic: "ADC A, A",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let v = cpu.regs.a;
            let c = u8::from(cpu.regs.f & 0x10 != 0);
            let result = a.wrapping_add(v).wrapping_add(c);
            cpu.regs.f = 0;
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) + (v & 0xF) + c > 0xF {
                cpu.regs.f |= 0x20;
            }
            if u16::from(a) + u16::from(v) + u16::from(c) > 0xFF {
                cpu.regs.f |= 0x10;
            }
            cpu.regs.a = result;
            Ok(false)
        }),
    };
    // SUB (HL)
    table[0x96] = Opcode {
        mnemonic: "SUB (HL)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: Box::new(|cpu, bus| {
            let a = cpu.regs.a;
            let v = bus.read(cpu.regs.hl());
            let result = a.wrapping_sub(v);
            cpu.regs.f = 0x40;
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) < (v & 0xF) {
                cpu.regs.f |= 0x20;
            }
            if v > a {
                cpu.regs.f |= 0x10;
            }
            cpu.regs.a = result;
            Ok(false)
        }),
    };
    // SBC A, L
    table[0x9D] = Opcode {
        mnemonic: "SBC A, L",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let v = cpu.regs.l;
            let c = u8::from(cpu.regs.f & 0x10 != 0);
            let result = a.wrapping_sub(v).wrapping_sub(c);
            cpu.regs.f = 0x40;
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) < ((v & 0xF) + c) {
                cpu.regs.f |= 0x20;
            }
            if u16::from(v) + u16::from(c) > u16::from(a) {
                cpu.regs.f |= 0x10;
            }
            cpu.regs.a = result;
            Ok(false)
        }),
    };
    // SBC A, A
    table[0x9F] = Opcode {
        mnemonic: "SBC A, A",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: Box::new(|cpu, _| {
            let a = cpu.regs.a;
            let v = cpu.regs.a;
            let c = u8::from(cpu.regs.f & 0x10 != 0);
            let result = a.wrapping_sub(v).wrapping_sub(c);
            cpu.regs.f = 0x40;
            if result == 0 {
                cpu.regs.f |= 0x80;
            }
            if (a & 0xF) < ((v & 0xF) + c) {
                cpu.regs.f |= 0x20;
            }
            if u16::from(a) + u16::from(v) + u16::from(c) > 0xFF {
                cpu.regs.f |= 0x10;
            }
            cpu.regs.a = result;
            Ok(false)
        }),
    };

    table
});
