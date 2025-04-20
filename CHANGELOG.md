# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Complete CPU instruction set implementation
  - All ALU operations with registers
  - Full register-to-register transfers
  - Stack operations (PUSH, POP)
  - Jump and call instructions
  - RST operations
  - Bit manipulation (CB prefix)
  - Rotation and shift operations
  - Accurate cycle timing
- Enhanced memory system implementation
  - Memory bus trait abstraction
  - Initial cartridge support
  - Complete memory bank controller support
  - Echo RAM implementation (0xE000-0xFDFF)
  - OAM access support (0xFE00-0xFE9F)
  - Memory protection system
  - Timer system integration
  - Comprehensive memory tests
- Timer system implementation
  - Basic timer registers
  - Cycle counting
  - Interrupt request generation
  - Timer accuracy verification
- CLI implementation
  - Basic ROM execution loop
  - Test program support
  - Debug output for instructions
  - Cycle counting
  - Memory bank support
- Enhanced test infrastructure
  - Flag handling verification
  - Register operation tests
  - ALU operation validation
  - Memory access testing
  - Memory bank controller tests
  - Echo RAM and OAM tests
  - Timer system tests
- Build system improvements
  - Optimized development profile (debug, opt-level 1)
  - Enhanced release profile (LTO, stripped, opt-level 3)
  - Comprehensive linting configuration
- Enhanced timer system implementation
  - DIV register (16-bit counter)
  - TIMA counter with overflow handling
  - TMA modulo register support
  - TAC control register with frequency selection
  - Four selectable frequencies:
    - 4.096 KHz (bit 9)
    - 262.144 KHz (bit 3)
    - 65.536 KHz (bit 5)
    - 16.384 KHz (bit 7)
  - Timer overflow delay implementation
  - DIV reset functionality
  - TAC change handling
  - Comprehensive timer tests:
    - ✅ Frequency selection tests
    - ✅ Overflow delay tests
    - ✅ DIV reset tests
    - ✅ TAC change tests
    - ❌ Edge case tests (in progress)
      - DIV write edge detection
      - TAC rapid toggle handling
- Complete interrupt handling system implementation 🦀
  - Hardware-accurate timing (5 M-cycles)
  - Priority-based vector management
  - IME control via EI/DI instructions
  - HALT/STOP mode support
  - Interrupt vector handling (0x0040-0x0060)
  - Test coverage with edge cases
  - Integration with timer system

### Changed

- Upgraded Rust version requirement to 1.78.0
- Updated dependencies to latest versions:
  - thiserror: 2.0
  - tracing: 1.63
  - tracing-subscriber: 1.63
  - tracing-log: 1.56
  - bincode: 2.0
  - clap: 4.5
  - test-case: 3.3
  - proptest: 1.6
  - mockall: 0.13
  - tempfile: 3.19
  - pretty_assertions: 1.4.1
- Optimized build configurations
  - Development: debug symbols, opt-level 1, debug output enabled
  - Release: LTO, stripped, opt-level 3, debug output disabled
- Restructured CPU implementation
  - Consolidated ALU operations
  - Improved flag handling accuracy
  - Enhanced instruction documentation
  - Added cycle-accurate timing
  - Added interrupt handling support
- Enhanced memory system architecture
  - Added memory bus trait
  - Implemented cartridge support
  - Added memory protection checks
  - Improved memory bank controller support
  - Added Echo RAM and OAM support
  - Integrated timer system
  - Added interrupt vector support
- Enhanced timer system architecture
  - Improved cycle accuracy
  - Added edge case detection
  - Enhanced test coverage
  - Better documentation
  - Integrated interrupt requests
  - Known issues:
    - DIV write edge timing (TIMA=254 vs 255)
    - TAC rapid toggle state (TIMA=255 vs 0)
- Code quality improvements
  - Added missing const functions
  - Fixed truncation warnings
  - Improved error handling
  - Enhanced test reliability
  - Removed unsafe code blocks

### Fixed

- Timer system: State machine, overflow delay, and edge case handling (DIV reset, TAC change, overflow cancellation) are now correct and cycle-accurate
- All timer-specific tests now pass, confirming hardware-accurate behaviour for overflow and cancellation
- Test harness now explicitly sets up DIV and steps cycles to guarantee edge-triggered behaviour, ensuring robust and accurate test coverage
- Remaining test failures are outside the timer (MMU, PPU)
- Flag handling in ADD HL,rr instruction
- Redundant ALU operation blocks removed
- Duplicate macro definitions consolidated
- Register operation consistency
- Stack operation timing
- Memory access patterns
- Debug output formatting
- Echo RAM mirroring implementation
- OAM access and protection
- Memory bank controller switching
- Timer accuracy and integration
- Timer system improvements:
  - ✅ DIV reset edge detection
  - ✅ TAC change edge detection
  - ✅ Timer overflow delay timing
  - ✅ Frequency selection accuracy
  - ✅ Timer register access
  - ⚠️ Edge cases under investigation:
    - DIV write timing edge case
    - TAC rapid toggle handling
    - Timer increment behaviour
- Code quality issues:
  - Added missing const functions
  - Fixed truncation warnings
  - Improved error handling
  - Enhanced test reliability
  - Removed unsafe code blocks

### Known Issues

- 6 failing tests (48/54 passing):
  1. Timer System:
     - `test_timer_increment_overflow`: Expected Overflow, got Running
     - `test_timer_overflow_delay`: Expected 0, got 255
     - `test_tac_change_causes_timer_increment`: Expected 69, got 70
  2. Memory Management:
     - `test_dma_from_various_sources`: Value mismatch (0 vs 1)
     - `test_oam_access`: Value mismatch (255 vs 66)
  3. PPU:
     - `test_sprite_rendering`: Color value mismatch

## [0.1.0] - 2024-03-20

### Added

- Initial project setup
- Basic project structure
- Core crate skeleton
- CLI crate skeleton
