//! Jump, call, and return opcode macros and implementations.
//!
//! This module defines macros and logic for jump, call, and return CPU instructions, including conditional and relative jumps.
//!
//! Keeping jump/call/return logic separate improves clarity and maintainability.

use super::types::Opcode;
use crate::cpu::CPU;
use crate::mmu::MemoryBusTrait;

macro_rules! jp_cc_nn {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("JP ", $cc, ", nn"),
            base_cycles: 16, // Always takes 16 cycles (4 for opcode fetch + 12 for execution)
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                if (cpu.regs.f & $flag) == $expected {
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
                false
            }),
        };
    };
}

macro_rules! jr_cc_e {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("JR ", $cc, ", e"),
            base_cycles: 12,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let e = bus.read(cpu.regs.pc) as i8;
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                if (cpu.regs.f & $flag) == $expected {
                    cpu.regs.pc = cpu.regs.pc.wrapping_add(e as u16);
                }
                false
            }),
        };
    };
}

macro_rules! call_cc_nn {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CALL ", $cc, ", nn"),
            base_cycles: 24, // Always takes 24 cycles (4 for opcode fetch + 20 for execution)
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                if (cpu.regs.f & $flag) == $expected {
                    // Push current PC to stack
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    let _ = bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    let _ = bus.write(cpu.regs.sp, cpu.regs.pc as u8);
                    // Jump to target address
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
                false
            }),
        };
    };
}

macro_rules! ret_cc {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RET ", $cc),
            base_cycles: 20, // Always takes 20 cycles (4 for opcode fetch + 16 for execution)
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                if (cpu.regs.f & $flag) == $expected {
                    let low = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    let high = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
                false
            }),
        };
    };
}

macro_rules! jp_nn {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "JP nn",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = u16::from_le_bytes([low, high]);
                false
            }),
        };
    };
}

macro_rules! jp_hl {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "JP HL",
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, _| {
                cpu.regs.pc = cpu.regs.hl();
                false
            }),
        };
    };
}

macro_rules! jr_e {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "JR e",
            base_cycles: 12,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let e = bus.read(cpu.regs.pc) as i8;
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(e as u16);
                false
            }),
        };
    };
}

macro_rules! call_nn {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "CALL nn",
            base_cycles: 24,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                let _ = bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
                cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                let _ = bus.write(cpu.regs.sp, cpu.regs.pc as u8);
                cpu.regs.pc = u16::from_le_bytes([low, high]);
                false
            }),
        };
    };
}

macro_rules! ret {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "RET",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.sp);
                cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                let high = bus.read(cpu.regs.sp);
                cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                cpu.regs.pc = u16::from_le_bytes([low, high]);
                false
            }),
        };
    };
}

macro_rules! reti {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "RETI",
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                let low = bus.read(cpu.regs.sp);
                let high = bus.read(cpu.regs.sp.wrapping_add(1));
                cpu.regs.sp = cpu.regs.sp.wrapping_add(2);
                cpu.regs.pc = u16::from_le_bytes([low, high]);
                cpu.ime = true;
                false
            }),
        };
    };
}

macro_rules! rst {
    ($table:ident, $code:expr, $addr:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RST ", stringify!($addr), "H"),
            base_cycles: 16,
            conditional_cycles: 0,
            exec: Box::new(|cpu: &mut CPU, bus: &mut dyn MemoryBusTrait| {
                cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                let _ = bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
                cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                let _ = bus.write(cpu.regs.sp, cpu.regs.pc as u8);
                cpu.regs.pc = $addr;
                false
            }),
        };
    };
}

macro_rules! di {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "DI",
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                cpu.ime = false;
                false
            }),
        };
    };
}

macro_rules! ei {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "EI",
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                cpu.ime = true;
                false
            }),
        };
    };
}

macro_rules! halt {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "HALT",
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                cpu.halted = true;
                false
            }),
        };
    };
}

macro_rules! stop {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "STOP",
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                cpu.stopped = true;
                false
            }),
        };
    };
}

pub(crate) use call_cc_nn;
pub(crate) use call_nn;
pub(crate) use di;
pub(crate) use ei;
pub(crate) use halt;
pub(crate) use jp_cc_nn;
pub(crate) use jp_hl;
pub(crate) use jp_nn;
pub(crate) use jr_cc_e;
pub(crate) use jr_e;
pub(crate) use ret;
pub(crate) use ret_cc;
pub(crate) use reti;
pub(crate) use rst;
pub(crate) use stop;
