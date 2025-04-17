# TODO

## Core

- [x] Basic CPU instruction set implementation
  - [x] ALU operations (ADD, SUB, AND, XOR, OR, CP)
  - [x] Register loads (LD r,n)
  - [x] HALT instruction
  - [x] Flag handling for basic operations
  - [x] Bit operations (BIT, RES, SET)
  - [x] Rotation and shift operations
- [ ] Complete CPU opcode map
  - [x] ALU operations with registers
  - [x] Register-to-register transfers
  - [x] Stack operations (PUSH, POP)
  - [x] Jump and call instructions
  - [x] RST operations
  - [x] Bit manipulation (CB prefix)
  - [ ] Memory banking operations
- [ ] Add timing-accurate instruction execution
- [x] Implement MMU with banking support
- [ ] Implement DMA and HDMA transfers
- [ ] Add interrupt handling
- [ ] Build test harness for CPU (blargg test ROMs)

## PPU

- [ ] Implement background rendering
- [ ] Add sprite rendering and priority handling
- [ ] Support CGB palettes and tile attributes

## APU

- [ ] Add APU channel emulation
- [ ] Integrate audio output (via `cpal`)

## CLI

- [x] Basic project structure
- [x] Build configuration
  - [x] Development profile (debug, opt-level 1)
  - [x] Release profile (LTO, stripped, opt-level 3)
- [ ] ROM loading via command line
- [ ] Add --debug flag for CPU trace
- [ ] Add cycle benchmarking for emulation loop

## UX

- [ ] Key mapping and input config
- [ ] Frame limiter

## Debug

- [x] Basic test infrastructure
- [x] Flag handling verification
- [ ] Add disassembler/debugger module
- [ ] Toggle debug UI via feature flag
- [ ] Expand test coverage for CPU instructions
