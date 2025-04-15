# 🕹️ gboxide

A fast, modular Game Boy Color emulator written in Rust with full DMG compatibility.

## 🎯 Goals

- ✅ Accurate emulation of CPU, MMU, PPU, Timer, and Joypad
- 🟨 GBC support (DMA, palettes, VRAM banking, speed switch)
- 🛠️ Modular codebase for extensibility
- 🔧 Feature flags for DMG/CGB modes and debugging
- 🚀 CLI runner (GUI coming soon)

## 📦 Crates

- `core/` – Emulator backend (CPU, MMU, PPU, etc.)
- `cli/` – Command-line frontend to run ROMs
- `common/` – Logging, types, shared utils
- `tests/` – Integration tests with test ROMs

## 🧪 Dev Setup

```bash
git clone https://github.com/sm-moshi/gboxide
cd gbc-rs
cargo build --workspace
cargo test --workspace
```

## ⚙️ Feature Flags

- `cgb` (default)
- `dmg`
- `debug`
- `logging`

## 📄 License

MIT OR Apache-2.0
