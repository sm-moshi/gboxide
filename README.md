# 🕹️ gboxide

A fast, modular Game Boy Color emulator written in Rust with full DMG compatibility.

## 🎯 Goals

- ✅ Accurate emulation of CPU, MMU, PPU, Timer, and Joypad
- ✅ All test code is idiomatic (no `unwrap()`/`expect()`), and the codebase is warning-free
- ✅ All timer, MMU, and PPU tests pass
- 🟨 GBC support (DMA, palettes, VRAM banking, speed switch)
- 🛠️ Modular codebase for extensibility
- 🔧 Feature flags for DMG/CGB modes and debugging
- 🚀 CLI runner (GUI coming soon)

## 📦 Crates

- `core-lib/` – Emulator backend (CPU, MMU, PPU, etc.)
- `cli/` – Command-line frontend to run ROMs
- `common/` – Logging, types, shared utils
- `tests/` – Integration tests with test ROMs

## 🧪 Dev Setup

```bash
git clone https://github.com/sm-moshi/gboxide
cd gboxide
cargo build --workspace
cargo test --workspace
```

## ⚙️ Feature Flags

- `cgb` (default)
- `dmg`
- `debug`
- `logging`

## 🚦 Project Status

- All test code is idiomatic and warning-free
- All timer, MMU, and PPU tests pass
- Next focus: advanced PPU features and integration 🦀

## 📄 License

MIT OR Apache-2.0
