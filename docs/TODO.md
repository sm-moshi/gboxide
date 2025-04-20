# TODO List

## High Priority

### Fix Failing Tests (48/54 passing) ❗

1. Timer System Tests
   - [ ] `test_timer_increment_overflow`: Fix state transition (Running vs Overflow)
   - [ ] `test_timer_overflow_delay`: Fix value mismatch (255 vs 0)
   - [ ] `test_tac_change_causes_timer_increment`: Fix increment value (70 vs 69)

2. Memory Management Tests
   - [ ] `test_dma_from_various_sources`: Fix value mismatch (1 vs 0)
   - [ ] `test_oam_access`: Fix value mismatch (66 vs 255)

3. PPU Tests
   - [ ] `test_sprite_rendering`: Fix color value calculation

### Core Components

- [x] Complete interrupt handling system
  - [x] Implement basic interrupt structure
  - [x] Add interrupt flags
  - [x] Implement interrupt vectors
  - [x] Add timing for interrupt handling
  - [x] Test interrupt behaviour

- [~] Enhance MMU implementation
  - [x] Basic memory access
  - [x] Memory bus trait
  - [x] Complete banking system
  - [x] Memory protection
  - [x] Test memory timing
  - [x] Echo RAM support
  - [x] OAM access
  - [ ] Fix DMA implementation
  - [ ] Fix OAM access value handling

- [~] Timer System Completion
  - [x] Basic timer registers
  - [x] Cycle counting
  - [x] DIV register implementation
  - [x] TIMA counter with overflow
  - [x] TMA modulo register
  - [x] TAC control register
  - [x] Four frequency modes
  - [x] Timer overflow delay
  - [x] DIV reset functionality
  - [x] TAC change handling
  - [x] Interrupt requests
  - [x] Fix critical timer edge cases:
    - [x] Timer increment overflow behavior
    - [x] Timer overflow delay timing
    - [x] TAC change increment handling
  - [x] Add additional edge case tests:
    - [x] TIMA write during overflow
    - [x] TMA write during reload
    - [x] Multiple TAC changes in sequence
  - [x] Complete timing accuracy verification
    - [x] Document timing diagrams
    - [x] Add cycle-accurate test cases
    - [x] Verify against hardware behaviour

- [ ] Code Quality Improvements
  - [ ] Fix clippy linting errors
  - [ ] Add missing const functions
  - [ ] Handle truncation warnings
  - [ ] Improve error handling in tests
  - [ ] Remove unwrap/expect usage
  - [ ] Update to latest dependencies:
    - [x] thiserror 2.0
    - [x] tracing 1.63
    - [x] tracing-subscriber 1.63
    - [x] tracing-log 1.56
    - [x] bincode 2.0
    - [x] clap 4.5
    - [x] test-case 3.3
    - [x] proptest 1.6
    - [x] mockall 0.13
    - [x] tempfile 3.19
    - [x] pretty_assertions 1.4.1

- [~] Begin PPU implementation
  - [~] Set up VRAM structure
  - [~] Implement tile data reading
  - [~] Add background rendering
  - [~] Basic sprite support
  - [ ] Fix sprite rendering colors
  - [~] Test PPU timing

## Medium Priority

- [~] Improve test coverage
  - [ ] Add blargg test ROMs
  - [ ] Implement mooneye-gb tests
  - [x] Add memory access tests
  - [x] Add basic timer tests
  - [x] Add timer frequency tests
  - [x] Add timer overflow tests
  - [x] Add DIV reset tests
  - [x] Add TAC change tests
  - [x] Add banking tests
  - [ ] Test CPU timing accuracy

- [ ] Enhance debugging capabilities
  - [ ] Add instruction stepping
  - [ ] Implement breakpoints
  - [ ] Add memory viewer
  - [ ] Add register display

- [ ] CLI improvements
  - [x] Basic ROM loading
  - [x] Memory bank support
  - [x] Debug output
  - [ ] Save state support
  - [ ] Debug commands
  - [ ] Performance profiling

## Low Priority

- [~] Documentation
  - [x] Add architecture overview
  - [x] Document memory map
  - [x] Document timer system
  - [ ] Add contribution guide
  - [ ] Update README

- [~] Build system
  - [x] Basic cargo workspace
  - [x] Test infrastructure
  - [x] Development profiles
  - [ ] Cross-platform testing
  - [ ] CI/CD pipeline
  - [ ] Release automation

## Completed ✅

- [x] Project structure setup
- [x] Basic CPU implementation
- [x] Initial MMU structure
- [x] Basic CLI frontend
- [x] Test infrastructure
- [x] Flag handling verification
- [x] Memory access testing
- [x] Echo RAM implementation
- [x] OAM access support
- [x] Memory bank controller support
- [x] Basic timer implementation
- [x] Development environment setup
- [x] Build profile optimization
- [x] Interrupt system implementation
- [x] CPU timing accuracy
- [x] Dependency updates to latest versions

## Notes

- Critical: Fix timer increment overflow test (Running vs Overflow state)
- Critical: Fix timer overflow delay test (255 vs 0)
- Critical: Fix TAC change increment test (70 vs 69)
- Critical: Fix DMA source value mismatch (1 vs 0)
- Critical: Fix OAM access value mismatch (66 vs 255)
- Critical: Fix sprite rendering color values
- Add comprehensive timer edge case test suite
- Document timer system behaviour and edge cases
- Verify timer interrupt timing accuracy
- Begin PPU implementation after timer system is stable
- Maintain test coverage as features are added
- Address code quality issues from linting
- Improve error handling in test code

# Note: All timer-specific tests now pass. Remaining test failures are outside the timer (MMU, PPU).
