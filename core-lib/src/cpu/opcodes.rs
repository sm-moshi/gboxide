/// core-lib/src/cpu/opcodes.rs
use super::CPU;
use crate::mmu::MemoryBusTrait;
use once_cell::sync::Lazy;
use pastey::paste;

const REG_INDICES: [(char, usize); 7] = [
    ('b', 0),
    ('c', 1),
    ('d', 2),
    ('e', 3),
    ('h', 4),
    ('l', 5),
    ('a', 7),
];

fn reg_to_index(reg: &str) -> usize {
    let reg_char = reg.chars().next().unwrap();
    REG_INDICES
        .iter()
        .find(|(c, _)| *c == reg_char)
        .map(|(_, idx)| *idx)
        .unwrap_or_else(|| panic!("Invalid register"))
}

#[derive(Clone, Copy)]
pub struct Opcode {
    pub mnemonic: &'static str,
    pub base_cycles: u32,        // Includes fetch cycles
    pub conditional_cycles: u32, // Additional cycles for conditional instructions
    pub exec: fn(&mut CPU, &mut dyn MemoryBusTrait) -> bool, // Returns true if condition met
}

macro_rules! alu_add {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("ADD A, ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: |cpu, _| {
                let lhs = cpu.regs.a;
                let rhs = cpu.regs.$reg;
                println!("ADD A,B: A = {:#04x}, B = {:#04x}", lhs, rhs);
                let result = lhs.wrapping_add(rhs);
                println!("ADD A,B: Result = {:#04x}", result);
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
                println!("ADD A,B: Final A = {:#04x}", cpu.regs.a);
                false
            },
        };
    };
}

macro_rules! alu_sub {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SUB ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
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
                false
            },
        };
    };
}

macro_rules! alu_and {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("AND ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: |cpu, _| {
                cpu.regs.a &= cpu.regs.$reg;
                cpu.regs.f = 0x20; // H
                if cpu.regs.a == 0 {
                    cpu.regs.f |= 0x80;
                } // Z
                false
            },
        };
    };
}

macro_rules! alu_xor {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("XOR ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: |cpu, _| {
                cpu.regs.a ^= cpu.regs.$reg;
                cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
                false
            },
        };
    };
}

macro_rules! alu_or {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("OR ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: |cpu, _| {
                cpu.regs.a |= cpu.regs.$reg;
                cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
                false
            },
        };
    };
}

macro_rules! alu_cp {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CP ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
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
                false
            },
        };
    };
}

macro_rules! inc_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("INC ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
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
                false
            },
        };
    };
}

macro_rules! dec_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("DEC ", stringify!($reg)),
            base_cycles: 4,
            conditional_cycles: 0,
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
                false
            },
        };
    };
}

macro_rules! add_hl_rr {
    ($table:ident, $code:expr, $rr:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("ADD HL, ", stringify!($rr)),
            base_cycles: 8,
            conditional_cycles: 0,
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
                false
            },
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
                exec: |cpu, _| {
                    let val = if stringify!($rr) == "sp" {
                        cpu.regs.sp
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.[<set_ $rr>](val.wrapping_add(1));
                    false
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
                base_cycles: 8,
                conditional_cycles: 0,
                exec: |cpu, _| {
                    let val = if stringify!($rr) == "sp" {
                        cpu.regs.sp
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.[<set_ $rr>](val.wrapping_sub(1));
                    false
                },
            };
        }
    };
}

macro_rules! ld_r_r {
    ($table:ident, $code:expr, $dst:ident, $src:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD ", stringify!($dst), ", ", stringify!($src)),
            base_cycles: 4,
            conditional_cycles: 0,
            exec: |cpu, _| {
                #[allow(clippy::self_assignment)]
                {
                    cpu.regs.$dst = cpu.regs.$src;
                }
                false
            },
        };
    };
}

macro_rules! ld_r_hl {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD ", stringify!($reg), ", (HL)"),
            base_cycles: 8,
            conditional_cycles: 0,
            exec: |cpu, bus| {
                let addr = cpu.regs.hl();
                cpu.regs.$reg = bus.read(addr);
                false
            },
        };
    };
}

macro_rules! ld_hl_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD (HL), ", stringify!($reg)),
            base_cycles: 8,
            conditional_cycles: 0,
            exec: |cpu, bus| {
                let addr = cpu.regs.hl();
                bus.write(addr, cpu.regs.$reg);
                false
            },
        };
    };
}

macro_rules! ld_rr_nn {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("LD ", stringify!($rr), ", nn"),
                base_cycles: 12,
                conditional_cycles: 0,
                exec: |cpu, bus| {
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
                },
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
                exec: |cpu, bus| {
                    let value = if stringify!($rr) == "af" {
                        u16::from_le_bytes([cpu.regs.f, cpu.regs.a])
                    } else {
                        cpu.regs.$rr()
                    };
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    bus.write(cpu.regs.sp, (value >> 8) as u8);
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    bus.write(cpu.regs.sp, value as u8);
                    false
                },
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
                exec: |cpu, bus| {
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
                },
            };
        }
    };
}

macro_rules! jp_cc_nn {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("JP ", $cc, ", nn"),
            base_cycles: 16, // Always takes 16 cycles (4 for opcode fetch + 12 for execution)
            conditional_cycles: 0,
            exec: |cpu, bus| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                if (cpu.regs.f & $flag) == $expected {
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
                false
            },
        };
    };
}

macro_rules! jr_cc_e {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("JR ", $cc, ", e"),
            base_cycles: 12,
            conditional_cycles: 0,
            exec: |cpu, bus| {
                let e = bus.read(cpu.regs.pc) as i8;
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                if (cpu.regs.f & $flag) == $expected {
                    cpu.regs.pc = cpu.regs.pc.wrapping_add(e as u16);
                }
                false
            },
        };
    };
}

macro_rules! call_cc_nn {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CALL ", $cc, ", nn"),
            base_cycles: 24, // Always takes 24 cycles (4 for opcode fetch + 20 for execution)
            conditional_cycles: 0,

            exec: |cpu, bus| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                if (cpu.regs.f & $flag) == $expected {
                    // Push current PC to stack
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
                    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
                    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
                    // Jump to target address
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
                false
            },
        };
    };
}

macro_rules! ret_cc {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RET ", $cc),
            base_cycles: 20, // Always takes 20 cycles (4 for opcode fetch + 16 for execution)
            conditional_cycles: 0,
            exec: |cpu, bus| {
                if (cpu.regs.f & $flag) == $expected {
                    let low = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    let high = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
                false
            },
        };
    };
}

fn rst_common(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait, address: u16) -> bool {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = address;
    false
}

fn rst_00(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x00)
}

fn rst_08(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x08)
}

fn rst_10(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x10)
}

fn rst_18(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x18)
}

fn rst_20(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x20)
}

fn rst_28(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x28)
}

fn rst_30(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x30)
}

fn rst_38(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x38)
}

macro_rules! generate_bit_op {
    ($table:ident, $base:expr, $bit:expr, $reg:expr, $op:ident) => {
        $table[$base + $bit * 8 + reg_to_index($reg)] = Opcode {
            mnemonic: concat!(stringify!($op), " ", stringify!($bit), ",", $reg),
            base_cycles: 8,
            conditional_cycles: 0,
            exec: |cpu, _| {
                let value = cpu.regs.get_reg($reg);
                let mask = 1 << $bit;

                match stringify!($op) {
                    "BIT" => {
                        let result = value & mask != 0;
                        let mut flags = cpu.regs.flags();
                        flags.zero = !result;
                        flags.subtract = false;
                        flags.half_carry = true;
                        cpu.regs.set_flags(flags);
                    }
                    "RES" => {
                        cpu.regs.set_reg($reg, value & !mask);
                    }
                    "SET" => {
                        cpu.regs.set_reg($reg, value | mask);
                    }
                    _ => panic!("Invalid bit operation"),
                }
                false
            },
        };
    };
}

macro_rules! generate_bit_ops_for_reg {
    ($table:ident, $reg:expr) => {
        // Generate BIT operations
        generate_bit_op!($table, 0x40, 0, $reg, BIT);
        generate_bit_op!($table, 0x40, 1, $reg, BIT);
        generate_bit_op!($table, 0x40, 2, $reg, BIT);
        generate_bit_op!($table, 0x40, 3, $reg, BIT);
        generate_bit_op!($table, 0x40, 4, $reg, BIT);
        generate_bit_op!($table, 0x40, 5, $reg, BIT);
        generate_bit_op!($table, 0x40, 6, $reg, BIT);
        generate_bit_op!($table, 0x40, 7, $reg, BIT);

        // Generate RES operations
        generate_bit_op!($table, 0x80, 0, $reg, RES);
        generate_bit_op!($table, 0x80, 1, $reg, RES);
        generate_bit_op!($table, 0x80, 2, $reg, RES);
        generate_bit_op!($table, 0x80, 3, $reg, RES);
        generate_bit_op!($table, 0x80, 4, $reg, RES);
        generate_bit_op!($table, 0x80, 5, $reg, RES);
        generate_bit_op!($table, 0x80, 6, $reg, RES);
        generate_bit_op!($table, 0x80, 7, $reg, RES);

        // Generate SET operations
        generate_bit_op!($table, 0xC0, 0, $reg, SET);
        generate_bit_op!($table, 0xC0, 1, $reg, SET);
        generate_bit_op!($table, 0xC0, 2, $reg, SET);
        generate_bit_op!($table, 0xC0, 3, $reg, SET);
        generate_bit_op!($table, 0xC0, 4, $reg, SET);
        generate_bit_op!($table, 0xC0, 5, $reg, SET);
        generate_bit_op!($table, 0xC0, 6, $reg, SET);
        generate_bit_op!($table, 0xC0, 7, $reg, SET);
    };
}

macro_rules! generate_bit_ops {
    ($table:ident) => {
        generate_bit_ops_for_reg!($table, "b");
        generate_bit_ops_for_reg!($table, "c");
        generate_bit_ops_for_reg!($table, "d");
        generate_bit_ops_for_reg!($table, "e");
        generate_bit_ops_for_reg!($table, "h");
        generate_bit_ops_for_reg!($table, "l");
        generate_bit_ops_for_reg!($table, "a");
    };
}

pub static OPCODES: Lazy<[Opcode; 256]> = Lazy::new(|| {
    let mut table: [Opcode; 256] = [Opcode {
        mnemonic: "UNUSED",
        base_cycles: 0,
        conditional_cycles: 0,
        exec: |_, _| panic!("Unimplemented opcode"),
    }; 256];

    table[0x00] = Opcode {
        mnemonic: "NOP",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |_, _| false,
    };

    // ADD A, B
    table[0x80] = Opcode {
        mnemonic: "ADD A, B",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let b = cpu.regs.b;
            let result = a.wrapping_add(b);

            cpu.regs.f = 0;
            if result == 0 {
                cpu.regs.f |= 0x80;
            } // Z
            if (a & 0xF) + (b & 0xF) > 0xF {
                cpu.regs.f |= 0x20;
            } // H
            if result < a {
                cpu.regs.f |= 0x10;
            } // C

            cpu.regs.a = result;
            false
        },
    };

    // INC B
    table[0x04] = Opcode {
        mnemonic: "INC B",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let val = cpu.regs.b;
            let result = val.wrapping_add(1);

            cpu.regs.f &= 0x10; // Preserve C
            if result == 0 {
                cpu.regs.f |= 0x80;
            } // Z
            if (val & 0x0F) + 1 > 0x0F {
                cpu.regs.f |= 0x20;
            } // H

            cpu.regs.b = result;
            false
        },
    };

    // ADD HL, BC
    table[0x09] = Opcode {
        mnemonic: "ADD HL, BC",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let hl = cpu.regs.hl();
            let bc = cpu.regs.bc();
            let result = hl.wrapping_add(bc);

            cpu.regs.f &= 0x80; // Preserve Z
            cpu.regs.f &= !0x40; // Reset N
            if (hl & 0x0FFF) + (bc & 0x0FFF) > 0x0FFF {
                cpu.regs.f |= 0x20;
            } // H
            if result < hl {
                cpu.regs.f |= 0x10;
            } // C

            cpu.regs.set_hl(result);
            false
        },
    };

    table[0xB8] = Opcode {
        mnemonic: "CP B",
        base_cycles: 4,
        conditional_cycles: 0,
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
            false
        },
    };

    table[0x06] = Opcode {
        mnemonic: "LD B, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.b = n;
            false
        },
    };

    table[0x0E] = Opcode {
        mnemonic: "LD C, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.c = n;
            false
        },
    };

    table[0x16] = Opcode {
        mnemonic: "LD D, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.d = n;
            false
        },
    };

    table[0x1E] = Opcode {
        mnemonic: "LD E, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.e = n;
            false
        },
    };

    table[0x26] = Opcode {
        mnemonic: "LD H, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.h = n;
            false
        },
    };

    table[0x2E] = Opcode {
        mnemonic: "LD L, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.l = n;
            false
        },
    };

    table[0x3E] = Opcode {
        mnemonic: "LD A, n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a = n;
            false
        },
    };

    table[0xC6] = Opcode {
        mnemonic: "ADD A, n",
        base_cycles: 8,
        conditional_cycles: 0,
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
            false
        },
    };

    table[0xD6] = Opcode {
        mnemonic: "SUB n",
        base_cycles: 8,
        conditional_cycles: 0,
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
            false
        },
    };

    table[0xE6] = Opcode {
        mnemonic: "AND n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a &= n;

            cpu.regs.f = 0x20; // H
            if cpu.regs.a == 0 {
                cpu.regs.f |= 0x80;
            } // Z
            false
        },
    };

    table[0xEE] = Opcode {
        mnemonic: "XOR n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a ^= n;

            cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
            false
        },
    };

    table[0xF6] = Opcode {
        mnemonic: "OR n",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.a |= n;

            cpu.regs.f = if cpu.regs.a == 0 { 0x80 } else { 0x00 }; // Z
            false
        },
    };

    table[0xFE] = Opcode {
        mnemonic: "CP n",
        base_cycles: 8,
        conditional_cycles: 0,
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
            false
        },
    };

    table[0x76] = Opcode {
        mnemonic: "HALT",
        base_cycles: 4, // The HALT instruction typically takes 4 cycles
        conditional_cycles: 0,
        exec: |cpu, _| {
            // HALT simply halts the CPU, so this should set the HALT flag or handle the condition
            cpu.halted = true;
            false
        },
    };

    table[0x10] = Opcode {
        mnemonic: "STOP",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            // skip immediate parameter
            bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.stopped = true;
            false
        },
    };

    // ADD A, r
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

    // Register to register loads
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

    // Load from (HL) to register
    ld_r_hl!(table, 0x46, b);
    ld_r_hl!(table, 0x4E, c);
    ld_r_hl!(table, 0x56, d);
    ld_r_hl!(table, 0x5E, e);
    ld_r_hl!(table, 0x66, h);
    ld_r_hl!(table, 0x6E, l);
    ld_r_hl!(table, 0x7E, a);

    // Load register to (HL)
    ld_hl_r!(table, 0x70, b);
    ld_hl_r!(table, 0x71, c);
    ld_hl_r!(table, 0x72, d);
    ld_hl_r!(table, 0x73, e);
    ld_hl_r!(table, 0x74, h);
    ld_hl_r!(table, 0x75, l);
    ld_hl_r!(table, 0x77, a);

    // Load immediate to (HL)
    table[0x36] = Opcode {
        mnemonic: "LD (HL), n",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = cpu.regs.hl();
            bus.write(addr, n);
            false
        },
    };

    // Load A to/from (BC)/(DE)
    table[0x02] = Opcode {
        mnemonic: "LD (BC), A",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            bus.write(cpu.regs.bc(), cpu.regs.a);
            false
        },
    };

    table[0x12] = Opcode {
        mnemonic: "LD (DE), A",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            bus.write(cpu.regs.de(), cpu.regs.a);
            false
        },
    };

    table[0x0A] = Opcode {
        mnemonic: "LD A, (BC)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            cpu.regs.a = bus.read(cpu.regs.bc());
            false
        },
    };

    table[0x1A] = Opcode {
        mnemonic: "LD A, (DE)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            cpu.regs.a = bus.read(cpu.regs.de());
            false
        },
    };

    // Load A to/from direct address
    table[0xFA] = Opcode {
        mnemonic: "LD A, (nn)",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
            let addr = u16::from_le_bytes([low, high]);
            cpu.regs.a = bus.read(addr);
            false
        },
    };

    table[0xEA] = Opcode {
        mnemonic: "LD (nn), A",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
            let addr = u16::from_le_bytes([low, high]);
            bus.write(addr, cpu.regs.a);
            false
        },
    };

    // Load A to/from high memory
    table[0xF0] = Opcode {
        mnemonic: "LDH A, (n)",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let offset = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = 0xFF00 | u16::from(offset);
            cpu.regs.a = bus.read(addr);
            false
        },
    };

    table[0xE0] = Opcode {
        mnemonic: "LDH (n), A",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let offset = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = 0xFF00 | u16::from(offset);
            bus.write(addr, cpu.regs.a);
            false
        },
    };

    // Load A to/from (FF00 + C)
    table[0xF2] = Opcode {
        mnemonic: "LDH A, (C)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let addr = 0xFF00 | u16::from(cpu.regs.c);
            cpu.regs.a = bus.read(addr);
            false
        },
    };

    table[0xE2] = Opcode {
        mnemonic: "LDH (C), A",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let addr = 0xFF00 | u16::from(cpu.regs.c);
            bus.write(addr, cpu.regs.a);
            false
        },
    };

    // Load A to/from (HL) with increment/decrement
    table[0x22] = Opcode {
        mnemonic: "LD (HL+), A",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            bus.write(addr, cpu.regs.a);
            cpu.regs.set_hl(addr.wrapping_add(1));
            false
        },
    };

    table[0x2A] = Opcode {
        mnemonic: "LD A, (HL+)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            cpu.regs.a = bus.read(addr);
            cpu.regs.set_hl(addr.wrapping_add(1));
            false
        },
    };

    table[0x32] = Opcode {
        mnemonic: "LD (HL-), A",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            bus.write(addr, cpu.regs.a);
            cpu.regs.set_hl(addr.wrapping_sub(1));
            false
        },
    };

    table[0x3A] = Opcode {
        mnemonic: "LD A, (HL-)",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            cpu.regs.a = bus.read(addr);
            cpu.regs.set_hl(addr.wrapping_sub(1));
            false
        },
    };

    // 16-bit loads
    ld_rr_nn!(table, 0x01, bc);
    ld_rr_nn!(table, 0x11, de);
    ld_rr_nn!(table, 0x21, hl);
    ld_rr_nn!(table, 0x31, sp);

    // Load SP to memory
    table[0x08] = Opcode {
        mnemonic: "LD (nn), SP",
        base_cycles: 20,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
            let addr = u16::from_le_bytes([low, high]);
            bus.write(addr, (cpu.regs.sp & 0xFF) as u8);
            bus.write(addr.wrapping_add(1), (cpu.regs.sp >> 8) as u8);
            false
        },
    };

    // Load HL to SP
    table[0xF9] = Opcode {
        mnemonic: "LD SP, HL",
        base_cycles: 8,
        conditional_cycles: 0,
        exec: |cpu, _| {
            cpu.regs.sp = cpu.regs.hl();
            false
        },
    };

    // Stack operations
    push_rr!(table, 0xC5, bc);
    push_rr!(table, 0xD5, de);
    push_rr!(table, 0xE5, hl);
    push_rr!(table, 0xF5, af);

    pop_rr!(table, 0xC1, bc);
    pop_rr!(table, 0xD1, de);
    pop_rr!(table, 0xE1, hl);
    pop_rr!(table, 0xF1, af);

    // Unconditional jumps
    table[0xC3] = Opcode {
        mnemonic: "JP nn",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = u16::from_le_bytes([low, high]);
            false
        },
    };

    table[0xE9] = Opcode {
        mnemonic: "JP HL",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            cpu.regs.pc = cpu.regs.hl();
            false
        },
    };

    // Conditional jumps
    jp_cc_nn!(table, 0xC2, "NZ", 0x80, 0x00);
    jp_cc_nn!(table, 0xCA, "Z", 0x80, 0x80);
    jp_cc_nn!(table, 0xD2, "NC", 0x10, 0x00);
    jp_cc_nn!(table, 0xDA, "C", 0x10, 0x10);

    // Relative jumps
    table[0x18] = Opcode {
        mnemonic: "JR e",
        base_cycles: 12,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let e = bus.read(cpu.regs.pc) as i8;
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            if (cpu.regs.f & 0x80) == 0x00 {
                cpu.regs.pc = cpu.regs.pc.wrapping_add(e as u16);
            }
            false
        },
    };

    jr_cc_e!(table, 0x20, "NZ", 0x80, 0x00);
    jr_cc_e!(table, 0x28, "Z", 0x80, 0x80);
    jr_cc_e!(table, 0x30, "NC", 0x10, 0x00);
    jr_cc_e!(table, 0x38, "C", 0x10, 0x10);

    // Call instructions
    table[0xCD] = Opcode {
        mnemonic: "CALL nn",
        base_cycles: 24,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);

            // Push current PC to stack
            cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
            bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
            cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
            bus.write(cpu.regs.sp, cpu.regs.pc as u8);

            // Jump to target address
            cpu.regs.pc = u16::from_le_bytes([low, high]);
            false
        },
    };

    call_cc_nn!(table, 0xC4, "NZ", 0x80, 0x00);
    call_cc_nn!(table, 0xCC, "Z", 0x80, 0x80);
    call_cc_nn!(table, 0xD4, "NC", 0x10, 0x00);
    call_cc_nn!(table, 0xDC, "C", 0x10, 0x10);

    // Return instructions
    table[0xC9] = Opcode {
        mnemonic: "RET",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.sp);
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            let high = bus.read(cpu.regs.sp);
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            cpu.regs.pc = u16::from_le_bytes([low, high]);
            false
        },
    };

    ret_cc!(table, 0xC0, "NZ", 0x80, 0x00);
    ret_cc!(table, 0xC8, "Z", 0x80, 0x80);
    ret_cc!(table, 0xD0, "NC", 0x10, 0x00);
    ret_cc!(table, 0xD8, "C", 0x10, 0x10);

    table[0xD9] = Opcode {
        mnemonic: "RETI",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: |cpu, bus| {
            // pop return address and enable interrupts
            let low = bus.read(cpu.regs.sp);
            let high = bus.read(cpu.regs.sp.wrapping_add(1));
            cpu.regs.sp = cpu.regs.sp.wrapping_add(2);
            cpu.regs.pc = u16::from_le_bytes([low, high]);
            cpu.ime = true;
            false
        },
    };

    // RST instructions
    table[0xC7] = Opcode {
        mnemonic: "RST 00H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_00,
    };

    table[0xCF] = Opcode {
        mnemonic: "RST 08H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_08,
    };

    table[0xD7] = Opcode {
        mnemonic: "RST 10H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_10,
    };

    table[0xDF] = Opcode {
        mnemonic: "RST 18H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_18,
    };

    table[0xE7] = Opcode {
        mnemonic: "RST 20H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_20,
    };

    table[0xEF] = Opcode {
        mnemonic: "RST 28H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_28,
    };

    table[0xF7] = Opcode {
        mnemonic: "RST 30H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_30,
    };

    table[0xFF] = Opcode {
        mnemonic: "RST 38H",
        base_cycles: 16,
        conditional_cycles: 0,
        exec: rst_38,
    };

    // Rotate A left
    table[0x07] = Opcode {
        mnemonic: "RLCA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let carry = (a & 0x80) != 0;
            cpu.regs.a = (a << 1) | u8::from(carry);
            cpu.regs.f = 0;
            if carry {
                cpu.regs.f |= 0x10;
            }
            false
        },
    };

    // Rotate A right
    table[0x0F] = Opcode {
        mnemonic: "RRCA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let carry = (a & 0x01) != 0;
            cpu.regs.a = (a >> 1) | u8::from(carry);
            cpu.regs.f = 0;
            if carry {
                cpu.regs.f |= 0x10;
            }
            false
        },
    };

    // Rotate A left through carry
    table[0x17] = Opcode {
        mnemonic: "RLA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (a & 0x80) != 0;
            cpu.regs.a = (a << 1) | u8::from(old_carry);
            cpu.regs.f = 0;
            if new_carry {
                cpu.regs.f |= 0x10;
            }
            false
        },
    };

    // Rotate A right through carry
    table[0x1F] = Opcode {
        mnemonic: "RRA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (a & 0x01) != 0;
            cpu.regs.a = (a >> 1) | u8::from(old_carry);
            cpu.regs.f = 0;
            if new_carry {
                cpu.regs.f |= 0x10;
            }
            false
        },
    };

    // Decimal adjust A
    table[0x27] = Opcode {
        mnemonic: "DAA",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
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
            false
        },
    };

    // Complement A
    table[0x2F] = Opcode {
        mnemonic: "CPL",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            cpu.regs.a = !cpu.regs.a;
            cpu.regs.f |= 0x60; // Set N and H flags
            false
        },
    };

    // Set carry flag
    table[0x37] = Opcode {
        mnemonic: "SCF",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            cpu.regs.f &= 0x80; // Keep Z flag
            cpu.regs.f |= 0x10; // Set C flag
            false
        },
    };

    // Complement carry flag
    table[0x3F] = Opcode {
        mnemonic: "CCF",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            let carry = cpu.regs.f & 0x10;
            cpu.regs.f &= 0x80; // Keep Z flag
            cpu.regs.f |= carry ^ 0x10; // Toggle C flag
            false
        },
    };

    // Disable interrupts
    table[0xF3] = Opcode {
        mnemonic: "DI",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            cpu.ime = false;
            false
        },
    };

    // Enable interrupts
    table[0xFB] = Opcode {
        mnemonic: "EI",
        base_cycles: 4,
        conditional_cycles: 0,
        exec: |cpu, _| {
            cpu.ime = true;
            false
        },
    };

    table
});

pub static CB_OPCODES: Lazy<[Opcode; 256]> = Lazy::new(|| {
    let mut table: [Opcode; 256] = [Opcode {
        mnemonic: "UNUSED",
        base_cycles: 0,
        conditional_cycles: 0,
        exec: |_, _| panic!("Unimplemented CB opcode"),
    }; 256];

    // RL r - Rotate register left through carry
    table[0x11] = Opcode {
        mnemonic: "RL C",
        base_cycles: 8, // CB prefix instructions take 8 cycles total
        conditional_cycles: 0,
        exec: |cpu, _| {
            let value = cpu.regs.c;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (value & 0x80) != 0;

            cpu.regs.c = (value << 1) | u8::from(old_carry);

            // Update flags
            cpu.regs.f = 0;
            if cpu.regs.c == 0 {
                cpu.regs.f |= 0x80; // Z flag
            }
            if new_carry {
                cpu.regs.f |= 0x10; // C flag
            }
            false
        },
    };

    // Generate all bit operations
    generate_bit_ops!(table);

    table
});
