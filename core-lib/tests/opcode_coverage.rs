use core_lib::cpu::CB_OPCODES;
use core_lib::cpu::CPU;
use core_lib::interrupts::InterruptFlag;
use core_lib::mmu::{MemoryBusTrait, MmuError};
use std::cell::RefCell;

/// Dummy bus that returns 0 for all reads and ignores writes.
struct DummyBus;
impl MemoryBusTrait for DummyBus {
    fn read(&self, _addr: u16) -> u8 {
        0
    }
    fn write(&mut self, _addr: u16, _val: u8) -> Result<(), MmuError> {
        Ok(())
    }
    fn get_interrupt(&self) -> Option<InterruptFlag> {
        None
    }
    fn clear_interrupt(&mut self, _flag: InterruptFlag) {}
    fn get_interrupt_vector(&self, _flag: InterruptFlag) -> u16 {
        0
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn interrupts_mut(&self) -> Option<&RefCell<core_lib::interrupts::Interrupts>> {
        None
    }
}

#[test]
fn test_all_opcodes_execute() {
    let mut cpu = CPU::new();
    let mut bus = DummyBus;

    for opcode in 0x00u8..=0xFF {
        // Reset CPU state for each opcode to avoid side effects
        cpu.regs = Default::default();
        cpu.ime = false;
        cpu.halted = false;
        cpu.stopped = false;
        cpu.regs.pc = 0;

        // Try to execute the opcode
        let result = cpu.execute(opcode, &mut bus);

        // We only care that the code path is exercised, not that it succeeds
        if let Err(e) = &result {
            println!("Opcode 0x{:02X} returned error: {:?}", opcode, e);
        }
    }
}

#[test]
fn test_all_cb_opcodes_execute() {
    let mut cpu = CPU::new();
    let mut bus = DummyBus;

    for cb_opcode in 0x00u8..=0xFF {
        // Reset CPU state for each opcode to avoid side effects
        cpu.regs = Default::default();
        cpu.ime = false;
        cpu.halted = false;
        cpu.stopped = false;
        cpu.regs.pc = 0;

        // Try to execute the CB-prefixed opcode
        let result = (CB_OPCODES[cb_opcode as usize].exec)(&mut cpu, &mut bus);

        // We only care that the code path is exercised, not that it succeeds
        if let Err(e) = &result {
            println!("CB Opcode 0x{:02X} returned error: {:?}", cb_opcode, e);
        }
    }
}
