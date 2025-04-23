//! Types and helpers for CPU opcode definitions.
//!
//! This module defines the core Opcode struct, register index constants, and helpers used throughout the opcode implementation.
//!
//! Keeping these types separate improves clarity and maintainability.

use super::CPU;
use crate::mmu::MemoryBusTrait;

/// Type alias for the opcode execution function signature.
pub type OpcodeExecFn = dyn Fn(&mut CPU, &mut dyn MemoryBusTrait) -> bool + Send + Sync;

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
    pub exec: Box<OpcodeExecFn>,
}
