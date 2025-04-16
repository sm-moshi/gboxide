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
- Enhanced test infrastructure
  - Flag handling verification
  - Register operation tests
  - ALU operation validation
- Build system improvements
  - Optimized development profile (debug, opt-level 1)
  - Enhanced release profile (LTO, stripped, opt-level 3)
  - Comprehensive linting configuration

### Changed

- Upgraded Rust version requirement to 1.78.0
- Optimized build configurations
  - Development: debug symbols, opt-level 1
  - Release: LTO, stripped, opt-level 3
- Restructured CPU implementation
  - Consolidated ALU operations
  - Improved flag handling accuracy
  - Enhanced instruction documentation

### Fixed

- Flag handling in ADD HL,rr instruction
- Redundant ALU operation blocks removed
- Duplicate macro definitions consolidated
- Register operation consistency
- Stack operation timing

## [0.1.0] - 2024-03-20

### Added

- Initial project setup
- Basic project structure
- Core crate skeleton
- CLI crate skeleton
