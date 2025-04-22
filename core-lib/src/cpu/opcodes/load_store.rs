//! Load/store opcode macros and implementations.
//!
//! This module defines macros and logic for load and store CPU instructions, such as LD, PUSH, POP, INC/DEC 16-bit, etc.
//!
//! Keeping load/store logic separate improves clarity and maintainability.

use super::types::Opcode;
use crate::cpu::CPU;
use crate::mmu::MemoryBusTrait;
use pastey::paste;

macro_rules! ld_r_r {
    ($table:ident, $code:expr, $dst:ident, $src:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD ", stringify!($dst), ", ", stringify!($src)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, _| {
                #[allow(clippy::self_assignment)]
                {
                    cpu.regs.$dst = cpu.regs.$src;
                }
                false
            }),
        };
    };
}

macro_rules! ld_r_hl {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD ", stringify!($reg), ", (HL)"),
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = cpu.regs.hl();
                cpu.regs.$reg = bus.read(addr);
                false
            }),
        };
    };
}

macro_rules! ld_hl_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD (HL), ", stringify!($reg)),
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = cpu.regs.hl();
                let _ = bus.write(addr, cpu.regs.$reg);
                false
            }),
        };
    };
}

macro_rules! inc_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("INC ", stringify!($rr)),
                base_cycles: 8,
                conditional_cycles: 0,
                exec: Box::new(|cpu: &mut CPU, _| {
                    let val = if stringify!($rr) == "sp" {
                        cpu.regs.sp
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.[<set_ $rr>](val.wrapping_add(1));
                    false
                }),
            };
        }
    };
}

macro_rules! dec_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("DEC ", stringify!($rr)),
                base_cycles: 8,
                conditional_cycles: 0,
                exec: Box::new(|cpu: &mut CPU, _| {
                    let val = if stringify!($rr) == "sp" {
                        cpu.regs.sp
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.[<set_ $rr>](val.wrapping_sub(1));
                    false
                }),
            };
        }
    };
}

macro_rules! ld_rr_nn {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("LD ", stringify!($rr), ", nn"),
                base_cycles: 12,
                conditional_cycles: 0,
                exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                    let low = bus.read(cpu.regs.pc);
                    let high = bus.read(cpu.regs.pc.wrapping_add(1));
                    cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                    let value = u16::from_le_bytes([low, high]);
                    if stringify!($rr) == "sp" {
                        cpu.regs.sp = value;
                    } else {
                        cpu.regs.[<set_ $rr>](value);
                    }
                    false
                }),
            };
        }
    };
}

macro_rules! push_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("PUSH ", stringify!($rr)),
                base_cycles: 16,
                conditional_cycles: 0,
                exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                    let value = if stringify!($rr) == "af" {
                        u16::from_le_bytes([cpu.regs.f, cpu.regs.a])
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    let _ = bus.write(cpu.regs.sp, (value >> 8) as u8);
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    let _ = bus.write(cpu.regs.sp, value as u8);
                    false
                }),
            };
        }
    };
}

macro_rules! pop_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("POP ", stringify!($rr)),
                base_cycles: 12,
                conditional_cycles: 0,
                exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                    let low = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    let high = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    let value = u16::from_le_bytes([low, high]);
                    if stringify!($rr) == "af" {
                        cpu.regs.a = high;
                        cpu.regs.f = low & 0xF0; // Lower 4 bits of F are always 0
                    } else {
                        cpu.regs.[<set_ $rr>](value);
                    }
                    false
                }),
            };
        }
    };
}

macro_rules! add_hl_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("ADD HL, ", stringify!($rr)),
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, _| {
                let hl = cpu.regs.hl();
                let rr = if stringify!($rr) == "sp" {
                    cpu.regs.sp
                } else {
                    cpu.regs.$rr()
                };
                let result = hl.wrapping_add(rr);
                cpu.regs.f &= 0x80; // Preserve Z
                cpu.regs.f &= !0x40; // Reset N
                if (hl & 0x0FFF) + (rr & 0x0FFF) > 0x0FFF {
                    cpu.regs.f |= 0x20;
                }
                if result < hl {
                    cpu.regs.f |= 0x10;
                }
                cpu.regs.set_hl(result);
                false
            }),
        };
    };
}

macro_rules! ld_nn_sp {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (nn), SP",
            base_cycles: 20,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                let addr = u16::from_le_bytes([low, high]);
                let _ = bus.write(addr, (cpu.regs.sp & 0xFF) as u8);
                let _ = bus.write(addr.wrapping_add(1), (cpu.regs.sp >> 8) as u8);
                false
            }),
        };
    };
}

macro_rules! ld_a_bc {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD A, (BC)",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                cpu.regs.a = bus.read(cpu.regs.bc());
                false
            }),
        };
    };
}

macro_rules! ld_a_de {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD A, (DE)",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                cpu.regs.a = bus.read(cpu.regs.de());
                false
            }),
        };
    };
}

macro_rules! ld_bc_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (BC), A",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let _ = bus.write(cpu.regs.bc(), cpu.regs.a);
                false
            }),
        };
    };
}

macro_rules! ld_de_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (DE), A",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let _ = bus.write(cpu.regs.de(), cpu.regs.a);
                false
            }),
        };
    };
}

macro_rules! ld_a_nn {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD A, (nn)",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                let addr = u16::from_le_bytes([low, high]);
                cpu.regs.a = bus.read(addr);
                false
            }),
        };
    };
}

macro_rules! ld_nn_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (nn), A",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                let addr = u16::from_le_bytes([low, high]);
                let _ = bus.write(addr, cpu.regs.a);
                false
            }),
        };
    };
}

macro_rules! ld_a_c {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LDH A, (C)",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = 0xFF00 | u16::from(cpu.regs.c);
                cpu.regs.a = bus.read(addr);
                false
            }),
        };
    };
}

macro_rules! ld_c_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LDH (C), A",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = 0xFF00 | u16::from(cpu.regs.c);
                let _ = bus.write(addr, cpu.regs.a);
                false
            }),
        };
    };
}

macro_rules! ld_a_hld {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD A, (HL-)",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = cpu.regs.hl();
                cpu.regs.a = bus.read(addr);
                cpu.regs.set_hl(addr.wrapping_sub(1));
                false
            }),
        };
    };
}

macro_rules! ld_hld_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (HL-), A",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = cpu.regs.hl();
                let _ = bus.write(addr, cpu.regs.a);
                cpu.regs.set_hl(addr.wrapping_sub(1));
                false
            }),
        };
    };
}

macro_rules! ld_a_hli {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD A, (HL+)",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = cpu.regs.hl();
                cpu.regs.a = bus.read(addr);
                cpu.regs.set_hl(addr.wrapping_add(1));
                false
            }),
        };
    };
}

macro_rules! ld_hli_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (HL+), A",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let addr = cpu.regs.hl();
                let _ = bus.write(addr, cpu.regs.a);
                cpu.regs.set_hl(addr.wrapping_add(1));
                false
            }),
        };
    };
}

macro_rules! ld_sp_hl {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD SP, HL",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, _| {
                cpu.regs.sp = cpu.regs.hl();
                false
            }),
        };
    };
}

macro_rules! ld_hl_sp_e {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD HL, SP+e",
            base_cycles: 12,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let e = bus.read(cpu.regs.pc) as i8 as i16 as u16;
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let sp = cpu.regs.sp;
                let result = sp.wrapping_add(e);
                cpu.regs.set_hl(result);
                cpu.regs.f = 0;
                if ((sp & 0xF) + (e & 0xF)) > 0xF {
                    cpu.regs.f |= 0x20;
                }
                if ((sp & 0xFF) + (e & 0xFF)) > 0xFF {
                    cpu.regs.f |= 0x10;
                }
                false
            }),
        };
    };
}

macro_rules! ld_ff00_n_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LDH (n), A",
            base_cycles: 12,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let offset = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let addr = 0xFF00 | u16::from(offset);
                let _ = bus.write(addr, cpu.regs.a);
                false
            }),
        };
    };
}

macro_rules! ld_a_ff00_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LDH A, (n)",
            base_cycles: 12,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let offset = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let addr = 0xFF00 | u16::from(offset);
                cpu.regs.a = bus.read(addr);
                false
            }),
        };
    };
}

macro_rules! ld_a_a16 {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD A, (a16)",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                let addr = u16::from_le_bytes([low, high]);
                cpu.regs.a = bus.read(addr);
                false
            }),
        };
    };
}

macro_rules! ld_a16_a {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "LD (a16), A",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                let addr = u16::from_le_bytes([low, high]);
                let _ = bus.write(addr, cpu.regs.a);
                false
            }),
        };
    };
}

pub(crate) use add_hl_rr;
pub(crate) use dec_rr;
pub(crate) use inc_rr;
pub(crate) use ld_a16_a;
pub(crate) use ld_a_a16;
pub(crate) use ld_a_bc;
pub(crate) use ld_a_c;
pub(crate) use ld_a_de;
pub(crate) use ld_a_ff00_n;
pub(crate) use ld_a_hld;
pub(crate) use ld_a_hli;
pub(crate) use ld_a_nn;
pub(crate) use ld_bc_a;
pub(crate) use ld_c_a;
pub(crate) use ld_de_a;
pub(crate) use ld_ff00_n_a;
pub(crate) use ld_hl_r;
pub(crate) use ld_hl_sp_e;
pub(crate) use ld_hld_a;
pub(crate) use ld_hli_a;
pub(crate) use ld_nn_a;
pub(crate) use ld_nn_sp;
pub(crate) use ld_r_hl;
pub(crate) use ld_r_r;
pub(crate) use ld_rr_nn;
pub(crate) use ld_sp_hl;
pub(crate) use pop_rr;
pub(crate) use push_rr;
