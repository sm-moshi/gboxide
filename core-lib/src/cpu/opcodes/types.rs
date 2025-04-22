//! Types and helpers for CPU opcode definitions.
//!
//! This module defines the core Opcode struct, register index constants, and helpers used throughout the opcode implementation.
//!
//! Keeping these types separate improves clarity and maintainability.

use super::CPU;
use crate::mmu::MemoryBusTrait;

/// Register indices for named 8-bit registers.
pub(crate) const REG_INDICES: [(char, usize); 7] = [
    ('b', 0),
    ('c', 1),
    ('d', 2),
    ('e', 3),
    ('h', 4),
    ('l', 5),
    ('a', 7),
];

/// Converts a register name (as a string) to its index in the register array.
///
/// # Panics
/// Panics if the register name is invalid.
pub(crate) fn reg_to_index(reg: &str) -> usize {
    let reg_char = reg.chars().next().unwrap();
    REG_INDICES
        .iter()
        .find(|(c, _)| *c == reg_char)
        .map(|(_, idx)| *idx)
        .unwrap_or_else(|| panic!("Invalid register"))
}

/// Represents a single CPU opcode and its execution logic.
#[doc = "Each Opcode contains its mnemonic, timing, and the function to execute it."]
pub struct Opcode {
    /// Human-readable mnemonic for debugging and disassembly.
    pub mnemonic: &'static str,
    /// Base cycle count (including fetch cycles).
    pub base_cycles: u32,
    /// Additional cycles for conditional instructions.
    pub conditional_cycles: u32,
    /// The function that executes the opcode. Returns true if a condition was met.
    pub exec: Box<dyn Fn(&mut CPU, &mut dyn MemoryBusTrait) -> bool + Send + Sync>,
}
