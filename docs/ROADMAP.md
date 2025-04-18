# Roadmap

## Milestone 1: Minimal Emulator (DMG mode) [IN PROGRESS]

- [x] Project setup and build configuration
  - [x] Workspace structure
  - [x] Build profiles optimized
  - [x] Linting rules established
- [~] CPU implementation
  - [x] Basic instruction set
  - [x] Complete opcode map
  - [x] Flag handling
  - [x] Basic timing
  - [ ] Complete timing accuracy
  - [x] Interrupt system
- [~] MMU implementation
  - [x] Basic memory access
  - [x] Memory bus trait
  - [x] Initial cartridge support
  - [x] Memory banking
  - [x] Memory protection
  - [x] Echo RAM support
  - [x] OAM access
  - [ ] DMA transfers
- [~] Timer System
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
  - [ ] Fix timer increment overflow test
  - [ ] Complete edge case handling
  - [ ] Verify timing accuracy
- [~] CLI frontend
  - [x] Basic ROM execution
  - [x] Test program support
  - [x] Debug output
  - [x] Memory bank support
  - [ ] Full ROM compatibility
- [ ] PPU implementation
  - [ ] Basic VRAM access
  - [ ] Background rendering
  - [ ] Sprite support
- [ ] Input handling
  - [ ] Keyboard input (directional + A/B)
  - [ ] Input interrupts

## Milestone 2: GBC Compatibility

- [ ] Add VRAM and OAM banking
- [ ] Implement double-speed mode
- [ ] Implement CGB tile attribute memory and palette logic
- [ ] HDMA transfers
- [ ] Color palette support

## Milestone 3: Accuracy + Feature Completeness

- [ ] Pass all blargg CPU instruction tests
- [ ] Pass mooneye-gb PPU timing tests
- [ ] Add audio output with timing sync
- [ ] Implement save states
- [x] Add memory protection system
- [x] Add timer system core functionality
- [ ] Complete timer accuracy
  - [ ] Fix timer increment overflow
  - [ ] Add edge case tests
  - [ ] Verify interrupt timing

## Milestone 4: Platform Support

- [x] macOS support established
- [ ] Linux support
- [ ] Basic Windows/FreeBSD compatibility
- [ ] Input abstraction layer
- [ ] Cross-platform testing

## Milestone 5: Debug + Dev Tools

- [x] Basic test infrastructure
- [x] Flag handling verification
- [x] Memory access testing
- [x] Memory bank controller testing
- [x] Timer system testing
- [ ] Stepper/debugger support
- [ ] Opcode logging
- [ ] Frame-by-frame execution
- [ ] Memory viewer/editor

## Milestone 6: Optional GUI

- [ ] egui or tauri-based frontend
- [ ] Gamepad support
- [ ] Save state + load state
- [ ] Debug interface

## Current Focus

- Fix timer increment overflow test
- Add comprehensive timer edge case tests
- Verify timer interrupt timing
- Verify interrupt edge cases
- Begin PPU implementation
- Expand test coverage with blargg tests
- Add DMA transfer support
