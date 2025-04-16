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

macro_rules! ld_r_r {
    ($table:ident, $code:expr, $dst:ident, $src:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD ", stringify!($dst), ", ", stringify!($src)),
            cycles: 4,
            exec: |cpu, _| {
                #[allow(clippy::self_assignment)]
                {
                    cpu.regs.$dst = cpu.regs.$src;
                }
            },
        };
    };
}

macro_rules! ld_r_hl {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD ", stringify!($reg), ", (HL)"),
            cycles: 8,
            exec: |cpu, bus| {
                let addr = cpu.regs.hl();
                cpu.regs.$reg = bus.read(addr);
            },
        };
    };
}

macro_rules! ld_hl_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("LD (HL), ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, bus| {
                let addr = cpu.regs.hl();
                bus.write(addr, cpu.regs.$reg);
            },
        };
    };
}

macro_rules! ld_rr_nn {
    ($table:ident, $code:expr, $rr:ident) => {
        paste! {
            $table[$code] = Opcode {
                mnemonic: concat!("LD ", stringify!($rr), ", nn"),
                cycles: 12,
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
                cycles: 16,
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
                cycles: 12,
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
                },
            };
        }
    };
}

macro_rules! jp_cc_nn {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("JP ", $cc, ", nn"),
            cycles: 16, // Always takes 16 cycles (4 for opcode fetch + 12 for execution)
            exec: |cpu, bus| {
                let low = bus.read(cpu.regs.pc);
                let high = bus.read(cpu.regs.pc.wrapping_add(1));
                cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
                if (cpu.regs.f & $flag) == $expected {
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
            },
        };
    };
}

macro_rules! jr_cc_e {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("JR ", $cc, ", e"),
            cycles: 12, // Always takes 12 cycles (4 for opcode fetch + 8 for execution)
            exec: |cpu, bus| {
                let e = bus.read(cpu.regs.pc) as i8;
                cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
                if (cpu.regs.f & $flag) == $expected {
                    cpu.regs.pc = cpu.regs.pc.wrapping_add(e as u16);
                }
            },
        };
    };
}

macro_rules! call_cc_nn {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("CALL ", $cc, ", nn"),
            cycles: 24, // Always takes 24 cycles (4 for opcode fetch + 20 for execution)
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
            },
        };
    };
}

macro_rules! ret_cc {
    ($table:ident, $code:expr, $cc:expr, $flag:expr, $expected:expr) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RET ", $cc),
            cycles: 20, // Always takes 20 cycles (4 for opcode fetch + 16 for execution)
            exec: |cpu, bus| {
                if (cpu.regs.f & $flag) == $expected {
                    let low = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    let high = bus.read(cpu.regs.sp);
                    cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
                    cpu.regs.pc = u16::from_le_bytes([low, high]);
                }
            },
        };
    };
}

fn rst_00(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x00;
}

fn rst_08(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x08;
}

fn rst_10(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x10;
}

fn rst_18(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x18;
}

fn rst_20(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x20;
}

fn rst_28(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x28;
}

fn rst_30(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x30;
}

fn rst_38(cpu: &mut CPU, bus: &mut dyn MemoryBus) {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = 0x38;
}

macro_rules! generate_bit_ops_for_reg {
    ($table:ident, $reg:ident) => {
        paste::paste! {
            // BIT n,r operations (base 0x40)
            {
                const fn bit_op<const BIT: u8>(cpu: &mut CPU, _: &mut dyn MemoryBus) {
                    let mask = 1 << BIT;
                    let value = cpu.regs.$reg;
                    cpu.regs.f = (cpu.regs.f & 0x10) | 0x20;
                    if (value & mask) == 0 { cpu.regs.f |= 0x80; }
                }

                for bit in 0..8 {
                    let base = 0x40 + (bit << 3);
                    $table[base + (stringify!($reg).as_bytes()[0] & 0x07) as usize] = Opcode {
                        mnemonic: concat!("BIT ", stringify!(bit), ",", stringify!($reg)),
                        cycles: 8,
                        exec: match bit {
                            0 => bit_op::<0>,
                            1 => bit_op::<1>,
                            2 => bit_op::<2>,
                            3 => bit_op::<3>,
                            4 => bit_op::<4>,
                            5 => bit_op::<5>,
                            6 => bit_op::<6>,
                            7 => bit_op::<7>,
                            _ => unreachable!(),
                        },
                    };
                }
            }

            // RES n,r operations (base 0x80)
            {
                const fn res_op<const BIT: u8>(cpu: &mut CPU, _: &mut dyn MemoryBus) {
                    let mask = !(1 << BIT);
                    cpu.regs.$reg &= mask;
                }

                for bit in 0..8 {
                    let base = 0x80 + (bit << 3);
                    $table[base + (stringify!($reg).as_bytes()[0] & 0x07) as usize] = Opcode {
                        mnemonic: concat!("RES ", stringify!(bit), ",", stringify!($reg)),
                        cycles: 8,
                        exec: match bit {
                            0 => res_op::<0>,
                            1 => res_op::<1>,
                            2 => res_op::<2>,
                            3 => res_op::<3>,
                            4 => res_op::<4>,
                            5 => res_op::<5>,
                            6 => res_op::<6>,
                            7 => res_op::<7>,
                            _ => unreachable!(),
                        },
                    };
                }
            }

            // SET n,r operations (base 0xC0)
            {
                const fn set_op<const BIT: u8>(cpu: &mut CPU, _: &mut dyn MemoryBus) {
                    let mask = 1 << BIT;
                    cpu.regs.$reg |= mask;
                }

                for bit in 0..8 {
                    let base = 0xC0 + (bit << 3);
                    $table[base + (stringify!($reg).as_bytes()[0] & 0x07) as usize] = Opcode {
                        mnemonic: concat!("SET ", stringify!(bit), ",", stringify!($reg)),
                        cycles: 8,
                        exec: match bit {
                            0 => set_op::<0>,
                            1 => set_op::<1>,
                            2 => set_op::<2>,
                            3 => set_op::<3>,
                            4 => set_op::<4>,
                            5 => set_op::<5>,
                            6 => set_op::<6>,
                            7 => set_op::<7>,
                            _ => unreachable!(),
                        },
                    };
                }
            }
        }
    };
}

macro_rules! rlc_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RLC ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let carry = (value & 0x80) != 0;
                let result = (value << 1) | (if carry { 1 } else { 0 });
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! rrc_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RRC ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let carry = (value & 0x01) != 0;
                let result = (value >> 1) | (if carry { 0x80 } else { 0 });
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! rl_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RL ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let old_carry = (cpu.regs.f & 0x10) != 0;
                let new_carry = (value & 0x80) != 0;
                let result = (value << 1) | (if old_carry { 1 } else { 0 });
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if new_carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! rr_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("RR ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let old_carry = (cpu.regs.f & 0x10) != 0;
                let new_carry = (value & 0x01) != 0;
                let result = (value >> 1) | (if old_carry { 0x80 } else { 0 });
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if new_carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! sla_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SLA ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let carry = (value & 0x80) != 0;
                let result = value << 1;
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! sra_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SRA ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let carry = (value & 0x01) != 0;
                let result = (value >> 1) | (value & 0x80);
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! srl_r {
    ($table:ident, $code:expr, $reg:ident) => {
        $table[$code] = Opcode {
            mnemonic: concat!("SRL ", stringify!($reg)),
            cycles: 8,
            exec: |cpu, _| {
                let value = cpu.regs.$reg;
                let carry = (value & 0x01) != 0;
                let result = value >> 1;
                cpu.regs.$reg = result;
                cpu.regs.f = 0;
                if result == 0 {
                    cpu.regs.f |= 0x80;
                }
                if carry {
                    cpu.regs.f |= 0x10;
                }
            },
        };
    };
}

macro_rules! generate_bit_ops {
    ($table:ident) => {
        // BIT n,r
        generate_bit_ops_for_reg!($table, b);
        generate_bit_ops_for_reg!($table, c);
        generate_bit_ops_for_reg!($table, d);
        generate_bit_ops_for_reg!($table, e);
        generate_bit_ops_for_reg!($table, h);
        generate_bit_ops_for_reg!($table, l);
        generate_bit_ops_for_reg!($table, a);
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
        cycles: 12,
        exec: |cpu, bus| {
            let n = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = cpu.regs.hl();
            bus.write(addr, n);
        },
    };

    // Load A to/from (BC)/(DE)
    table[0x02] = Opcode {
        mnemonic: "LD (BC), A",
        cycles: 8,
        exec: |cpu, bus| {
            bus.write(cpu.regs.bc(), cpu.regs.a);
        },
    };

    table[0x12] = Opcode {
        mnemonic: "LD (DE), A",
        cycles: 8,
        exec: |cpu, bus| {
            bus.write(cpu.regs.de(), cpu.regs.a);
        },
    };

    table[0x0A] = Opcode {
        mnemonic: "LD A, (BC)",
        cycles: 8,
        exec: |cpu, bus| {
            cpu.regs.a = bus.read(cpu.regs.bc());
        },
    };

    table[0x1A] = Opcode {
        mnemonic: "LD A, (DE)",
        cycles: 8,
        exec: |cpu, bus| {
            cpu.regs.a = bus.read(cpu.regs.de());
        },
    };

    // Load A to/from direct address
    table[0xFA] = Opcode {
        mnemonic: "LD A, (nn)",
        cycles: 16,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
            let addr = u16::from_le_bytes([low, high]);
            cpu.regs.a = bus.read(addr);
        },
    };

    table[0xEA] = Opcode {
        mnemonic: "LD (nn), A",
        cycles: 16,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
            let addr = u16::from_le_bytes([low, high]);
            bus.write(addr, cpu.regs.a);
        },
    };

    // Load A to/from high memory
    table[0xF0] = Opcode {
        mnemonic: "LDH A, (n)",
        cycles: 12,
        exec: |cpu, bus| {
            let offset = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = 0xFF00 | u16::from(offset);
            cpu.regs.a = bus.read(addr);
        },
    };

    table[0xE0] = Opcode {
        mnemonic: "LDH (n), A",
        cycles: 12,
        exec: |cpu, bus| {
            let offset = bus.read(cpu.regs.pc);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            let addr = 0xFF00 | u16::from(offset);
            bus.write(addr, cpu.regs.a);
        },
    };

    // Load A to/from (FF00 + C)
    table[0xF2] = Opcode {
        mnemonic: "LDH A, (C)",
        cycles: 8,
        exec: |cpu, bus| {
            let addr = 0xFF00 | u16::from(cpu.regs.c);
            cpu.regs.a = bus.read(addr);
        },
    };

    table[0xE2] = Opcode {
        mnemonic: "LDH (C), A",
        cycles: 8,
        exec: |cpu, bus| {
            let addr = 0xFF00 | u16::from(cpu.regs.c);
            bus.write(addr, cpu.regs.a);
        },
    };

    // Load A to/from (HL) with increment/decrement
    table[0x22] = Opcode {
        mnemonic: "LD (HL+), A",
        cycles: 8,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            bus.write(addr, cpu.regs.a);
            cpu.regs.set_hl(addr.wrapping_add(1));
        },
    };

    table[0x2A] = Opcode {
        mnemonic: "LD A, (HL+)",
        cycles: 8,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            cpu.regs.a = bus.read(addr);
            cpu.regs.set_hl(addr.wrapping_add(1));
        },
    };

    table[0x32] = Opcode {
        mnemonic: "LD (HL-), A",
        cycles: 8,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            bus.write(addr, cpu.regs.a);
            cpu.regs.set_hl(addr.wrapping_sub(1));
        },
    };

    table[0x3A] = Opcode {
        mnemonic: "LD A, (HL-)",
        cycles: 8,
        exec: |cpu, bus| {
            let addr = cpu.regs.hl();
            cpu.regs.a = bus.read(addr);
            cpu.regs.set_hl(addr.wrapping_sub(1));
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
        cycles: 20,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = cpu.regs.pc.wrapping_add(2);
            let addr = u16::from_le_bytes([low, high]);
            bus.write(addr, (cpu.regs.sp & 0xFF) as u8);
            bus.write(addr.wrapping_add(1), (cpu.regs.sp >> 8) as u8);
        },
    };

    // Load HL to SP
    table[0xF9] = Opcode {
        mnemonic: "LD SP, HL",
        cycles: 8,
        exec: |cpu, _| {
            cpu.regs.sp = cpu.regs.hl();
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
        cycles: 16,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.pc);
            let high = bus.read(cpu.regs.pc.wrapping_add(1));
            cpu.regs.pc = u16::from_le_bytes([low, high]);
        },
    };

    table[0xE9] = Opcode {
        mnemonic: "JP HL",
        cycles: 4,
        exec: |cpu, _| {
            cpu.regs.pc = cpu.regs.hl();
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
        cycles: 12,
        exec: |cpu, bus| {
            let e = bus.read(cpu.regs.pc) as i8;
            cpu.regs.pc = cpu.regs.pc.wrapping_add(1);
            cpu.regs.pc = cpu.regs.pc.wrapping_add(e as u16);
        },
    };

    jr_cc_e!(table, 0x20, "NZ", 0x80, 0x00);
    jr_cc_e!(table, 0x28, "Z", 0x80, 0x80);
    jr_cc_e!(table, 0x30, "NC", 0x10, 0x00);
    jr_cc_e!(table, 0x38, "C", 0x10, 0x10);

    // Call instructions
    table[0xCD] = Opcode {
        mnemonic: "CALL nn",
        cycles: 24,
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
        },
    };

    call_cc_nn!(table, 0xC4, "NZ", 0x80, 0x00);
    call_cc_nn!(table, 0xCC, "Z", 0x80, 0x80);
    call_cc_nn!(table, 0xD4, "NC", 0x10, 0x00);
    call_cc_nn!(table, 0xDC, "C", 0x10, 0x10);

    // Return instructions
    table[0xC9] = Opcode {
        mnemonic: "RET",
        cycles: 16,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.sp);
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            let high = bus.read(cpu.regs.sp);
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            cpu.regs.pc = u16::from_le_bytes([low, high]);
        },
    };

    ret_cc!(table, 0xC0, "NZ", 0x80, 0x00);
    ret_cc!(table, 0xC8, "Z", 0x80, 0x80);
    ret_cc!(table, 0xD0, "NC", 0x10, 0x00);
    ret_cc!(table, 0xD8, "C", 0x10, 0x10);

    table[0xD9] = Opcode {
        mnemonic: "RETI",
        cycles: 16,
        exec: |cpu, bus| {
            let low = bus.read(cpu.regs.sp);
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            let high = bus.read(cpu.regs.sp);
            cpu.regs.sp = cpu.regs.sp.wrapping_add(1);
            cpu.regs.pc = u16::from_le_bytes([low, high]);
            cpu.ime = true; // Enable interrupts
        },
    };

    // RST instructions
    table[0xC7] = Opcode {
        mnemonic: "RST 00H",
        cycles: 16,
        exec: rst_00,
    };

    table[0xCF] = Opcode {
        mnemonic: "RST 08H",
        cycles: 16,
        exec: rst_08,
    };

    table[0xD7] = Opcode {
        mnemonic: "RST 10H",
        cycles: 16,
        exec: rst_10,
    };

    table[0xDF] = Opcode {
        mnemonic: "RST 18H",
        cycles: 16,
        exec: rst_18,
    };

    table[0xE7] = Opcode {
        mnemonic: "RST 20H",
        cycles: 16,
        exec: rst_20,
    };

    table[0xEF] = Opcode {
        mnemonic: "RST 28H",
        cycles: 16,
        exec: rst_28,
    };

    table[0xF7] = Opcode {
        mnemonic: "RST 30H",
        cycles: 16,
        exec: rst_30,
    };

    table[0xFF] = Opcode {
        mnemonic: "RST 38H",
        cycles: 16,
        exec: rst_38,
    };

    // Rotate A left
    table[0x07] = Opcode {
        mnemonic: "RLCA",
        cycles: 4,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let carry = (a & 0x80) != 0;
            cpu.regs.a = (a << 1) | (if carry { 1 } else { 0 });
            cpu.regs.f = 0;
            if carry {
                cpu.regs.f |= 0x10;
            }
        },
    };

    // Rotate A right
    table[0x0F] = Opcode {
        mnemonic: "RRCA",
        cycles: 4,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let carry = (a & 0x01) != 0;
            cpu.regs.a = (a >> 1) | (if carry { 0x80 } else { 0 });
            cpu.regs.f = 0;
            if carry {
                cpu.regs.f |= 0x10;
            }
        },
    };

    // Rotate A left through carry
    table[0x17] = Opcode {
        mnemonic: "RLA",
        cycles: 4,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (a & 0x80) != 0;
            cpu.regs.a = (a << 1) | (if old_carry { 1 } else { 0 });
            cpu.regs.f = 0;
            if new_carry {
                cpu.regs.f |= 0x10;
            }
        },
    };

    // Rotate A right through carry
    table[0x1F] = Opcode {
        mnemonic: "RRA",
        cycles: 4,
        exec: |cpu, _| {
            let a = cpu.regs.a;
            let old_carry = (cpu.regs.f & 0x10) != 0;
            let new_carry = (a & 0x01) != 0;
            cpu.regs.a = (a >> 1) | (if old_carry { 0x80 } else { 0 });
            cpu.regs.f = 0;
            if new_carry {
                cpu.regs.f |= 0x10;
            }
        },
    };

    // Decimal adjust A
    table[0x27] = Opcode {
        mnemonic: "DAA",
        cycles: 4,
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
        },
    };

    // Complement A
    table[0x2F] = Opcode {
        mnemonic: "CPL",
        cycles: 4,
        exec: |cpu, _| {
            cpu.regs.a = !cpu.regs.a;
            cpu.regs.f |= 0x60; // Set N and H flags
        },
    };

    // Set carry flag
    table[0x37] = Opcode {
        mnemonic: "SCF",
        cycles: 4,
        exec: |cpu, _| {
            cpu.regs.f &= 0x80; // Keep Z flag
            cpu.regs.f |= 0x10; // Set C flag
        },
    };

    // Complement carry flag
    table[0x3F] = Opcode {
        mnemonic: "CCF",
        cycles: 4,
        exec: |cpu, _| {
            let carry = cpu.regs.f & 0x10;
            cpu.regs.f &= 0x80; // Keep Z flag
            cpu.regs.f |= carry ^ 0x10; // Toggle C flag
        },
    };

    // Disable interrupts
    table[0xF3] = Opcode {
        mnemonic: "DI",
        cycles: 4,
        exec: |cpu, _| {
            cpu.ime = false;
        },
    };

    // Enable interrupts
    table[0xFB] = Opcode {
        mnemonic: "EI",
        cycles: 4,
        exec: |cpu, _| {
            cpu.ime = true;
        },
    };

    // Generate bit operations for all registers
    generate_bit_ops!(table);

    table
});

pub static CB_OPCODES: Lazy<[Opcode; 256]> = Lazy::new(|| {
    let mut table: [Opcode; 256] = [Opcode {
        mnemonic: "UNUSED",
        cycles: 0,
        exec: |_, _| panic!("Unimplemented CB opcode"),
    }; 256];

    // RLC r
    rlc_r!(table, 0x00, b);
    rlc_r!(table, 0x01, c);
    rlc_r!(table, 0x02, d);
    rlc_r!(table, 0x03, e);
    rlc_r!(table, 0x04, h);
    rlc_r!(table, 0x05, l);
    rlc_r!(table, 0x07, a);

    // Generate all bit operations
    generate_bit_ops!(table);

    table
});
