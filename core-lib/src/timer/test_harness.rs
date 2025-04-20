/// core-lib/src/timer/test_harness.rs
/// Test harness for timer testing
use super::{Timer, TimerError, TimerState};
use tracing::{debug, instrument};

/// TimerTestHarness provides a way to easily test timer behavior
pub struct TimerTestHarness {
    /// The timer under test
    pub timer: Timer,

    /// Log of timer states during testing
    pub cycle_log: Vec<(u64, TimerState, u8, bool)>, // (cycle, state, tima, interrupt)
}

impl TimerTestHarness {
    /// Create a new test harness with default timer
    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
            cycle_log: Vec::new(),
        }
    }

    /// Create a test harness with specific timer configuration
    pub fn setup(tima: u8, tma: u8, tac: u8, div: u16) -> Self {
        let mut harness = Self::new();
        harness.timer.tima = tima;
        harness.timer.tma = tma;
        harness.timer.write(0xFF07, tac).unwrap();
        harness.timer.set_div(div);
        harness
    }

    /// Write to the TAC register
    pub fn write_tac(&mut self, value: u8) {
        self.timer.write(0xFF07, value).unwrap();
    }

    /// Write to the TIMA register
    pub fn write_tima(&mut self, value: u8) {
        self.timer.write(0xFF05, value).unwrap();
    }

    /// Write to the TMA register
    pub fn write_tma(&mut self, value: u8) {
        self.timer.write(0xFF06, value).unwrap();
    }

    /// Read the TIMA register
    pub fn read_tima(&self) -> u8 {
        self.timer.read(0xFF05).unwrap()
    }

    /// Step until TIMA changes value
    pub fn step_until_tima_change(&mut self) -> Result<TimerState, TimerError> {
        let start_tima = self.read_tima();
        let mut cycles = 0;

        while self.read_tima() == start_tima && cycles < 10000 {
            let state = self.step_cycles(1)?;
            cycles += 1;
            if cycles >= 10000 {
                return Err(TimerError::HardwareError(
                    "Timeout waiting for TIMA change".to_string(),
                ));
            }
        }

        Ok(self.timer.get_state())
    }

    /// Step the timer for a specific number of cycles and log the state
    #[instrument(skip(self), level = "debug")]
    pub fn step_cycles(&mut self, cycles: u32) -> Result<TimerState, TimerError> {
        let result = self.timer.step(cycles)?;

        // Log state
        self.cycle_log.push((
            self.timer.global_cycles,
            self.timer.state,
            self.timer.tima,
            self.timer.get_interrupt_flag(),
        ));

        debug!(
            cycles = cycles,
            state = ?result,
            tima = self.timer.tima,
            "Stepped timer"
        );

        Ok(result)
    }

    /// Run the timer until a specific state is reached or timeout
    #[instrument(skip(self), level = "debug")]
    pub fn run_until_state(
        &mut self,
        target_state: TimerState,
        max_cycles: u32,
    ) -> Result<u32, TimerError> {
        debug!(
            target_state = ?target_state,
            max_cycles = max_cycles,
            "Running until state"
        );

        let start_cycles = self.timer.global_cycles;

        for i in 0..max_cycles {
            let state = self.step_cycles(1)?;
            if state == target_state {
                let cycles_taken = i + 1;
                debug!(cycles_taken = cycles_taken, "Target state reached");
                return Ok(cycles_taken);
            }
        }

        Err(TimerError::HardwareError(format!(
            "Timeout waiting for state {:?} after {} cycles",
            target_state, max_cycles
        )))
    }

    /// Assert that the timer went through a specific sequence of states
    pub fn assert_sequence(&self, expected: &[(TimerState, u8, bool)]) {
        let actual = self
            .cycle_log
            .iter()
            .map(|(_, state, tima, interrupt)| (*state, *tima, *interrupt))
            .collect::<Vec<_>>();

        assert_eq!(
            actual.len(),
            expected.len(),
            "State sequence length mismatch"
        );

        for (i, (expected_item, actual_item)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_eq!(
                expected_item, actual_item,
                "State sequence mismatch at position {}: expected {:?}, got {:?}",
                i, expected_item, actual_item
            );
        }
    }

    /// Clear the log but keep the timer state
    pub fn clear_log(&mut self) {
        self.cycle_log.clear();
    }

    /// Reset both the timer and the log
    pub fn reset(&mut self) {
        self.timer = Timer::new();
        self.cycle_log.clear();
    }
}
