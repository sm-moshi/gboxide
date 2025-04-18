# TODO List

## High Priority

- [x] Complete interrupt handling system
  - [x] Implement basic interrupt structure
  - [x] Add interrupt flags
  - [x] Implement interrupt vectors
  - [x] Add timing for interrupt handling
  - [x] Test interrupt behaviour

- [x] Enhance MMU implementation
  - [x] Basic memory access
  - [x] Memory bus trait
  - [x] Complete banking system
  - [x] Memory protection
  - [x] Test memory timing
  - [x] Echo RAM support
  - [x] OAM access
  - [ ] Implement DMA

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
  - [ ] Fix timer increment overflow test
  - [ ] Add additional edge case tests
  - [ ] Complete timing accuracy verification

- [ ] Begin PPU implementation
  - [ ] Set up VRAM structure
  - [ ] Implement tile data reading
  - [ ] Add background rendering
  - [ ] Basic sprite support
  - [ ] Test PPU timing

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

## Notes

- Fix timer increment overflow test as top priority
- Add more comprehensive timer edge case tests
- Verify timer interrupt timing accuracy
- Begin PPU implementation after timer system is stable
- Maintain test coverage as features are added
- Document timer system behaviour and edge cases
- Verify interrupt timing edge cases
- Add more comprehensive interrupt tests
