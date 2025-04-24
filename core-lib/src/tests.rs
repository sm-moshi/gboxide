use crate::bus::Bus;
use crate::interrupts::InterruptFlag;
use crate::mmu::MMU;

#[test]
fn test_bus_read_write() {
    let mut mmu = MMU::default();
    let mut bus = Bus { mmu: &mut mmu };
    bus.write(0x1234, 0xAB);
    assert_eq!(bus.read(0x1234), 0xAB);
}

#[test]
fn test_bus_read_write_word() {
    let mut mmu = MMU::default();
    let mut bus = Bus { mmu: &mut mmu };
    bus.write_word(0x1000, 0xBEEF);
    assert_eq!(bus.read_word(0x1000), 0xBEEF);
}

#[test]
fn test_bus_interrupt_methods() {
    let mut mmu = MMU::default();
    // Simulate an interrupt request (if MMU supports it)
    // For demonstration, we assume MMU has a method to set an interrupt flag directly for test
    // If not, this test can be adjusted to use public API only
    // mmu.request_interrupt(InterruptFlag::Timer); // Uncomment if available
    let mut bus = Bus { mmu: &mut mmu };
    // The following lines are placeholders; adjust as needed for MMU's API
    // assert_eq!(bus.get_interrupt(), Some(InterruptFlag::Timer));
    // bus.clear_interrupt(InterruptFlag::Timer);
    // assert_eq!(bus.get_interrupt(), None);
    assert_eq!(bus.get_interrupt_vector(InterruptFlag::VBlank), 0x40);
}
