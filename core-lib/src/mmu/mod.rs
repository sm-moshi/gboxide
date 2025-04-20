/// core-lib/src/mmu/mod.rs
use crate::cartridge::Cartridge;
use crate::interrupts::{InterruptFlag, Interrupts};
use crate::ppu::{Ppu, PpuMode};
use crate::timer::{Timer, TimerError, TimerState};
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
    #[error("Timer error: {0}")]
    TimerError(#[from] TimerError),
}

/// Memory Bus trait for memory access operations
pub trait MemoryBusTrait {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MmuError>;

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
    pub interrupts: Interrupts,
    pub timer: Timer,
    pub ppu: Ppu,        // Pixel Processing Unit
    dma_active: bool,    // Whether DMA is currently active
    dma_start_addr: u16, // Source address for DMA
    dma_cycles: u32,     // Remaining cycles for DMA
    dma_byte: u8,        // Current byte being transferred in DMA
}

#[allow(clippy::cast_possible_truncation)] // Intentional truncation for DMA cycles and memory addressing
impl MMU {
    #[must_use = "Returns a new MMU instance or an error if initialization fails"]
    pub fn new(rom: Vec<u8>) -> Result<Self, MmuError> {
        let cartridge = Cartridge::new(rom)?;
        Ok(Self {
            mbc: cartridge.create_mbc(),
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupts: Interrupts::new(),
            timer: Timer::new(),
            ppu: Ppu::new(),
            dma_active: false,
            dma_start_addr: 0,
            dma_cycles: 0,
            dma_byte: 0,
        })
    }

    /// Start a DMA transfer from the given source address
    fn start_dma(&mut self, value: u8) {
        self.dma_active = true;
        self.dma_start_addr = u16::from(value) << 8;
        self.dma_cycles = 160; // 160 M-cycles for DMA
        self.dma_byte = 0;
    }

    /// Process DMA transfer for the given number of cycles
    fn process_dma(&mut self, cycles: u32) {
        if !self.dma_active || self.dma_cycles == 0 {
            return;
        }

        // Process one byte per cycle
        let bytes_to_transfer = cycles.min(self.dma_cycles) as usize;
        for _ in 0..bytes_to_transfer {
            let source_addr = self.dma_start_addr + u16::from(self.dma_byte);
            let data = match source_addr {
                0x0000..=0x7FFF => self.mbc.read(source_addr).unwrap_or(0),
                0x8000..=0x9FFF => self.vram[(source_addr - 0x8000) as usize],
                0xA000..=0xBFFF => self.mbc.read(source_addr).unwrap_or(0),
                0xC000..=0xDFFF => self.wram[(source_addr - 0xC000) as usize],
                0xE000..=0xFDFF => self.wram[(source_addr - 0xE000) as usize],
                _ => 0xFF,
            };
            self.oam[self.dma_byte as usize] = data;
            self.dma_byte = self.dma_byte.wrapping_add(1);

            // DMA transfer is complete when we've transferred all 160 bytes
            if self.dma_byte >= 0xA0 {
                self.dma_active = false;
                self.dma_cycles = 0;
                break;
            }
        }

        self.dma_cycles = self.dma_cycles.saturating_sub(bytes_to_transfer as u32);
    }

    /// Read a byte from memory
    pub fn read(&self, addr: u16) -> u8 {
        // During DMA, only HRAM is accessible
        if self.dma_active && (addr < 0xFF80 || addr > 0xFFFE) && addr != 0xFF46 {
            return 0xFF;
        }

        match addr {
            // ROM bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => self.mbc.read(addr).unwrap_or(0xFF),

            // ROM bank 1-N (0x4000-0x7FFF)
            0x4000..=0x7FFF => self.mbc.read(addr).unwrap_or(0xFF),

            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => match self.ppu.get_mode() {
                PpuMode::PixelTransfer => 0xFF,
                _ => self.vram[(addr - 0x8000) as usize],
            },

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => self.mbc.read(addr).unwrap_or(0xFF),

            // Working RAM (0xC000-0xDFFF)
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],

            // Echo RAM (0xE000-0xFDFF)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],

            // OAM (0xFE00-0xFE9F)
            0xFE00..=0xFE9F => {
                if self.dma_active {
                    0xFF
                } else {
                    match self.ppu.get_mode() {
                        PpuMode::OamSearch | PpuMode::PixelTransfer => 0xFF,
                        _ => self.oam[(addr - 0xFE00) as usize],
                    }
                }
            }

            // Unused (0xFEA0-0xFEFF)
            0xFEA0..=0xFEFF => 0xFF,

            // I/O registers (0xFF00-0xFF7F)
            0xFF00..=0xFF7F => {
                match addr {
                    // PPU Registers
                    0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read(addr).unwrap_or(0xFF),

                    // DMA Register
                    0xFF46 => (self.dma_start_addr >> 8) as u8,

                    // Timer Registers
                    0xFF04..=0xFF07 => self.timer.read(addr).unwrap_or(0xFF),

                    // Other I/O
                    _ => self.io_registers[(addr - 0xFF00) as usize],
                }
            }

            // High RAM (0xFF80-0xFFFE)
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],

            // Interrupt Enable Register (0xFFFF)
            0xFFFF => self.interrupts.read_ie(),
        }
    }

    /// Write a byte to memory
    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), MmuError> {
        // During DMA, only HRAM is accessible
        if self.dma_active && (addr < 0xFF80 || addr > 0xFFFE) && addr != 0xFF46 {
            return Ok(());
        }

        match addr {
            // ROM bank 0 (0x0000-0x3FFF)
            0x0000..=0x3FFF => self.mbc.write(addr, value)?,

            // ROM bank 1-N (0x4000-0x7FFF)
            0x4000..=0x7FFF => self.mbc.write(addr, value)?,

            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                if self.ppu.get_mode() != PpuMode::PixelTransfer {
                    self.vram[(addr - 0x8000) as usize] = value;
                }
            }

            // External RAM (0xA000-0xBFFF)
            0xA000..=0xBFFF => self.mbc.write(addr, value)?,

            // Working RAM (0xC000-0xDFFF)
            0xC000..=0xDFFF => {
                self.wram[(addr - 0xC000) as usize] = value;
            }

            // Echo RAM (0xE000-0xFDFF)
            0xE000..=0xFDFF => {
                self.wram[(addr - 0xE000) as usize] = value;
            }

            // OAM (0xFE00-0xFE9F)
            0xFE00..=0xFE9F => {
                if !self.dma_active
                    && self.ppu.get_mode() != PpuMode::OamSearch
                    && self.ppu.get_mode() != PpuMode::PixelTransfer
                {
                    self.oam[(addr - 0xFE00) as usize] = value;
                }
            }

            // Unused (0xFEA0-0xFEFF)
            0xFEA0..=0xFEFF => {}

            // I/O registers (0xFF00-0xFF7F)
            0xFF00..=0xFF7F => {
                match addr {
                    // PPU Registers
                    0xFF40..=0xFF45 | 0xFF47..=0xFF4B => {
                        let _ = self.ppu.write(addr, value);
                    }

                    // DMA Register
                    0xFF46 => self.start_dma(value),

                    // Timer Registers
                    0xFF04..=0xFF07 => {
                        let _ = self.timer.write(addr, value);
                    }

                    // Other I/O
                    _ => {
                        self.io_registers[(addr - 0xFF00) as usize] = value;
                    }
                }
            }

            // High RAM (0xFF80-0xFFFE)
            0xFF80..=0xFFFE => {
                self.hram[(addr - 0xFF80) as usize] = value;
            }

            // Interrupt Enable Register (0xFFFF)
            0xFFFF => self.interrupts.write_ie(value),
        }
        Ok(())
    }

    pub fn step(&mut self, cycles: u32) {
        self.process_dma(cycles);

        // Handle timer step and possible interrupt
        if let Ok(state) = self.timer.step(cycles) {
            if state == TimerState::Reloading && self.timer.get_interrupt_flag() {
                self.interrupts.request(InterruptFlag::Timer);
                self.timer.clear_interrupt_flag();
            }
        }

        if let Some(interrupt) = self.ppu.step(cycles) {
            self.interrupts.request(interrupt);
        }
    }

    pub fn save_ram(&self) -> Vec<u8> {
        self.mbc.save_ram()
    }

    pub fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError> {
        self.mbc.load_ram(data)
    }
}

impl MemoryBusTrait for MMU {
    fn read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) -> Result<(), MmuError> {
        self.write(addr, value)
    }

    fn get_interrupt(&self) -> Option<InterruptFlag> {
        self.interrupts.get_interrupt()
    }

    fn clear_interrupt(&mut self, flag: InterruptFlag) {
        self.interrupts.clear(flag);
    }

    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16 {
        Interrupts::get_vector(flag)
    }
}

#[cfg(test)]
mod tests;
