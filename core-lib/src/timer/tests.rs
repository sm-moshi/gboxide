/// core-lib/src/timer/tests.rs
use super::*;
use pretty_assertions::assert_eq;

#[test]
fn test_timer_frequencies() {
    let mut timer = Timer::new();
    timer.write(0xFF07, 0b100); // Enable timer

    // Test 4.096 KHz
    timer.write(0xFF07, 0b100); // 0b100 | 0b00 is just 0b100
    assert_eq!(timer.get_counter_mask(), 1 << 9);

    // Test 262.144 KHz
    timer.write(0xFF07, 0b101); // 0b100 | 0b01
    assert_eq!(timer.get_counter_mask(), 1 << 3);

    // Test 65.536 KHz
    timer.write(0xFF07, 0b110); // 0b100 | 0b10
    assert_eq!(timer.get_counter_mask(), 1 << 5);

    // Test 16.384 KHz
    timer.write(0xFF07, 0b111); // 0b100 | 0b11
    assert_eq!(timer.get_counter_mask(), 1 << 7);
}

#[test]
fn test_timer_overflow_delay() {
    let mut timer = Timer::new();
    timer.write(0xFF07, 0b100); // Enable timer
    timer.tma = 0x42; // Set modulo value

    // Manually trigger overflow
    timer.tima = 0xFF;
    timer.tima_overflow = true;

    // Next step should load TMA and request interrupt
    assert!(timer.step(1));
    assert_eq!(timer.tima, 0x42);
    assert!(!timer.tima_overflow);
}
// TODO: Fix this test
// #[test]
// fn test_timer_increment_overflow() {
//     let mut timer = Timer::new();
//     timer.write(0xFF07, 0b100); // Enable timer
//     timer.tma = 0x42; // Set modulo value
//     timer.tima = 0xFF; // Set counter to overflow value

//     // Set up for falling edge
//     timer.div_counter = (1 << 9) - 1; // Just before bit 9 becomes 1
//     assert!(!timer.get_input()); // Verify input bit is 0

//     // Step one cycle to set bit 9 to 1
//     timer.step(1);
//     assert!(timer.get_input()); // Verify input bit is 1

//     // Step one cycle to clear bit 9, triggering falling edge and overflow
//     timer.step(1);
//     assert_eq!(timer.tima, 0xFF); // TIMA stays at 0xFF for one cycle
//     assert!(timer.tima_overflow);

//     // Next step should load TMA and request interrupt
//     assert!(timer.step(1));
//     assert_eq!(timer.tima, 0x42);
//     assert!(!timer.tima_overflow);
// }

#[test]
fn test_div_reset() {
    let mut timer = Timer::new();
    timer.div_counter = 0x1234;
    timer.div = 0x12;

    timer.reset_div();
    assert_eq!(timer.div_counter, 0);
    assert_eq!(timer.div, 0);
}

#[test]
fn test_div_reset_causes_timer_increment() {
    let mut timer = Timer::new();
    timer.write(0xFF07, 0b100); // Enable timer
    timer.tima = 0x42;

    // Set DIV so that the input bit is 1
    timer.div_counter = 1 << 9;
    assert!(timer.get_input()); // Verify input bit is 1

    // Resetting DIV should cause a timer increment
    timer.reset_div();
    assert_eq!(timer.tima, 0x43);
}

#[test]
fn test_tac_change_causes_timer_increment() {
    let mut timer = Timer::new();
    timer.write(0xFF07, 0b100); // Enable timer
    timer.tima = 0x42;

    // Set DIV so that bit 9 is 1 (used for 4.096 KHz)
    timer.div_counter = 1 << 9;
    assert!(timer.get_input()); // Verify input bit is 1

    // Changing to 16.384 KHz should cause increment if bit 7 is 0
    timer.write(0xFF07, 0b111);
    assert_eq!(timer.tima, 0x43);
}
