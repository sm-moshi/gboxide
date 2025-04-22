//! CB-prefixed opcode macros and implementations.
//!
//! This module defines the logic for all CB-prefixed CPU instructions (bit operations, shifts, rotates, etc.).
//!
//! Keeping CB-prefixed logic separate improves clarity and maintainability.

use super::types::Opcode;
use crate::cpu::CPU;
use crate::mmu::MemoryBusTrait;
use once_cell::sync::Lazy;

// The CB-prefixed opcode table is constructed here.
pub static CB_OPCODES: Lazy<[Opcode; 256]> = Lazy::new(|| {
    let mut table: [Opcode; 256] = std::array::from_fn(|_| Opcode {
        mnemonic: "UNUSED",
        base_cycles: 0,
        conditional_cycles: 0,
        exec: Box::new(|_, _| panic!("Unimplemented CB opcode")),
    });

    // Ported from legacy: generate_cb_ops! macro and CB opcode logic
    let regs = ["b", "c", "d", "e", "h", "l", "hl", "a"];
    // RLC
    for (i, &reg) in regs.iter().enumerate() {
        table[0x00 + i] = Opcode {
            mnemonic: Box::leak(format!("CB RLC {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let res = val.rotate_left(1);
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x80 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // RRC
    for (i, &reg) in regs.iter().enumerate() {
        table[0x08 + i] = Opcode {
            mnemonic: Box::leak(format!("CB RRC {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let res = val.rotate_right(1);
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x01 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // RL
    for (i, &reg) in regs.iter().enumerate() {
        table[0x10 + i] = Opcode {
            mnemonic: Box::leak(format!("CB RL {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let c = if cpu.regs.f & 0x10 != 0 { 1 } else { 0 };
                let res = (val << 1) | c;
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x80 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // RR
    for (i, &reg) in regs.iter().enumerate() {
        table[0x18 + i] = Opcode {
            mnemonic: Box::leak(format!("CB RR {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let c = if cpu.regs.f & 0x10 != 0 { 0x80 } else { 0 };
                let res = (val >> 1) | c;
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x01 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // SLA
    for (i, &reg) in regs.iter().enumerate() {
        table[0x20 + i] = Opcode {
            mnemonic: Box::leak(format!("CB SLA {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let res = val << 1;
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x80 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // SRA
    for (i, &reg) in regs.iter().enumerate() {
        table[0x28 + i] = Opcode {
            mnemonic: Box::leak(format!("CB SRA {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let res = (val >> 1) | (val & 0x80);
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x01 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // SWAP
    for (i, &reg) in regs.iter().enumerate() {
        table[0x30 + i] = Opcode {
            mnemonic: Box::leak(format!("CB SWAP {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let res = val.rotate_left(4);
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // SRL
    for (i, &reg) in regs.iter().enumerate() {
        table[0x38 + i] = Opcode {
            mnemonic: Box::leak(format!("CB SRL {reg}").into_boxed_str()),
            base_cycles: if reg == "hl" { 16 } else { 8 },
            conditional_cycles: 0,
            exec: Box::new(move |cpu, bus| {
                let val = if reg == "hl" {
                    bus.read(cpu.regs.hl())
                } else {
                    cpu.regs.get_reg(reg)
                };
                let res = val >> 1;
                cpu.regs.f = 0;
                if res == 0 {
                    cpu.regs.f |= 0x80;
                }
                if val & 0x01 != 0 {
                    cpu.regs.f |= 0x10;
                }
                if reg == "hl" {
                    let _ = bus.write(cpu.regs.hl(), res);
                } else {
                    cpu.regs.set_reg(reg, res);
                }
                false
            }),
        };
    }
    // BIT, RES, SET
    for bit in 0..8 {
        for (i, &reg) in regs.iter().enumerate() {
            // BIT
            table[0x40 + bit * 8 + i] = Opcode {
                mnemonic: Box::leak(format!("BIT {bit},{reg}").into_boxed_str()),
                base_cycles: if reg == "hl" { 12 } else { 8 },
                conditional_cycles: 0,
                exec: Box::new(move |cpu, bus| {
                    let val = if reg == "hl" {
                        bus.read(cpu.regs.hl())
                    } else {
                        cpu.regs.get_reg(reg)
                    };
                    let mask = 1 << bit;
                    let mut f = cpu.regs.f & 0x10; // preserve C
                    f |= 0x20; // H
                    if val & mask == 0 {
                        f |= 0x80;
                    } // Z
                    cpu.regs.f = f;
                    false
                }),
            };
            // RES
            table[0x80 + bit * 8 + i] = Opcode {
                mnemonic: Box::leak(format!("RES {bit},{reg}").into_boxed_str()),
                base_cycles: if reg == "hl" { 16 } else { 8 },
                conditional_cycles: 0,
                exec: Box::new(move |cpu, bus| {
                    let val = if reg == "hl" {
                        bus.read(cpu.regs.hl())
                    } else {
                        cpu.regs.get_reg(reg)
                    };
                    let res = val & !(1 << bit);
                    if reg == "hl" {
                        let _ = bus.write(cpu.regs.hl(), res);
                    } else {
                        cpu.regs.set_reg(reg, res);
                    }
                    false
                }),
            };
            // SET
            table[0xC0 + bit * 8 + i] = Opcode {
                mnemonic: Box::leak(format!("SET {bit},{reg}").into_boxed_str()),
                base_cycles: if reg == "hl" { 16 } else { 8 },
                conditional_cycles: 0,
                exec: Box::new(move |cpu, bus| {
                    let val = if reg == "hl" {
                        bus.read(cpu.regs.hl())
                    } else {
                        cpu.regs.get_reg(reg)
                    };
                    let res = val | (1 << bit);
                    if reg == "hl" {
                        let _ = bus.write(cpu.regs.hl(), res);
                    } else {
                        cpu.regs.set_reg(reg, res);
                    }
                    false
                }),
            };
        }
    }
    table
});
