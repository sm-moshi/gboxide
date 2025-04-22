# TODO List

## High Priority

### Fix Failing Tests (now 54/54 passing) ✅

- [x] All timer, MMU, and PPU tests pass; codebase is warning-free and all test code is idiomatic (no `unwrap()`/`expect()`).

### Core Components

- [x] Complete interrupt handling system
  - [x] Implement basic interrupt structure
  - [x] Add interrupt flags
  - [x] Implement interrupt vectors
  - [x] Add timing for interrupt handling
  - [x] Test interrupt behaviour
- [x] MBC3 RTC battery-backed persistence
  - [x] RTC state serialised/deserialised with RAM (Pandocs-compliant)
  - [x] Fully tested with dedicated RTC persistence test 🦀

- [x] Enhance MMU implementation
  - [x] Basic memory access
  - [x] Memory bus trait
  - [x] Complete banking system
  - [x] Memory protection
  - [x] Test memory timing
  - [x] Echo RAM support
  - [x] OAM access
  - [x] Fix DMA implementation
  - [x] Fix OAM access value handling

- [x] Timer System Completion
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

- [x] Code Quality Improvements
- [x] Fix clippy linting errors
- [x] Add missing const functions
- [x] Handle truncation warnings
- [x] Improve error handling in tests
- [x] Remove unwrap/expect usage
- [x] Update to latest dependencies:
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
  - [x] Set up VRAM structure
  - [x] Implement tile data reading
  - [x] Add background rendering
  - [x] Basic sprite support
  - [x] Fix sprite rendering colors
  - [x] Test PPU timing

## Medium Priority

- [~] Improve test coverage
  - [ ] Add blargg test ROMs
  - [x] Implement mooneye-gb tests
  - [x] Add memory access tests
  - [x] Add basic timer tests
  - [x] Add timer frequency tests
  - [x] Add timer overflow tests
  - [x] Add DIV reset tests
  - [x] Add TAC change tests
  - [x] Add banking tests
  - [x] Test CPU timing accuracy

- [~] Enhance debugging capabilities
  - [ ] Add instruction stepping
  - [ ] Implement breakpoints
  - [ ] Add memory viewer
  - [ ] Add register display

- [ ] CLI improvements
  - [ ] Basic ROM loading
  - [ ] Memory bank support
  - [ ] Debug output
  - [ ] Save state support
  - [ ] Debug commands
  - [ ] Performance profiling

## Low Priority

- [ ] Documentation
  - [ ] Add architecture overview
  - [ ] Document memory map
  - [ ] Document timer system
  - [ ] Add contribution guide
  - [ ] Update README

- [~] Build system
  - [x] Basic cargo workspace
  - [~] Test infrastructure
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

## Test Infrastructure and Coverage 🦀

- [x] Integration test crate (`tests/`) with macro-based Mooneye GB test harness is now active
- [x] Error handling and diagnostics use `anyhow`, `tracing`, and `pretty_assertions`
- [x] MBC3 RTC persistence test: verifies serialisation, deserialisation, and time catch-up after reload 🦀
- [ ] Add `insta` for snapshot testing
- [ ] Add `proptest` for property-based testing
- [ ] Add `criterion` for performance benchmarking
- [ ] Expand macro to cover all Mooneye GB acceptance and common tests
- [ ] Implement property-based, snapshot, and performance tests for core logic and integration
- [ ] Document all new test strategies and patterns in the memory bank and docs

## Notes

- All timer, MMU, and PPU tests now pass. Codebase is warning-free and all test code is idiomatic.
- Next focus: advanced PPU features and integration. 🦀
