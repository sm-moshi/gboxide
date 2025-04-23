//! Arithmetic and logic unit (ALU) opcode macros and implementations.
//!
//! This module defines macros and logic for ALU-related CPU instructions, such as ADD, SUB, AND, OR, XOR, CP, INC, and DEC.
//!
//! Keeping ALU logic separate improves clarity and maintainability.

// use super::types::Opcode;
// use crate::cpu::CPU;
// use crate::mmu::MemoryBusTrait;

#[macro_export]
macro_rules! alu_add {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("ADD A, ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                let result = lhs.wrapping_add(rhs);
                // Flag logic: Z if result is zero, H if lower nibble overflow, C if full overflow
                cpu.regs.f = 0
                    | (result == 0).then_some(0x80).unwrap_or(0) // Z
                    | (((lhs & 0xF) + (rhs & 0xF)) > 0xF).then_some(0x20).unwrap_or(0) // H
                    | (result < lhs).then_some(0x10).unwrap_or(0); // C
                cpu.regs.a = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_sub {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SUB ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                let result = lhs.wrapping_sub(rhs);
                cpu.regs.f = 0x40 // N
                    | (result == 0).then_some(0x80).unwrap_or(0) // Z
                    | ((lhs & 0xF) < (rhs & 0xF)).then_some(0x20).unwrap_or(0) // H
                    | (rhs > lhs).then_some(0x10).unwrap_or(0); // C
                cpu.regs.a = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_and {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("AND ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                cpu.regs.a &= cpu.regs.$reg;
                cpu.regs.f = 0x20 // H
                    | (cpu.regs.a == 0).then_some(0x80).unwrap_or(0); // Z
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_xor {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("XOR ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                cpu.regs.a ^= cpu.regs.$reg;
                cpu.regs.f = (cpu.regs.a == 0).then_some(0x80).unwrap_or(0); // Z
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_or {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("OR ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                cpu.regs.a |= cpu.regs.$reg;
                cpu.regs.f = (cpu.regs.a == 0).then_some(0x80).unwrap_or(0); // Z
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_cp {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CP ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                // Flag logic: Z if equal, N always set, H if lower nibble borrow, C if full borrow
                cpu.regs.f = 0x40 // N
                    | (lhs == rhs).then_some(0x80).unwrap_or(0) // Z
                    | ((lhs & 0xF) < (rhs & 0xF)).then_some(0x20).unwrap_or(0) // H
                    | (rhs > lhs).then_some(0x10).unwrap_or(0); // C
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! inc_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("INC ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                let val = cpu.regs.$reg;
                let result = val.wrapping_add(1);
                cpu.regs.f = (cpu.regs.f & 0x10) // Preserve C
                    | (result == 0).then_some(0x80).unwrap_or(0) // Z
                    | ((val & 0x0F) + 1 > 0x0F).then_some(0x20).unwrap_or(0); // H
                cpu.regs.$reg = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! dec_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("DEC ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: Box::new(|cpu, _| {
                let val = cpu.regs.$reg;
                let result = val.wrapping_sub(1);
                cpu.regs.f = (cpu.regs.f & 0x10) // Preserve C
                    | 0x40 // N
                    | (result == 0).then_some(0x80).unwrap_or(0) // Z
                    | ((val & 0x0F) == 0).then_some(0x20).unwrap_or(0); // H
                cpu.regs.$reg = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_add_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "ADD A, n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let a = cpu.regs.a;
                let result = a.wrapping_add(n);
                cpu.regs.f = 0
                    | (result == 0).then_some(0x80).unwrap_or(0)
                    | (((a & 0xF) + (n & 0xF)) > 0xF).then_some(0x20).unwrap_or(0)
                    | (result < a).then_some(0x10).unwrap_or(0);
                cpu.regs.a = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_sub_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "SUB n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let a = cpu.regs.a;
                let result = a.wrapping_sub(n);
                cpu.regs.f = 0x40 // N
                    | (result == 0).then_some(0x80).unwrap_or(0)
                    | ((a & 0xF) < (n & 0xF)).then_some(0x20).unwrap_or(0)
                    | (n > a).then_some(0x10).unwrap_or(0);
                cpu.regs.a = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_and_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "AND n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                cpu.regs.a &= n;
                cpu.regs.f = 0x20 // H
                    | (cpu.regs.a == 0).then_some(0x80).unwrap_or(0);
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_or_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "OR n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                cpu.regs.a |= n;
                cpu.regs.f = (cpu.regs.a == 0).then_some(0x80).unwrap_or(0);
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_xor_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "XOR n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                cpu.regs.a ^= n;
                cpu.regs.f = (cpu.regs.a == 0).then_some(0x80).unwrap_or(0);
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_cp_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "CP n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let a = cpu.regs.a;
                cpu.regs.f = 0x40;
                if a == n {
                    cpu.regs.f |= 0x80;
                }
                if (a & 0xF) < (n & 0xF) {
                    cpu.regs.f |= 0x20;
                }
                if n > a {
                    cpu.regs.f |= 0x10;
                }
                println!(
                    "[DEBUG][CP n] A={:02X} n={:02X} F={:02X} Z={} N={} H={} C={}",
                    a,
                    n,
                    cpu.regs.f,
                    (cpu.regs.f & 0x80) != 0,
                    (cpu.regs.f & 0x40) != 0,
                    (cpu.regs.f & 0x20) != 0,
                    (cpu.regs.f & 0x10) != 0,
                );
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_adc_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "ADC A, n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let a = cpu.regs.a;
                let c = u8::from(cpu.regs.f & 0x10 != 0);
                let result = a.wrapping_add(n).wrapping_add(c);
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if (a & 0xF) + (n & 0xF) + c > 0xF {
                    cpu.regs.f |= 0x20;
                }
                if u16::from(a) + u16::from(n) + u16::from(c) > 0xFF {
                    cpu.regs.f |= 0x10;
                }
                cpu.regs.a = result;
                false
            }),
        };
    };
}

#[macro_export]
macro_rules! alu_sbc_n {
    ($table:ident, $code:expr) => {
        $table[$code] = Opcode {
            mnemonic: "SBC A, n",
            base_cycles: 8,
            conditional_cycles: 0,
            exec: Box::new(|cpu, bus| {
                let n = bus.read(cpu.regs.pc);
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                let a = cpu.regs.a;
                let c = u8::from(cpu.regs.f & 0x10 != 0);
                let result = a.wrapping_sub(n).wrapping_sub(c);
                cpu.regs.f = 0x40;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if (a & 0xF) < ((n & 0xF) + c) {
                    cpu.regs.f |= 0x20;
                }
                if u16::from(n) + u16::from(c) > u16::from(a) {
                    cpu.regs.f |= 0x10;
                }
                cpu.regs.a = result;
                false
            }),
        };
    };
}

pub use alu_adc_n;
pub use alu_add;
pub use alu_add_n;
pub use alu_and;
pub use alu_and_n;
pub use alu_cp;
pub use alu_cp_n;
pub use alu_or;
pub use alu_or_n;
pub use alu_sbc_n;
pub use alu_sub;
pub use alu_sub_n;
pub use alu_xor;
pub use alu_xor_n;
pub use dec_r;
pub use inc_r;
