use super::*;
use pretty_assertions::assert_eq;

#[test]
fn test_interrupt_priority() {
    let mut interrupts = Interrupts::new();

    // Enable all interrupts
    interrupts.write_ie(0x1F);
    interrupts.ime = true;

    // Request multiple interrupts
    interrupts.request(InterruptFlag::Timer);
    interrupts.request(InterruptFlag::VBlank);
    interrupts.request(InterruptFlag::LcdStat);

    // VBlank should be handled first (highest priority)
    assert!(matches!(
        interrupts.get_interrupt(),
        Some(InterruptFlag::VBlank)
    ));
    interrupts.clear(InterruptFlag::VBlank);

    // LCD STAT should be next
    assert!(matches!(
        interrupts.get_interrupt(),
        Some(InterruptFlag::LcdStat)
    ));
    interrupts.clear(InterruptFlag::LcdStat);

    // Timer should be last
    assert!(matches!(
        interrupts.get_interrupt(),
        Some(InterruptFlag::Timer)
    ));
}

#[test]
fn test_interrupt_masking() {
    let mut interrupts = Interrupts::new();

    // Only enable VBlank
    interrupts.write_ie(1 << InterruptFlag::VBlank as u8);
    interrupts.ime = true;

    // Request Timer interrupt (should be masked)
    interrupts.request(InterruptFlag::Timer);
    assert!(interrupts.get_interrupt().is_none());

    // Request VBlank (should be handled)
    interrupts.request(InterruptFlag::VBlank);
    assert!(matches!(
        interrupts.get_interrupt(),
        Some(InterruptFlag::VBlank)
    ));
}

#[test]
fn test_ime_scheduling() {
    let mut interrupts = Interrupts::new();

    // Schedule IME to be enabled
    interrupts.schedule_enable_ime();
    assert!(!interrupts.ime);

    // IME should not be enabled yet
    assert!(!interrupts.pending());

    // Update IME (simulating next instruction completion)
    interrupts.update_ime();
    assert!(interrupts.ime);
}

#[test]
fn test_halt_bug_detection() {
    let mut interrupts = Interrupts::new();

    // Set up condition for HALT bug
    interrupts.ime = false;
    interrupts.write_ie(1 << InterruptFlag::VBlank as u8);
    interrupts.request(InterruptFlag::VBlank);

    // Should detect pending interrupt regardless of IME
    assert!(interrupts.pending_regardless_of_ime());
    assert!(!interrupts.pending());
}

#[test]
fn test_interrupt_vectors() {
    assert_eq!(Interrupts::get_vector(InterruptFlag::VBlank), 0x0040);
    assert_eq!(Interrupts::get_vector(InterruptFlag::LcdStat), 0x0048);
    assert_eq!(Interrupts::get_vector(InterruptFlag::Timer), 0x0050);
    assert_eq!(Interrupts::get_vector(InterruptFlag::Serial), 0x0058);
    assert_eq!(Interrupts::get_vector(InterruptFlag::Joypad), 0x0060);
}

#[test]
fn test_interrupt_handling_sequence() {
    let mut interrupts = Interrupts::new();

    // Enable VBlank interrupt and IME
    interrupts.write_ie(1 << InterruptFlag::VBlank as u8);
    interrupts.ime = true;

    // Request VBlank interrupt
    interrupts.request(InterruptFlag::VBlank);

    // Verify interrupt is pending
    assert!(interrupts.pending());
    assert!(matches!(
        interrupts.get_interrupt(),
        Some(InterruptFlag::VBlank)
    ));

    // Clear interrupt and verify it's handled
    interrupts.clear(InterruptFlag::VBlank);
    assert!(!interrupts.pending());
    assert!(interrupts.get_interrupt().is_none());
}

#[test]
fn test_halt_mode_exit() {
    let mut interrupts = Interrupts::new();

    // Enable Timer interrupt but keep IME disabled
    interrupts.write_ie(1 << InterruptFlag::Timer as u8);
    interrupts.ime = false;

    // Request Timer interrupt
    interrupts.request(InterruptFlag::Timer);

    // Verify interrupt is pending but not handled (due to IME=0)
    assert!(!interrupts.pending());
    assert!(interrupts.pending_regardless_of_ime());

    // This state should trigger HALT mode exit
    assert!(interrupts.get_interrupt().is_none());
}

#[test]
fn test_no_interrupt_when_disabled() {
    let mut interrupts = Interrupts::new();
    interrupts.set_ime(false);
    interrupts.request(InterruptFlag::VBlank);
    assert!(interrupts.get_interrupt().is_none());
}

#[test]
fn test_no_interrupt_when_not_enabled() {
    let mut interrupts = Interrupts::new();
    interrupts.set_ime(true);
    interrupts.request(InterruptFlag::VBlank);
    assert!(interrupts.get_interrupt().is_none());
}

#[test]
fn test_no_interrupt_when_not_requested() {
    let mut interrupts = Interrupts::new();
    interrupts.set_ime(true);
    interrupts.enable(InterruptFlag::VBlank);
    assert!(interrupts.get_interrupt().is_none());
}
