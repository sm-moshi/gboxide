use super::{Mbc, MbcError};
use std::time::{Duration, SystemTime};

/// RTC registers for MBC3
#[derive(Clone, Debug)]
struct Rtc {
    seconds: u8, // 0–59
    minutes: u8, // 0–59
    hours: u8,   // 0–23
    days: u16,   // 0–511
    halt: bool,
    carry: bool,
    last_update: SystemTime,
}

impl Rtc {
    fn new() -> Self {
        Self {
            seconds: 0,
            minutes: 0,
            hours: 0,
            days: 0,
            halt: false,
            carry: false,
            last_update: SystemTime::now(),
        }
    }
    /// Update RTC based on elapsed time (unless halted)
    fn tick(&mut self) {
        if self.halt {
            self.last_update = SystemTime::now();
            return;
        }
        let now = SystemTime::now();
        let elapsed = now
            .duration_since(self.last_update)
            .unwrap_or(Duration::ZERO);
        let mut secs = elapsed.as_secs();
        if secs == 0 {
            return;
        }
        self.last_update = now;
        while secs > 0 {
            self.seconds += 1;
            if self.seconds == 60 {
                self.seconds = 0;
                self.minutes += 1;
                if self.minutes == 60 {
                    self.minutes = 0;
                    self.hours += 1;
                    if self.hours == 24 {
                        self.hours = 0;
                        self.days += 1;
                        if self.days == 512 {
                            self.days = 0;
                            self.carry = true;
                        }
                    }
                }
            }
            secs -= 1;
        }
    }
    /// Latch a snapshot of the current RTC state
    fn latch(&self) -> Self {
        self.clone()
    }
    /// Read a register (0x08–0x0C)
    const fn read_reg(&self, reg: u8) -> u8 {
        match reg {
            0x08 => self.seconds,
            0x09 => self.minutes,
            0x0A => self.hours,
            0x0B => (self.days & 0xFF) as u8,
            0x0C => {
                let mut v = 0u8;
                v |= ((self.days >> 8) & 0x01) as u8; // bit 0: day 8
                if self.halt {
                    v |= 1 << 6;
                }
                if self.carry {
                    v |= 1 << 7;
                }
                v
            }
            _ => 0xFF,
        }
    }
    /// Write a register (0x08–0x0C)
    fn write_reg(&mut self, reg: u8, value: u8) {
        match reg {
            0x08 => self.seconds = value % 60,
            0x09 => self.minutes = value % 60,
            0x0A => self.hours = value % 24,
            0x0B => self.days = (self.days & 0x100) | u16::from(value),
            0x0C => {
                self.days = (self.days & 0xFF) | ((u16::from(value) & 0x01) << 8);
                self.halt = value & 0x40 != 0;
                self.carry = value & 0x80 != 0;
            }
            _ => {}
        }
    }
    /// Serialise RTC state to bytes (for battery-backed save)
    fn to_bytes(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0] = self.seconds;
        buf[1] = self.minutes;
        buf[2] = self.hours;
        buf[3] = (self.days & 0xFF) as u8;
        buf[4] = ((self.days >> 8) & 0x01) as u8;
        buf[5] = u8::from(self.halt);
        buf[6] = u8::from(self.carry);
        let ts = self
            .last_update
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        buf[8..16].copy_from_slice(&ts.to_le_bytes());
        buf
    }
    /// Deserialise RTC state from bytes
    fn from_bytes(buf: &[u8]) -> Self {
        let seconds = buf.first().copied().unwrap_or(0);
        let minutes = buf.get(1).copied().unwrap_or(0);
        let hours = buf.get(2).copied().unwrap_or(0);
        let days_lo = u16::from(buf.get(3).copied().unwrap_or(0));
        let days_hi = u16::from(buf.get(4).copied().unwrap_or(0));
        let halt = buf.get(5).copied().unwrap_or(0) != 0;
        let carry = buf.get(6).copied().unwrap_or(0) != 0;
        let mut ts_bytes = [0u8; 8];
        ts_bytes.copy_from_slice(buf.get(8..16).unwrap_or(&[0; 8]));
        let ts = u64::from_le_bytes(ts_bytes);
        let last_update = SystemTime::UNIX_EPOCH + Duration::from_secs(ts);
        Self {
            seconds,
            minutes,
            hours,
            days: days_lo | (days_hi << 8),
            halt,
            carry,
            last_update,
        }
    }
}

/// MBC3: 2MB ROM, 32KB RAM, Real-Time Clock (RTC)
/// - Up to 128 ROM banks (0x4000–0x7FFF, 7 bits)
/// - Up to 4 RAM banks (8KB each, 0xA000–0xBFFF)
/// - RTC registers (0x08–0x0C, mapped to 0xA000–0xBFFF)
/// - RAM/RTC enable: 0x0000–0x1FFF, 0x0A enables
/// - ROM bank select: 0x2000–0x3FFF (7 bits, 0 selects 1)
/// - RAM bank/RTC select: 0x4000–0x5FFF (0–3 = RAM, 0x08–0x0C = RTC)
/// - Latch clock data: 0x6000–0x7FFF (write 0 then 1)
pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>, // up to 32KB (4 x 8KB)
    ram_enabled: bool,
    rom_bank: u8,    // 7 bits
    ram_bank: u8,    // 2 bits (0–3) or RTC reg (0x08–0x0C)
    rtc: Rtc,        // live RTC
    rtc_latch: Rtc,  // latched RTC
    latch_state: u8, // 0 or 1
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>) -> Self {
        // RAM size: up to 32KB (4 banks)
        let ram_buf = vec![0; 0x8000];
        let rtc = Rtc::new();
        Self {
            rom,
            ram: ram_buf,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc: rtc.clone(),
            rtc_latch: rtc,
            latch_state: 0,
        }
    }
    const fn rtc_index(sel: u8) -> Option<u8> {
        match sel {
            0x08..=0x0C => Some(sel),
            _ => None,
        }
    }
}

impl Mbc for Mbc3 {
    fn read(&self, addr: u16) -> Result<u8, MbcError> {
        match addr {
            // ROM Bank 0
            0x0000..=0x3FFF => Ok(self.rom.get(addr as usize).copied().unwrap_or(0xFF)),
            // ROM Bank 1–127
            0x4000..=0x7FFF => {
                let mut bank = self.rom_bank & 0x7F;
                if bank == 0 {
                    bank = 1;
                }
                let idx = (bank as usize) * 0x4000 + (addr as usize - 0x4000);
                Ok(self.rom.get(idx).copied().unwrap_or(0xFF))
            }
            // RAM or RTC
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                Self::rtc_index(self.ram_bank).map_or_else(
                    || {
                        if self.ram_bank < 4 {
                            let idx = (self.ram_bank as usize) * 0x2000 + (addr as usize - 0xA000);
                            Ok(*self.ram.get(idx).unwrap_or(&0xFF))
                        } else {
                            Err(MbcError::InvalidRamBank(self.ram_bank as usize))
                        }
                    },
                    |reg| Ok(self.rtc_latch.read_reg(reg)),
                )
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MbcError> {
        // Only tick RTC on non-RTC register writes
        let is_rtc_reg = Self::rtc_index(self.ram_bank)
            .is_some_and(|reg| (0xA000..=0xBFFF).contains(&addr) && (0x08..=0x0C).contains(&reg));
        if !is_rtc_reg {
            self.rtc.tick();
        }
        match addr {
            // RAM/RTC Enable
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
                Ok(())
            }
            // ROM Bank Number (7 bits)
            0x2000..=0x3FFF => {
                let mut bank = value & 0x7F;
                if bank == 0 {
                    bank = 1;
                }
                self.rom_bank = bank;
                Ok(())
            }
            // RAM Bank Number or RTC Register Select
            0x4000..=0x5FFF => {
                self.ram_bank = value;
                Ok(())
            }
            // Latch Clock Data
            0x6000..=0x7FFF => {
                // Latch on 0->1 transition
                if self.latch_state == 0 && value == 1 {
                    self.rtc.tick();
                    self.rtc_latch = self.rtc.latch();
                }
                self.latch_state = value & 1;
                Ok(())
            }
            // RAM or RTC
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return Err(MbcError::RamDisabled);
                }
                if let Some(reg) = Self::rtc_index(self.ram_bank) {
                    self.rtc.tick();
                    self.rtc.write_reg(reg, value);
                    Ok(())
                } else if self.ram_bank < 4 {
                    let idx = (self.ram_bank as usize) * 0x2000 + (addr as usize - 0xA000);
                    if idx < self.ram.len() {
                        self.ram[idx] = value;
                        Ok(())
                    } else {
                        Err(MbcError::InvalidRamBank(self.ram_bank as usize))
                    }
                } else {
                    Err(MbcError::InvalidRamBank(self.ram_bank as usize))
                }
            }
            _ => Err(MbcError::ProtectionViolation(addr)),
        }
    }
    fn rom_bank(&self) -> usize {
        self.rom_bank as usize
    }
    fn ram_bank(&self) -> usize {
        self.ram_bank as usize
    }
    fn is_ram_enabled(&self) -> bool {
        self.ram_enabled
    }
    fn save_ram(&self) -> Vec<u8> {
        let mut out = self.ram.clone();
        out.extend_from_slice(&self.rtc.to_bytes());
        out
    }
    fn load_ram(&mut self, data: Vec<u8>) -> Result<(), MbcError> {
        let ram_len = self.ram.len();
        if data.len() == ram_len + 16 {
            // Avoid simultaneous borrow by splitting first
            let rtc_bytes = &data[ram_len..];
            self.ram.copy_from_slice(&data[..ram_len]);
            self.rtc = Rtc::from_bytes(rtc_bytes);
            Ok(())
        } else if data.len() == ram_len {
            self.ram.copy_from_slice(&data);
            self.rtc = Rtc::new();
            Ok(())
        } else {
            Err(MbcError::InvalidRamBank(data.len() / 0x2000))
        }
    }
}
