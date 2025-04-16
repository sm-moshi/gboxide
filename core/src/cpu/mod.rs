use crate::bus::MemoryBus;
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
}

pub struct CPU {
    pub regs: Registers,
    pub ime: bool,
    pub halted: bool,
    pub stopped: bool,
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
        }
    }

    pub fn step(&mut self, bus: &mut dyn MemoryBus) -> u8 {
        let opcode = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        let op = &OPCODES[opcode as usize];
        (op.exec)(self, bus);
        op.cycles
    }

    pub fn execute(&mut self, opcode: u8, bus: &mut dyn MemoryBus) {
        (opcodes::OPCODES[opcode as usize].exec)(self, bus)
    }
}

#[cfg(test)]
mod tests;
