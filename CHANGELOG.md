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
  - Four selectable frequency modes:
    - 4.096 KHz (bit 9)
    - 262.144 KHz (bit 3)
    - 65.536 KHz (bit 5)
    - 16.384 KHz (bit 7)
  - Timer overflow delay implementation
  - DIV reset functionality
  - TAC change handling
  - Comprehensive timer tests:
    - Frequency selection tests
    - Overflow delay tests
    - DIV reset tests
    - TAC change tests
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
  - Added edge case handling
  - Enhanced test coverage
  - Better documentation
  - Integrated interrupt requests

### Fixed

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
  - DIV reset edge detection
  - TAC change edge detection
  - Timer overflow delay timing
  - Frequency selection accuracy
  - Timer register access

## [0.1.0] - 2024-03-20

### Added

- Initial project setup
- Basic project structure
- Core crate skeleton
- CLI crate skeleton
