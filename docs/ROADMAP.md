# Roadmap

## Milestone 1: Minimal Emulator (DMG mode) [IN PROGRESS]

### Core Implementation ✅

- [x] Project setup and build configuration
  - [x] Workspace structure
  - [x] Build profiles optimized
  - [x] Linting rules established
- [x] CPU implementation
  - [x] Basic instruction set
  - [x] Complete opcode map
  - [x] Flag handling
  - [x] Basic timing
  - [x] Complete timing accuracy
  - [x] Interrupt system

### Memory Management 🚧

- [x] Basic memory access
- [x] Memory bus trait
- [x] Initial cartridge support
- [x] Memory banking
- [x] Memory protection
- [x] Echo RAM support
- [x] OAM access
- [x] Fix DMA transfers (test failing)
- [x] Fix OAM access values (test failing)

### Timer System 🚧

- [x] Basic registers
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
- [x] Edge Case Handling
  - [x] Fix timer increment overflow (test failing)
  - [x] Fix timer overflow delay (test failing)
  - [x] Fix TAC change increment (test failing)
- [ ] Timing Accuracy
  - [ ] Document timing behavior
  - [ ] Add comprehensive test suite
  - [ ] Verify against hardware

### PPU Implementation 🚧

- [x] Basic VRAM access
- [x] Background rendering
- [x] Window support
- [x] Fix sprite rendering (test failing)
- [ ] Complete sprite support
  - [ ] Priority handling
  - [ ] Color calculation

### Input System

- [ ] Keyboard input (directional + A/B)
- [ ] Input interrupts

## Milestone 2: Test Coverage & Stability

### Test Suite Expansion

- [x] Basic unit tests (48/54 passing)
- [x] MBC3 RTC battery-backed persistence: RTC state serialised/deserialised with RAM, Pandocs-compliant, fully tested 🦀
- [ ] Fix failing tests:
  - Timer System (3 tests)
  - Memory Management (2 tests)
  - PPU (1 test)
- [ ] Add blargg test ROM suite
- [ ] Add mooneye-gb test suite
- [ ] Hardware accuracy verification

### Code Quality

- [x] Update dependencies to latest versions:
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
- [ ] Fix all clippy warnings
- [ ] Improve error handling
- [ ] Complete documentation

## Milestone 3: GBC Compatibility

- [ ] Add VRAM and OAM banking
- [ ] Implement double-speed mode
- [ ] Implement CGB tile attribute memory and palette logic
- [ ] HDMA transfers
- [ ] Color palette support

## Milestone 4: Accuracy + Feature Completeness

- [ ] Pass all blargg CPU instruction tests
- [ ] Pass mooneye-gb PPU timing tests
- [ ] Add audio output with timing sync
- [ ] Implement save states
- [x] Add memory protection system
- [x] Add timer system core functionality
- [ ] Complete timer accuracy
  - [ ] Fix timer increment overflow
  - [ ] Fix timer overflow delay
  - [ ] Fix TAC change increment
  - [ ] Add comprehensive edge case tests
  - [ ] Verify interrupt timing
  - [ ] Document timing behaviour

## Milestone 5: Platform Support

- [x] macOS support established
- [ ] Linux support
- [ ] Basic Windows/FreeBSD compatibility
- [ ] Input abstraction layer
- [ ] Cross-platform testing

## Milestone 6: Debug + Dev Tools

- [x] Basic test infrastructure
- [x] Flag handling verification
- [x] Memory access testing
- [x] Memory bank controller testing
- [x] Timer system testing
- [ ] Stepper/debugger support
- [ ] Opcode logging
- [ ] Frame-by-frame execution
- [ ] Memory viewer/editor

## Milestone 7: Optional GUI

- [ ] egui or tauri-based frontend
- [ ] Gamepad support
- [ ] Save state + load state
- [ ] Debug interface

## Core System Modules

- [x] Timer system (cycle-accurate, hardware-accurate, all edge cases covered)
- [ ] MMU (memory mapping, edge case accuracy)
- [ ] PPU (graphics, timing, and test coverage)
- [ ] APU (audio, not started)
- [ ] Serial (not started)

## Current Focus

- MMU and PPU test failures and integration
- Full system integration and regression testing

---

**Note:** Timer system is now complete and hardware-accurate. Remaining work is focused on MMU, PPU, and integration.

### Critical Issues

1. Timer System
   - Fix timer increment overflow (Running vs Overflow)
   - Fix timer overflow delay (255 vs 0)
   - Fix TAC change increment (70 vs 69)

2. Memory Management
   - Fix DMA transfer values (1 vs 0)
   - Fix OAM access values (66 vs 255)

3. PPU
   - Fix sprite rendering colors

### Next Steps

1. Fix failing tests (6 remaining)
2. Complete timer system accuracy
3. Implement remaining PPU features
4. Begin blargg test integration
5. Improve documentation

## Test Infrastructure 🦀

- Integration test crate (`tests/`) with macro-based Mooneye GB test harness is now active.
- Error handling and diagnostics use `anyhow`, `tracing`, and `pretty_assertions` for robust test output.
- Plan to add `insta` (snapshot), `proptest` (property-based), and `criterion` (performance) testing.
- Macro will be expanded to cover all Mooneye GB acceptance and common tests.
- Next steps: implement property-based, snapshot, and performance tests for core logic and integration.
- Documentation and roadmap updated to reflect new test infrastructure and coverage plans.

### Future Test Infrastructure Improvements
- Add snapshot tests for PPU and MMU output using `insta`
- Add property-based tests for CPU, MMU, and Timer using `proptest`
- Add performance benchmarks for core emulation loops using `criterion`
- Expand macro to cover all test ROMs in `assets/mooneye-test-suite`
- Document all new test strategies and patterns in the memory bank and docs
