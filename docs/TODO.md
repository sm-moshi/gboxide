# TODO List

## High Priority

### PPU Fixes and Features

- [x] Fix failing PPU sprite test (`test_sprite_pixel_for_x_dmg_and_cgb`)
  - [x] Verify CGB bit math for x/y flipped sprites
  - [x] Ensure correct VRAM byte and bit index are read based on sprite attributes
  - [x] Confirm PPU state (LCDC, STAT) allows sprite visibility
  - [x] Implement accurate OAM sprite selection (10/line, priority)
  - [x] Implement accurate sprite-to-BG priority
  - [x] Verify LCD timings and STAT interrupts
  - [x] Implement and test sprite collision detection (all logic now passes)

### GBC Feature Implementation

- [ ] VRAM Banking (SVBK 0xFF70)
  - [ ] Implement bank selection (bits 0-2 select bank 1-7)
  - [ ] Map to 0xD000-0xDFFF range
- [ ] CGB Palettes
  - [ ] Implement BCPS/BCPD (0xFF68/9)
  - [ ] Implement OCPS/OCPD (0xFF6A/B)
  - [ ] Handle auto-increment
- [ ] CGB Tile Attributes
  - [ ] VRAM bank 1 access for BG map attributes
  - [ ] Implement BG Priority, VFlip, HFlip
  - [ ] Handle VRAM Bank selection and Palette numbers
- [ ] HDMA Transfers
  - [ ] Implement HDMA1-5 registers (0xFF51-5)
  - [ ] Support General Purpose and H-Blank DMA modes
- [ ] Double-Speed Mode
  - [ ] Implement KEY1 register (0xFF4D)
  - [ ] Handle speed switching

### Core Components

- [ ] Serial I/O Implementation
  - [ ] SB/SC registers
  - [ ] Clock selection
  - [ ] Transfer logic
  - [ ] Serial interrupts
- [ ] Audio Output
  - [ ] Integrate cpal
  - [ ] Buffer management
  - [ ] Sample rate handling

## Medium Priority

### Testing Enhancement

- [ ] Expand Test Coverage
  - [ ] Add blargg test ROMs
    - [ ] CPU tests
    - [ ] Instruction timing
    - [ ] Memory timing
    - [ ] OAM bug tests
  - [ ] Add mooneye-gb tests
    - [ ] PPU tests
    - [ ] Timer tests
    - [ ] Interrupt tests
    - [ ] MBC tests
  - [ ] Add property-based tests
    - [ ] CPU instruction logic
    - [ ] MMU banking logic
  - [ ] Add snapshot tests for PPU
  - [ ] Current coverage targets:
    - [ ] PPU core (target: >84%, current: 84%)
    - [ ] Sprite system (target: 100%, current: 100%)
    - [ ] Rendering (target: >80%, current: 62.26%)
    - [ ] PPU modes (target: >80%, current: 61.54%)

### Debug Features

- [ ] Debugger Implementation
  - [ ] Instruction stepping
  - [ ] Breakpoint system
  - [ ] Memory viewer/editor
  - [ ] Register display
  - [ ] PPU state visualization
- [ ] CLI Enhancements
  - [ ] Save state support
  - [ ] Debug commands
  - [ ] Performance profiling

### Documentation

- [ ] Architecture Documentation
  - [ ] System overview
  - [ ] Memory map (including GBC)
  - [ ] PPU internals
  - [ ] Timer system
  - [ ] Serial I/O
- [ ] User/Developer Guides
  - [ ] Contribution guide
  - [ ] README updates
  - [ ] API documentation

## Completed ✅

### Core Systems
- [x] Most core modules fully audited and compliant
- [x] CPU implementation complete
- [x] Memory system (basic) complete
- [x] Timer system complete and accurate
- [x] Basic PPU implementation
- [x] APU system fully audited
- [x] Cartridge support with MBC1/2/3/5
- [x] Interrupt handling system
- [x] Memory bank controllers
- [x] MBC3 RTC battery-backed persistence
- [x] Basic CLI frontend
- [x] PPU sprite test and collision detection (all logic now passes)

### Testing Infrastructure
- [x] Unit test framework
- [x] Integration test framework
- [x] Test ROM support
- [x] CI/CD pipeline
- [x] Memory access tests
- [x] Timer system tests
- [x] Banking tests
- [x] CPU timing accuracy tests

### Code Quality
- [x] Warning-free codebase (except allowed test warnings)
- [x] Idiomatic test code (no unwrap()/expect())
- [x] Documentation coverage for completed modules
- [x] Code formatting and linting
- [x] Dependency updates
- [x] Build profile optimization

## Current Status Notes

- All core modules (CPU, MMU, Timer, APU, Cartridge, Interrupts, Bus, Helpers) are fully audited, warning-free, and pass their tests
- All PPU sprite logic now passes; focus is on advanced PPU features, CGB support, and test coverage
- APU module is fully audited, warning-free, and all tests pass
- The only remaining warnings are in test-only code and are suppressed with #[allow(dead_code)]
- Test coverage is good for core modules and improved for PPU (84%) and sprite system (100%)
- Integration test crate with macro-based Mooneye GB test harness is active
- Error handling uses anyhow, tracing, and pretty_assertions
- Current focus is on:
  1. Implementing advanced PPU features
  2. Adding GBC support
  3. Expanding test coverage
  4. Improving debugging capabilities
