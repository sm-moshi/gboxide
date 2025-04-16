use crate::{bus::MemoryBus, cpu::CPU};
use once_cell::sync::Lazy;
use paste::paste;

#[derive(Copy, Clone)]
pub struct Opcode {
    pub mnemonic: &'static str,
    pub cycles: u8,
    pub exec: fn(&mut CPU, &mut dyn MemoryBus),
}

macro_rules! alu_add {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("ADD A, ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                let result = lhs.wrapping_add(rhs);
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
                if (lhs & 0xF) + (rhs & 0xF) > 0xF {
                    cpu.regs.f |= 0x20;
                } // H
                if result < lhs {
                    cpu.regs.f |= 0x10;
                } // C
                cpu.regs.a = result;
            },
        };
    };
}

macro_rules! alu_sub {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SUB ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                let result = lhs.wrapping_sub(rhs);
                cpu.regs.f = 0x40; // N
                if result == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
                if (lhs & 0xF) < (rhs & 0xF) {
                    cpu.regs.f |= 0x20;
                } // H
                if rhs > lhs {
                    cpu.regs.f |= 0x10;
                } // C
                cpu.regs.a = result;
            },
        };
    };
}

macro_rules! alu_and {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("AND ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                cpu.regs.a &= cpu.regs.$reg;
                cpu.regs.f = 0x20;
                if cpu.regs.a == 0 {
                    cpu.regs.f |= 0x80;
                }
            },
        };
    };
}

macro_rules! alu_xor {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("XOR ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                cpu.regs.a ^= cpu.regs.$reg;
                cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 };
            },
        };
    };
}

macro_rules! alu_or {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("OR ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                cpu.regs.a |= cpu.regs.$reg;
                cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 };
            },
        };
    };
}

macro_rules! alu_cp {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CP ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                cpu.regs.f = 0x40; // N
                if lhs == rhs {
                    cpu.regs.f |= 0x80;
                } // Z
                if (lhs & 0xF) < (rhs & 0xF) {
                    cpu.regs.f |= 0x20;
                } // H
                if rhs > lhs {
                    cpu.regs.f |= 0x10;
                } // C
            },
        };
    };
}

macro_rules! alu_sub {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SUB ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                let result = lhs.wrapping_sub(rhs);

                cpu.regs.f = 0x40; // N
                if result == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
                if (lhs & 0xF) < (rhs & 0xF) {
                    cpu.regs.f |= 0x20;
                } // H
                if rhs > lhs {
                    cpu.regs.f |= 0x10;
                } // C

                cpu.regs.a = result;
            },
        };
    };
}

macro_rules! alu_and {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("AND ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                cpu.regs.a &= cpu.regs.$reg;
                cpu.regs.f = 0x20; // H
                if cpu.regs.a == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
            },
        };
    };
}

macro_rules! alu_xor {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("XOR ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                cpu.regs.a ^= cpu.regs.$reg;
                cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
            },
        };
    };
}

macro_rules! alu_or {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("OR ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                cpu.regs.a |= cpu.regs.$reg;
                cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
            },
        };
    };
}

macro_rules! alu_cp {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CP ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                cpu.regs.f = 0x40; // N
                if lhs == rhs {
                    cpu.regs.f |= 0x80;
                } // Z
                if (lhs & 0xF) < (rhs & 0xF) {
                    cpu.regs.f |= 0x20;
                } // H
                if rhs > lhs {
                    cpu.regs.f |= 0x10;
                } // C
            },
        };
    };
}

macro_rules! inc_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("INC ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let val = cpu.regs.$reg;
                let result = val.wrapping_add(1);

                cpu.regs.f &= 0x10; // Preserve C
                if result == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
                if (val & 0x0F) + 1 > 0x0F {
                    cpu.regs.f |= 0x20;
                } // H

                cpu.regs.$reg = result;
            },
        };
    };
}

macro_rules! dec_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("DEC ", stringify!($reg)),
            cycles: 4,
            exec: |cpu, _| {
                let val = cpu.regs.$reg;
                let result = val.wrapping_sub(1);

                cpu.regs.f &= 0x10; // Preserve C
                cpu.regs.f |= 0x40; // N
                if result == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
                if (val & 0x0F) == 0x00 {
                    cpu.regs.f |= 0x20;
                } // H

                cpu.regs.$reg = result;
            },
        };
    };
}

macro_rules! add_hl_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("ADD HL, ", stringify!($rr)),
            cycles: 8,
            exec: |cpu, _| {
                let hl = cpu.regs.hl();
                let rr = if stringify!($rr) == "sp" {
                    cpu.regs.sp
                } else {
                    cpu.regs.$rr()
                };

                // Preserve Z flag
                let old_z = cpu.regs.f & 0x80;

                // Calculate result and flags
                let (result, carry) = hl.overflowing_add(rr);
                let h_carry = (hl & 0x0FFF) + (rr & 0x0FFF) > 0x0FFF;

                // Set flags:
                // - Z is preserved (handled above)
                // - N is reset
                // - H is set if carry from bit 11
                // - C is set if carry from bit 15
                cpu.regs.f = old_z | // Preserved Z flag
                            (if h_carry { 0x20 } else { 0 }) | // H flag
                            (if carry { 0x10 } else { 0 }); // C flag

                cpu.regs.set_hl(result);

                // Debug output
                println!(
                    "ADD HL, {}: HL = {:#06X}, RR = {:#06X}, Result = {:#06X}, Carry = {}, H_Carry = {}, F = {:#04X}",
                    stringify!($rr),
                    hl,
                    rr,
                    result,
                    carry,
                    h_carry,
                    cpu.regs.f
                );
            },
        };
    };
}

macro_rules! inc_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("INC ", stringify!($rr)),
                cycles: 8,
                exec: |cpu, _| {
                    let val = if stringify!($rr) == "sp" {
                        cpu.regs.sp
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.[<set_ $rr>](val.wrapping_add(1));
                },
            };
        }
    };
}

macro_rules! dec_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("DEC ", stringify!($rr)),
                cycles: 8,
                exec: |cpu, _| {
                    let val = if stringify!($rr) == "sp" {
                        cpu.regs.sp
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.[<set_ $rr>](val.wrapping_sub(1));
                },
            };
        }
    };
}

pub static OPCODES: Lazy<[Opcode; 256]> = Lazy::new(|| {
    let mut table: [Opcode; 256] = [Opcode {
        mnemonic: "UNUSED",
        cycles: 0,
        exec: |_, _| panic!("Unimplemented opcode"),
    }; 256];

    table[0x00] = Opcode {
        mnemonic: "NOP",
        cycles: 4,
        exec: |_, _| {},
    };

    table[0xB8] = Opcode {
        mnemonic: "CP B",
        cycles: 4,
        exec: |cpu, _| {
            let lhs = cpu.regs.a;
            let rhs = cpu.regs.b;
            cpu.regs.f = 0x40;
            if lhs == rhs {
                cpu.regs.f |= 0x80;
            }
            if (lhs & 0xF) < (rhs & 0xF) {
                cpu.regs.f |= 0x20;
            }
            if rhs > lhs {
                cpu.regs.f |= 0x10;
            }
        },
    };
    // Repeat 0xB9–0xBF for CP C–A

    table[0x06] = Opcode {
        mnemonic: "LD B, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.b = n;
        },
    };

    table[0x0E] = Opcode {
        mnemonic: "LD C, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.c = n;
        },
    };

    table[0x16] = Opcode {
        mnemonic: "LD D, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.d = n;
        },
    };

    table[0x1E] = Opcode {
        mnemonic: "LD E, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.e = n;
        },
    };

    table[0x26] = Opcode {
        mnemonic: "LD H, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.h = n;
        },
    };

    table[0x2E] = Opcode {
        mnemonic: "LD L, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.l = n;
        },
    };

    table[0x3E] = Opcode {
        mnemonic: "LD A, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a = n;
        },
    };

    table[0xC6] = Opcode {
        mnemonic: "ADD A, n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);

            let a = cpu.regs.a;
            let result = a.wrapping_add(n);

            cpu.regs.f = 0;
            if result == 0 {
                cpu.regs.f |= 0x80;
            } // Z
            if (a & 0xF) + (n & 0xF) > 0xF {
                cpu.regs.f |= 0x20;
            } // H
            if result < a {
                cpu.regs.f |= 0x10;
            } // C

            cpu.regs.a = result;
        },
    };

    table[0xD6] = Opcode {
        mnemonic: "SUB n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);

            let a = cpu.regs.a;
            let result = a.wrapping_sub(n);

            cpu.regs.f = 0x40; // N
            if result == 0 {
                cpu.regs.f |= 0x80;
            } // Z
            if (a & 0xF) < (n & 0xF) {
                cpu.regs.f |= 0x20;
            } // H
            if n > a {
                cpu.regs.f |= 0x10;
            } // C

            cpu.regs.a = result;
        },
    };

    table[0xE6] = Opcode {
        mnemonic: "AND n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a &= n;

            cpu.regs.f = 0x20; // H
            if cpu.regs.a == 0 {
                cpu.regs.f |= 0x80;
            } // Z
        },
    };

    table[0xEE] = Opcode {
        mnemonic: "XOR n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a ^= n;

            cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
        },
    };

    table[0xF6] = Opcode {
        mnemonic: "OR n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a |= n;

            cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
        },
    };

    table[0xFE] = Opcode {
        mnemonic: "CP n",
        cycles: 8,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let a = cpu.regs.a;

            cpu.regs.f = 0x40; // N
            if a == n {
                cpu.regs.f |= 0x80;
            } // Z
            if (a & 0xF) < (n & 0xF) {
                cpu.regs.f |= 0x20;
            } // H
            if n > a {
                cpu.regs.f |= 0x10;
            } // C
        },
    };

    table[0x76] = Opcode {
        mnemonic: "HALT",
        cycles: 4, // The HALT instruction typically takes 4 cycles
        exec: |cpu, _| {
            // HALT simply halts the CPU, so this should set the HALT flag or handle the condition
            cpu.halted = true;
        },
    };

    // ADD A, r
    alu_add!(table, 0x80, b);
    alu_add!(table, 0x81, c);
    alu_add!(table, 0x82, d);
    alu_add!(table, 0x83, e);
    alu_add!(table, 0x84, h);
    alu_add!(table, 0x85, l);
    alu_add!(table, 0x87, a);

    // SUB r
    alu_sub!(table, 0x90, b);
    alu_sub!(table, 0x91, c);
    alu_sub!(table, 0x92, d);
    alu_sub!(table, 0x93, e);
    alu_sub!(table, 0x94, h);
    alu_sub!(table, 0x95, l);
    alu_sub!(table, 0x97, a);

    // AND r
    alu_and!(table, 0xA0, b);
    alu_and!(table, 0xA1, c);
    alu_and!(table, 0xA2, d);
    alu_and!(table, 0xA3, e);
    alu_and!(table, 0xA4, h);
    alu_and!(table, 0xA5, l);
    alu_and!(table, 0xA7, a);

    // XOR r
    alu_xor!(table, 0xA8, b);
    alu_xor!(table, 0xA9, c);
    alu_xor!(table, 0xAA, d);
    alu_xor!(table, 0xAB, e);
    alu_xor!(table, 0xAC, h);
    alu_xor!(table, 0xAD, l);
    alu_xor!(table, 0xAF, a);

    // OR r
    alu_or!(table, 0xB0, b);
    alu_or!(table, 0xB1, c);
    alu_or!(table, 0xB2, d);
    alu_or!(table, 0xB3, e);
    alu_or!(table, 0xB4, h);
    alu_or!(table, 0xB5, l);
    alu_or!(table, 0xB7, a);

    // CP r
    alu_cp!(table, 0xB8, b);
    alu_cp!(table, 0xB9, c);
    alu_cp!(table, 0xBA, d);
    alu_cp!(table, 0xBB, e);
    alu_cp!(table, 0xBC, h);
    alu_cp!(table, 0xBD, l);
    alu_cp!(table, 0xBF, a);

    // INC r
    inc_r!(table, 0x04, b);
    inc_r!(table, 0x0C, c);
    inc_r!(table, 0x14, d);
    inc_r!(table, 0x1C, e);
    inc_r!(table, 0x24, h);
    inc_r!(table, 0x2C, l);
    inc_r!(table, 0x3C, a);

    // DEC r
    dec_r!(table, 0x05, b);
    dec_r!(table, 0x0D, c);
    dec_r!(table, 0x15, d);
    dec_r!(table, 0x1D, e);
    dec_r!(table, 0x25, h);
    dec_r!(table, 0x2D, l);
    dec_r!(table, 0x3D, a);

    // ADD A, r
    alu_add!(table, 0x80, b);
    alu_add!(table, 0x81, c);
    alu_add!(table, 0x82, d);
    alu_add!(table, 0x83, e);
    alu_add!(table, 0x84, h);
    alu_add!(table, 0x85, l);
    alu_add!(table, 0x87, a);

    // SUB r
    alu_sub!(table, 0x90, b);
    alu_sub!(table, 0x91, c);
    alu_sub!(table, 0x92, d);
    alu_sub!(table, 0x93, e);
    alu_sub!(table, 0x94, h);
    alu_sub!(table, 0x95, l);
    alu_sub!(table, 0x97, a);

    // AND r
    alu_and!(table, 0xA0, b);
    alu_and!(table, 0xA1, c);
    alu_and!(table, 0xA2, d);
    alu_and!(table, 0xA3, e);
    alu_and!(table, 0xA4, h);
    alu_and!(table, 0xA5, l);
    alu_and!(table, 0xA7, a);

    // XOR r
    alu_xor!(table, 0xA8, b);
    alu_xor!(table, 0xA9, c);
    alu_xor!(table, 0xAA, d);
    alu_xor!(table, 0xAB, e);
    alu_xor!(table, 0xAC, h);
    alu_xor!(table, 0xAD, l);
    alu_xor!(table, 0xAF, a);

    // OR r
    alu_or!(table, 0xB0, b);
    alu_or!(table, 0xB1, c);
    alu_or!(table, 0xB2, d);
    alu_or!(table, 0xB3, e);
    alu_or!(table, 0xB4, h);
    alu_or!(table, 0xB5, l);
    alu_or!(table, 0xB7, a);

    // CP r
    alu_cp!(table, 0xB8, b);
    alu_cp!(table, 0xB9, c);
    alu_cp!(table, 0xBA, d);
    alu_cp!(table, 0xBB, e);
    alu_cp!(table, 0xBC, h);
    alu_cp!(table, 0xBD, l);
    alu_cp!(table, 0xBF, a);

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

    // INC r
    inc_r!(table, 0x04, b);
    inc_r!(table, 0x0C, c);
    inc_r!(table, 0x14, d);
    inc_r!(table, 0x1C, e);
    inc_r!(table, 0x24, h);
    inc_r!(table, 0x2C, l);
    inc_r!(table, 0x3C, a);

    // DEC r
    dec_r!(table, 0x05, b);
    dec_r!(table, 0x0D, c);
    dec_r!(table, 0x15, d);
    dec_r!(table, 0x1D, e);
    dec_r!(table, 0x25, h);
    dec_r!(table, 0x2D, l);
    dec_r!(table, 0x3D, a);

    add_hl_rr!(table, 0x09, bc);
    add_hl_rr!(table, 0x19, de);
    add_hl_rr!(table, 0x29, hl);
    add_hl_rr!(table, 0x39, sp);

    //
    inc_rr!(table, 0x03, bc);
    inc_rr!(table, 0x13, de);
    inc_rr!(table, 0x23, hl);
    inc_rr!(table, 0x33, sp); // sp = sp + 1

    dec_rr!(table, 0x0B, bc);
    dec_rr!(table, 0x1B, de);
    dec_rr!(table, 0x2B, hl);
    dec_rr!(table, 0x3B, sp); // sp = sp - 1

    table
});
