/// core-lib/src/cpu/mod.rs
use crate::MemoryBusTrait;
mod opcodes;
pub use opcodes::OPCODES;

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub pc: u16,
    pub sp: u16,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Flags {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Registers {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = val as u8 & 0xF0;
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, val: u16) {
        self.sp = val;
    }

    pub fn get_reg(&self, reg: &str) -> u8 {
        match reg {
            "a" => self.a,
            "f" => self.f,
            "b" => self.b,
            "c" => self.c,
            "d" => self.d,
            "e" => self.e,
            "h" => self.h,
            "l" => self.l,
            _ => panic!("Invalid register name"),
        }
    }

    pub fn set_reg(&mut self, reg: &str, val: u8) {
        match reg {
            "a" => self.a = val,
            "f" => self.f = val & 0xF0, // Only upper 4 bits are used
            "b" => self.b = val,
            "c" => self.c = val,
            "d" => self.d = val,
            "e" => self.e = val,
            "h" => self.h = val,
            "l" => self.l = val,
            _ => panic!("Invalid register name"),
        }
    }

    pub fn flags(&self) -> Flags {
        Flags {
            zero: (self.f & 0x80) != 0,
            subtract: (self.f & 0x40) != 0,
            half_carry: (self.f & 0x20) != 0,
            carry: (self.f & 0x10) != 0,
        }
    }

    pub fn set_flags(&mut self, flags: Flags) {
        self.f = 0;
        if flags.zero {
            self.f |= 0x80;
        }
        if flags.subtract {
            self.f |= 0x40;
        }
        if flags.half_carry {
            self.f |= 0x20;
        }
        if flags.carry {
            self.f |= 0x10;
        }
    }
}

pub struct CPU {
    pub regs: Registers,
    pub ime: bool,
    pub halted: bool,
    pub stopped: bool,
    cycles: u64,         // Total cycles since boot
    current_cycles: u32, // Cycles for current instruction
}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            regs: Registers::default(),
            ime: false,
            halted: false,
            stopped: false,
            cycles: 0,
            current_cycles: 0,
        }
    }

    pub fn step(&mut self, bus: &mut dyn MemoryBusTrait) -> u32 {
        // Reset current instruction cycles
        self.current_cycles = 0;

        // Check for interrupts
        if let Some(interrupt) = bus.get_interrupt() {
            // Exit HALT mode
            self.halted = false;

            // If IME is enabled, handle the interrupt
            if self.ime {
                self.ime = false;

                // Push current PC onto stack (2 M-cycles)
                self.regs.sp = self.regs.sp.wrapping_sub(1);
                bus.write(self.regs.sp, (self.regs.pc >> 8) as u8);
                self.regs.sp = self.regs.sp.wrapping_sub(1);
                bus.write(self.regs.sp, self.regs.pc as u8);

                // Clear the interrupt flag
                bus.clear_interrupt(interrupt);

                // Jump to interrupt vector (1 M-cycle)
                self.regs.pc = bus.get_interrupt_vector(interrupt);

                // Total: 5 M-cycles (20 T-cycles)
                self.current_cycles = 20;
                self.cycles += u64::from(self.current_cycles);
                return self.current_cycles;
            }
        }

        // If halted, consume 4 cycles and return
        if self.halted {
            self.current_cycles = 4;
            self.cycles += u64::from(self.current_cycles);
            return self.current_cycles;
        }

        // Fetch opcode
        let opcode = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);

        #[cfg(debug_assertions)]
        println!("Fetched opcode {:#04x} at PC {:#06x}", opcode, self.regs.pc);
        #[cfg(debug_assertions)]
        println!(
            "Before execution: A = {:#04x}, B = {:#04x}, F = {:#04x}",
            self.regs.a, self.regs.b, self.regs.f
        );

        let op = if opcode == 0xCB {
            // Handle CB prefix opcodes
            let cb_opcode = bus.read(self.regs.pc);
            self.regs.pc = self.regs.pc.wrapping_add(1);
            #[cfg(debug_assertions)]
            println!(
                "Executing CB instruction: {}",
                opcodes::CB_OPCODES[cb_opcode as usize].mnemonic
            );
            &opcodes::CB_OPCODES[cb_opcode as usize]
        } else {
            #[cfg(debug_assertions)]
            println!(
                "Executing instruction: {}",
                opcodes::OPCODES[opcode as usize].mnemonic
            );
            &opcodes::OPCODES[opcode as usize]
        };

        let condition_met = (op.exec)(self, bus);
        self.current_cycles = op.base_cycles;

        // Add conditional cycles if condition was met
        if condition_met {
            self.current_cycles += op.conditional_cycles;
        }

        #[cfg(debug_assertions)]
        println!(
            "After execution: A = {:#04x}, B = {:#04x}, F = {:#04x}",
            self.regs.a, self.regs.b, self.regs.f
        );

        // Update total cycles
        self.cycles += u64::from(self.current_cycles);
        self.current_cycles
    }

    /// Get the total number of cycles since boot
    pub fn get_cycles(&self) -> u64 {
        self.cycles
    }

    /// Get the number of cycles for the current/last instruction
    pub fn get_current_cycles(&self) -> u32 {
        self.current_cycles
    }

    pub fn execute(&mut self, opcode: u8, bus: &mut dyn MemoryBusTrait) -> bool {
        (opcodes::OPCODES[opcode as usize].exec)(self, bus)
    }
}

#[cfg(test)]
mod tests;
