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
  - [ ] Timing accuracy
  - [ ] Interrupt system
- [ ] MMU + basic PPU
  - [ ] Memory banking
  - [ ] DMA transfers
  - [ ] Basic VRAM access
- [ ] Background rendering only
- [ ] Keyboard input (directional + A/B)
- [ ] Build CLI frontend

## Milestone 2: GBC Compatibility

- [ ] Add VRAM and OAM banking
- [ ] Implement double-speed mode
- [ ] Implement CGB tile attribute memory and palette logic
- [ ] HDMA transfers

## Milestone 3: Accuracy + Feature Completeness

- [ ] Pass all blargg CPU instruction tests
- [ ] Pass mooneye-gb PPU timing tests
- [ ] Add audio output with timing sync
- [ ] Implement save states

## Milestone 4: Platform Support

- [x] macOS support established
- [ ] Linux support
- [ ] Basic Windows/FreeBSD compatibility
- [ ] Input abstraction layer

## Milestone 5: Debug + Dev Tools

- [x] Basic test infrastructure
- [x] Flag handling verification
- [ ] Stepper/debugger support
- [ ] Opcode logging
- [ ] Frame-by-frame execution

## Milestone 6: Optional GUI

- [ ] egui or tauri-based frontend
- [ ] Gamepad support
- [ ] Save state + load state

## Current Focus

- Implement accurate timing for CPU instructions
- Set up MMU with banking support
- Add interrupt handling system
- Begin PPU implementation
- Expand test coverage with blargg tests
