/// core-lib/src/mmu/mod.rs
use crate::cartridge::Cartridge;
use crate::interrupts::{InterruptFlag, Interrupts};
use crate::timer::Timer;
use thiserror::Error;

pub mod mbc;
pub use mbc::{Mbc, Mbc1, MbcError, NoMbc};

#[derive(Debug, Error)]
pub enum MmuError {
    #[error("MBC error: {0}")]
    MbcError(#[from] MbcError),
    #[error("Invalid memory access at address: {0:#06X}")]
    InvalidAccess(u16),
    #[error("Cartridge error: {0}")]
    CartridgeError(#[from] crate::cartridge::CartridgeError),
}

/// Memory Bus trait for memory access operations
pub trait MemoryBusTrait {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    /// Get the highest priority pending interrupt
    fn get_interrupt(&self) -> Option<InterruptFlag>;

    /// Clear an interrupt flag after handling
    fn clear_interrupt(&mut self, flag: InterruptFlag);

    /// Get the interrupt vector address for a given interrupt
    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16;
}

/// Memory Management Unit implementation
pub struct MMU {
    mbc: Box<dyn Mbc>,
    vram: [u8; 0x2000],       // 8KB Video RAM
    wram: [u8; 0x2000],       // 8KB Work RAM
    oam: [u8; 0xA0],          // Object Attribute Memory
    io_registers: [u8; 0x80], // Hardware I/O Registers
    hram: [u8; 0x7F],         // High RAM
    ie_register: u8,          // Interrupt Enable Register
    pub interrupts: Interrupts,
    pub timer: Timer,
}

impl MMU {
    #[must_use]
    pub fn new(rom: Vec<u8>) -> Result<Self, MmuError> {
        let cartridge = Cartridge::new(rom)?;
        Ok(Self {
            mbc: cartridge.create_mbc(),
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            ie_register: 0,
            interrupts: Interrupts::new(),
            timer: Timer::new(),
        })
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM Banks
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc.read(addr).unwrap_or(0xFF),

            // VRAM
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],

            // WRAM
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],

            // Echo RAM (mirrors WRAM)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],

            // OAM
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],

            // Timer Registers
            0xFF04..=0xFF07 => self.timer.read(addr),

            // Interrupt Flag
            0xFF0F => self.interrupts.read_if(),

            // I/O Registers
            0xFF00..=0xFF7F => self.io_registers[(addr - 0xFF00) as usize],

            // High RAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],

            // Interrupt Enable Register
            0xFFFF => self.interrupts.read_ie(),

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // ROM Banks and External RAM
            0x0000..=0x7FFF | 0xA000..=0xBFFF => {
                let _ = self.mbc.write(addr, value);
            }

            // VRAM
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,

            // WRAM
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,

            // Echo RAM (mirrors WRAM)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value,

            // OAM
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,

            // Timer Registers
            0xFF04..=0xFF07 => self.timer.write(addr, value),

            // Interrupt Flag
            0xFF0F => self.interrupts.write_if(value),

            // I/O Registers
            0xFF00..=0xFF7F => self.io_registers[(addr - 0xFF00) as usize] = value,

            // High RAM
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,

            // Interrupt Enable Register
            0xFFFF => self.interrupts.write_ie(value),

            _ => {}
        }
    }

    pub fn step(&mut self, cycles: u32) {
        if self.timer.step(cycles) {
            self.interrupts.request(InterruptFlag::Timer);
        }
    }

    #[must_use]
    pub fn save_ram(&self) -> Vec<u8> {
        self.mbc.save_ram()
    }

    /// Load a RAM state
    pub fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError> {
        self.mbc.load_ram(data)
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }
}

impl MemoryBusTrait for MMU {
    fn read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }

    fn get_interrupt(&self) -> Option<InterruptFlag> {
        self.interrupts.get_interrupt()
    }

    fn clear_interrupt(&mut self, flag: InterruptFlag) {
        self.interrupts.clear(flag)
    }

    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16 {
        Interrupts::get_vector(flag)
    }
}

#[cfg(test)]
mod tests;
