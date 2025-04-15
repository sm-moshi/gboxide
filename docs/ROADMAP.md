# Roadmap

## Milestone 1: Minimal Emulator (DMG mode)

- Boot DMG test ROMs
- CPU + MMU + basic PPU
- Background rendering only
- Keyboard input (directional + A/B)
- Build CLI frontend

## Milestone 2: GBC Compatibility

- Add VRAM and OAM banking
- Implement double-speed mode
- Implement CGB tile attribute memory and palette logic

## Milestone 3: Accuracy + Feature Completeness

- Pass all blargg CPU instruction tests
- Pass mooneye-gb PPU timing tests
- Add audio output with timing sync

## Milestone 4: Platform Support

- macOS and Linux support
- Basic Windows/FreeBSD compatibility
- Input abstraction layer

## Milestone 5: Debug + Dev Tools

- Stepper/debugger support
- Opcode logging
- Frame-by-frame execution

## Milestone 6: Optional GUI

- egui or tauri-based frontend
- Gamepad support
- Save state + load state
