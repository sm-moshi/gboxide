//! Utility helpers for opcode execution.
//!
//! This module contains shared helper functions for opcode logic, such as RST routines.
//!
//! Keeping helpers separate reduces duplication and clarifies intent.

use super::CPU;
use crate::mmu::MemoryBusTrait;

/// Common RST routine: pushes PC to stack and jumps to the given address.
pub(crate) fn rst_common(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait, address: u16) -> bool {
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    let _ = bus.write(cpu.regs.sp, (cpu.regs.pc >> 8) as u8);
    cpu.regs.sp = cpu.regs.sp.wrapping_sub(1);
    let _ = bus.write(cpu.regs.sp, cpu.regs.pc as u8);
    cpu.regs.pc = address;
    false
}

/// RST 00H handler.
pub(crate) fn rst_00(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x00)
}
/// RST 08H handler.
pub(crate) fn rst_08(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x08)
}
/// RST 10H handler.
pub(crate) fn rst_10(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x10)
}
/// RST 18H handler.
pub(crate) fn rst_18(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x18)
}
/// RST 20H handler.
pub(crate) fn rst_20(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x20)
}
/// RST 28H handler.
pub(crate) fn rst_28(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x28)
}
/// RST 30H handler.
pub(crate) fn rst_30(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x30)
}
/// RST 38H handler.
pub(crate) fn rst_38(cpu: &mut CPU, bus: &mut dyn MemoryBusTrait) -> bool {
    rst_common(cpu, bus, 0x38)
}
