# TODO

## Core

- [ ] Implement full CPU opcode map
- [ ] Add timing-accurate instruction execution
- [ ] Implement MMU with banking support
- [ ] Implement DMA and HDMA transfers
- [ ] Add interrupt handling
- [ ] Build test harness for CPU (blargg test ROMs)
- [ ]

## PPU

- [ ] Implement background rendering
- [ ] Add sprite rendering and priority handling
- [ ] Support CGB palettes and tile attributes

## APU

- [ ] Add APU channel emulation
- [ ] Integrate audio output (via `cpal`)

## CLI

- [ ] ROM loading via command line
- [ ] Add --debug flag for CPU trace
- [ ] Add cycle benchmarking for emulation loop

## UX

- [ ] Key mapping and input config
- [ ] Frame limiter

## Debug

- [ ] Add disassembler/debugger module
- [ ] Toggle debug UI via feature flag
