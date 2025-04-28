# gboxide Roadmap

## Current Status (2025-04)

✅ **Core Systems Complete and Audited**
- All core modules (CPU, MMU, Timer, PPU, APU, Cartridge, Interrupts, Bus, Helpers) are fully audited
- Codebase is warning-free and all tests pass
- Test code is idiomatic (no unwrap()/expect())
- Documentation is up to date
- The only remaining warnings are in test-only code and are suppressed with #[allow(dead_code)]

### Core Module Status
- [x] CPU implementation (complete instruction set, timing accuracy)
- [x] Timer system (cycle-accurate, hardware-accurate, all edge cases covered)
- [x] MMU implementation (DMG memory mapping, banking - GBC banking pending)
- [x] APU implementation (Audio logic done, tests pass - output pending)
- [x] Cartridge support (MBC1/2/3/5 + RTC, tests pass)
- [x] Interrupts (Hardware accurate timing, tests pass)
- [x] Bus (Routes IO, tests pass)
- [x] Helpers (Audited, tests pass)
- [x] PPU implementation
  - [x] Basic BG/Window Rendering & Timings
  - [x] Basic Sprite Rendering Logic
  - [x] Sprite collision detection (complete)
  - [ ] Accurate OAM Sprite Selection (10/line, priority)
  - [ ] Accurate Sprite-to-BG Priority
  - [ ] Accurate STAT Interrupts & Timings

🚧 **In Progress**
- Advanced PPU features (sprites, window, colour)
- Integration testing expansion
- CLI feature extensions
- Test suite expansion (blargg, mooneye-gb, property-based, snapshot)

## Short-term Goals (Q3 2025)

### PPU Completion
- [x] Complete advanced sprite features
  - [x] CGB sprite priority system
  - [x] Sprite collision detection
  - [ ] Edge case handling
- [ ] Enhance window rendering
  - [ ] Accurate timing
  - [ ] Edge case handling
- [ ] Optimise rendering pipeline
  - [ ] Memory access patterns
  - [ ] SIMD optimizations

### Testing Enhancement
- [ ] Expand integration test suite
  - [ ] Add more blargg test ROMs
  - [ ] Add mooneye-gb test ROMs
  - [ ] Add property-based tests
  - [ ] Add snapshot tests
- [ ] Improve test coverage
  - [ ] PPU core (currently 84%)
  - [ ] Sprite system (currently 100%)
  - [ ] Rendering (currently 62.26%)
  - [ ] PPU modes (currently 61.54%)

### CLI Improvements
- [ ] Add debugging features
  - [ ] Memory viewer
  - [ ] Register viewer
  - [ ] Breakpoint system
- [ ] Add profiling tools
  - [ ] Performance metrics
  - [ ] Memory usage tracking
- [ ] Add batch testing support
  - [ ] Test ROM automation
  - [ ] Results reporting

## Medium-term Goals (Q3-Q4 2025)

### GBC Compatibility
- [ ] VRAM Banking (SVBK 0xFF70)
- [ ] WRAM Banking (SVBK 0xFF70)
- [ ] Double-Speed Mode (KEY1 0xFF4D)
- [ ] CGB Palettes (BCPS/BCPD 0xFF68/9, OCPS/OCPD 0xFF6A/B)
- [ ] CGB BG Map Attributes (VRAM Bank 1 access)
- [ ] HDMA Transfers (HDMA1-5 0xFF51-5)

### Performance Optimization
- [ ] Implement SIMD optimizations
- [ ] Profile and optimize hot paths
- [ ] Improve memory access patterns
- [ ] Add benchmarking suite

### Feature Expansion
- [ ] Add save state support (Serialize/Deserialize via serde/bincode)
- [ ] Add rewind support
- [ ] Add cheat system
- [ ] Add screenshot/recording support
- [ ] Audio Output (Integrate cpal, buffer APU output)

### Documentation
- [ ] Add architecture documentation
- [ ] Add contributor guide
- [ ] Add user guide
- [ ] Add API documentation

## Long-term Goals (2025+)

### Advanced Features
- [ ] Add network play support
- [ ] Add debugger UI
  - [ ] Stepper/debugger support
  - [ ] Opcode logging
  - [ ] Frame-by-frame execution
  - [ ] Memory viewer/editor
  - [ ] Register viewer
  - [ ] PPU state visualizer
- [ ] Add ROM analysis tools
- [ ] Add performance profiling tools

### Platform Support
- [x] macOS support established
- [ ] Linux support
- [ ] Basic Windows/FreeBSD compatibility
- [ ] Add WebAssembly support
- [ ] Add mobile support
- [ ] Add console support
- [ ] Input abstraction layer
- [ ] Cross-platform testing

### Community
- [ ] Build contributor community
- [ ] Add plugin system
- [ ] Add mod support

### Optional GUI
- [ ] egui or tauri-based frontend
- [ ] Gamepad support
- [ ] Save state + load state
- [ ] Debug interface

## Completed Milestones 🎉

### Core Implementation ✅
- [x] Project setup and build configuration
  - [x] Workspace structure
  - [x] Build profiles optimized
  - [x] Linting rules established
- [x] CPU implementation
- [x] Memory system
- [x] Timer system
- [x] Basic PPU
- [x] APU system
- [x] Cartridge support
- [x] Interrupt handling
- [x] Memory bank controllers
- [x] Basic CLI

### Testing Infrastructure ✅
- [x] Unit test framework
- [x] Integration test framework
- [x] Test ROM support
- [x] CI/CD pipeline
- [x] Basic test infrastructure
- [x] Basic Logging (tracing)
- [x] Flag handling verification
- [x] Memory access testing
- [x] Memory bank controller testing
- [x] Timer system testing

### Code Quality ✅
- [x] Warning-free codebase
- [x] Idiomatic test code
- [x] Documentation coverage
- [x] Code formatting
- [x] Linting configuration
- [x] Update dependencies to latest versions
- [x] Improve error handling

### Recent Achievements 🌟
- [x] Fixed PPU sprite test and collision detection (all sprite logic now passes)
- [x] Completed APU audit
- [x] Implemented MBC3 RTC persistence
- [x] All core modules fully audited
- [x] Codebase is warning-free