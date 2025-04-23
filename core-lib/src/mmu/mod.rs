use crate::cartridge::Cartridge;
use crate::interrupts::{InterruptFlag, Interrupts};
use crate::ppu::{Ppu, PpuMode};
use crate::timer::{Timer, TimerState};
use std::cell::RefCell;
use thiserror::Error;

pub mod mbc;
pub use mbc::{Mbc, Mbc1, MbcError, NoMbc};

pub mod input;
pub use input::GameBoyButton;

/// Joypad buttons (Game Boy)
#[derive(Debug, Default, Clone, Copy)]
pub struct Joypad {
    /// Bit 0: Right / A
    /// Bit 1: Left  / B
    /// Bit 2: Up    / Select
    /// Bit 3: Down  / Start
    /// Bit 4: Select Direction/Action
    /// Bit 5-7: Unused
    pub select: u8, // 0: Direction, 1: Action
    pub state: u8, // 4 bits: 0=pressed, 1=released (active low)
}

impl Joypad {
    pub const fn new() -> Self {
        Self {
            select: 0,
            state: 0xFF,
        }
    }

    /// Set the button state (bit: 0=pressed, 1=released)
    pub fn set_button(&mut self, button: GameBoyButton, pressed: bool) {
        let idx = button.to_index();
        if pressed {
            self.state &= !(1 << idx);
        } else {
            self.state |= 1 << idx;
        }
    }

    /// Read the current joypad value (active low)
    pub const fn read(&self) -> u8 {
        // Bit 4: Select Direction (0) or Action (1)
        // If both bits are 1, all buttons return 1 (not pressed)
        let mut res = 0xCF; // Upper 4 bits always 1
        if self.select & 0x10 == 0 {
            // Direction: Down, Up, Left, Right (bits 3-0)
            res |= self.state & 0x0F;
        }
        if self.select & 0x20 == 0 {
            // Action: Start, Select, B, A (bits 3-0)
            res |= (self.state >> 4) & 0x0F;
        }
        res
    }

    /// Write to the joypad select bits
    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }
}

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
    /// Write a byte to memory
    ///
    /// # Errors
    /// Returns an error if the address is invalid or the underlying MBC fails.
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MmuError>;

    /// Get the highest priority pending interrupt
    fn get_interrupt(&self) -> Option<InterruptFlag>;

    /// Clear an interrupt flag after handling
    fn clear_interrupt(&mut self, flag: InterruptFlag);

    /// Get the interrupt vector address for a given interrupt
    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16;

    fn as_any(&mut self) -> &mut dyn std::any::Any;

    fn interrupts_mut(&self) -> Option<&RefCell<Interrupts>>;
}

/// Memory Management Unit implementation
pub struct MMU {
    mbc: Box<dyn Mbc>,
    vram: [u8; 0x2000],       // 8KB Video RAM
    wram: [u8; 0x2000],       // 8KB Work RAM
    oam: [u8; 0xA0],          // Object Attribute Memory
    io_registers: [u8; 0x80], // Hardware I/O Registers
    hram: [u8; 0x7F],         // High RAM
    pub interrupts: RefCell<Interrupts>,
    pub timer: Timer,
    pub ppu: Ppu,        // Pixel Processing Unit
    dma_active: bool,    // Whether DMA is currently active
    dma_start_addr: u16, // Source address for DMA
    dma_cycles: u32,     // Remaining cycles for DMA
    dma_byte: u8,        // Current byte being transferred in DMA
    pub joypad: Joypad,  // Add joypad field
    serial_data: u8,     // Serial transfer data (SB, 0xFF01)
    serial_control: u8,  // Serial transfer control (SC, 0xFF02)
}

#[allow(clippy::cast_possible_truncation)] // Intentional truncation for DMA cycles and memory addressing
impl MMU {
    /// Creates a new MMU instance from the given ROM data.
    ///
    /// # Errors
    /// Returns an error if the cartridge or MBC cannot be created.
    #[must_use = "Returns a new MMU instance or an error if initialization fails"]
    pub fn new(rom: Vec<u8>) -> Result<Self, MmuError> {
        let cartridge = Cartridge::new(rom)?;
        let mbc = cartridge.create_mbc()?;
        Ok(Self {
            mbc,
            vram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_registers: [0; 0x80],
            hram: [0; 0x7F],
            interrupts: RefCell::new(Interrupts::new()),
            timer: Timer::new(),
            ppu: Ppu::new(),
            dma_active: false,
            dma_start_addr: 0,
            dma_cycles: 0,
            dma_byte: 0,
            joypad: Joypad::new(),
            serial_data: 0,
            serial_control: 0,
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
                0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc.read(source_addr).unwrap_or(0),
                0x8000..=0x9FFF => self.vram[(source_addr - 0x8000) as usize],
                0xC000..=0xDFFF => self.wram[(source_addr - 0xC000) as usize],
                0xE000..=0xFDFF => self.wram[(source_addr - 0xE000) as usize],
                0xFE00..=0xFE9F => {
                    if self.dma_active {
                        0xFF
                    } else {
                        match self.ppu.get_mode() {
                            PpuMode::OamSearch | PpuMode::PixelTransfer => 0xFF,
                            _ => self.oam[(source_addr - 0xFE00) as usize],
                        }
                    }
                }
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
        if self.dma_active && !(0xFF80..=0xFFFE).contains(&addr) && addr != 0xFF46 {
            return 0xFF;
        }

        match addr {
            // ROM bank 0/1-N (0x0000-0x7FFF)
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc.read(addr).unwrap_or(0xFF),
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => match self.ppu.get_mode() {
                PpuMode::PixelTransfer => 0xFF,
                _ => self.vram[(addr - 0x8000) as usize],
            },
            // Working RAM (0xC000-0xDFFF)
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            // Echo RAM (0xE000-0xFDFF) mirrors 0xC000-0xDDFF
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
                    0xFF00 => self.joypad.read(),
                    0xFF01 => self.serial_data,
                    0xFF02 => self.serial_control,
                    0xFF0F => self.interrupts.borrow().read_if(),
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
            0xFFFF => self.interrupts.borrow().read_ie(),
        }
    }

    /// Write a byte to memory
    ///
    /// # Errors
    /// Returns an error if the address is invalid or the underlying MBC fails.
    pub fn write(&mut self, addr: u16, value: u8) -> Result<(), MmuError> {
        // During DMA, only HRAM is accessible
        if self.dma_active && !(0xFF80..=0xFFFE).contains(&addr) && addr != 0xFF46 {
            return Ok(());
        }

        match addr {
            // ROM bank 0/1-N (0x0000-0x7FFF)
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc.write(addr, value)?,
            // VRAM (0x8000-0x9FFF)
            0x8000..=0x9FFF => {
                if self.ppu.get_mode() != PpuMode::PixelTransfer {
                    self.vram[(addr - 0x8000) as usize] = value;
                }
            }
            // Working RAM (0xC000-0xDFFF)
            0xC000..=0xDFFF => {
                self.wram[(addr - 0xC000) as usize] = value;
            }
            // Echo RAM (0xE000-0xFDFF) mirrors 0xC000-0xDDFF
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
                    0xFF00 => {
                        self.joypad.write(value);
                    }
                    0xFF01 => {
                        self.serial_data = value;
                    }
                    0xFF02 => {
                        self.serial_control = value;
                        // No actual serial transfer emulation; CLI will poll for 0x81
                    }
                    0xFF0F => {
                        self.interrupts.borrow_mut().write_if(value);
                    }
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
            0xFFFF => self.interrupts.borrow_mut().write_ie(value),
        }
        Ok(())
    }

    pub fn step(&mut self, cycles: u32) {
        self.process_dma(cycles);

        // Handle timer step and possible interrupt
        if let Ok(state) = self.timer.step(cycles) {
            if state == TimerState::Reloading && self.timer.get_interrupt_flag() {
                self.interrupts.borrow_mut().request(InterruptFlag::Timer);
                self.timer.clear_interrupt_flag();
            }
        }

        if let Some(interrupt) = self.ppu.step(cycles) {
            self.interrupts.borrow_mut().request(interrupt);
        }
    }

    pub fn save_ram(&self) -> Vec<u8> {
        self.mbc.save_ram()
    }

    /// Load RAM data into the MBC
    ///
    /// # Errors
    /// Returns an error if the RAM data size is invalid.
    pub fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError> {
        self.mbc.load_ram(data)
    }

    /// Update the joypad state from host input
    pub fn update_joypad(&mut self, button: GameBoyButton, pressed: bool) {
        let prev = self.joypad.state;
        self.joypad.set_button(button, pressed);
        // If a button was just pressed, trigger interrupt
        if (prev & !(self.joypad.state)) != 0 {
            self.interrupts.borrow_mut().request(InterruptFlag::Joypad);
        }
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
        self.interrupts.borrow().get_interrupt()
    }

    fn clear_interrupt(&mut self, flag: InterruptFlag) {
        self.interrupts.borrow_mut().clear(flag);
    }

    fn get_interrupt_vector(&self, flag: InterruptFlag) -> u16 {
        Interrupts::get_vector(flag)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn interrupts_mut(&self) -> Option<&RefCell<Interrupts>> {
        Some(&self.interrupts)
    }
}

#[cfg(test)]
mod tests;
