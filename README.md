<p align="center">
  <img src="icon.ico" width="128" height="128" alt="BrewKeep Icon">
</p>

<h1 align="center">BrewKeep</h1>

<p align="center">
  <strong>Lightweight Windows awake/sleep tray utility</strong>
</p>

<p align="center">
  <a href="https://github.com/mwolfinspace/BrewKeep/releases/latest">
    <img src="https://img.shields.io/github/v/release/mwolfinspace/BrewKeep?style=flat-square&color=blue" alt="Release">
  </a>
  <a href="https://github.com/mwolfinspace/BrewKeep/releases">
    <img src="https://img.shields.io/github/downloads/mwolfinspace/BrewKeep/total?style=flat-square&color=green" alt="Downloads">
  </a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-blue?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/rust-stable%20%3E%3D1.75-orange?style=flat-square" alt="Rust">
</p>

---

**BrewKeep** is a super-lightweight, UI-less Windows utility that lives in your system tray. It either *holds the screen awake* (prevents turn-off) or *forces the screen to turn off* after a custom timeout, regardless of your Control Panel power settings. On exit, your original power configuration is fully restored.

## Features

| Feature | Description |
|---|---|
| **Hold Mode** | Prevents the screen from turning off — overrides any short timeout |
| **Force Sleep** | Forces screen off after 30s or 1min — overrides "Never" |
| **Paused** | No intervention — original power settings apply |
| **Quick Toggle** | Left-click the tray icon to instantly swap between Hold and Paused |
| **Auto-Start** | Toggle Windows auto-start directly from the tray menu |
| **Clean Exit** | Original power settings are always restored on exit, even on crash |
| **Self-Signed** | Shows as verified publisher locally via self-signed certificate |
| **Tiny Binary** | ~600 KB single static `.exe`, no runtime dependencies, no installer |

## Download

Download the latest `brewkeep.exe` from the [**Releases**](https://github.com/mwolfinspace/BrewKeep/releases/latest) page.

No installation needed — just run it.

## Usage

1. **Run** `brewkeep.exe` — a coffee icon appears in the system tray
2. **Left-click** the icon to quickly toggle **Hold** / **Paused**
3. **Right-click** for the full menu:
   - Hold (keep awake)
   - Force Sleep: 30s
   - Force Sleep: 1min
   - Paused
   - Auto-start with Windows
   - Exit

> On exit (or crash), BrewKeep automatically restores your original monitor timeout settings.

## How It Works

BrewKeep uses two Windows APIs:

- **`SetThreadExecutionState`** — Tells Windows the system and display are in use, preventing sleep/screensaver
- **Power Scheme API** (`PowerReadACValueIndex` / `PowerWriteACValueIndex`) — Directly modifies the active power plan's monitor timeout values

On startup, it snapshots your current power settings. On exit (via the menu, process termination, or crash), the `Drop` guard restores them.

## Build from Source

### Requirements

- [Rust](https://rustup.org/) >= 1.75 (MSVC toolchain)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload
- Windows SDK

### Steps

```powershell
# Clone
git clone https://github.com/mwolfinspace/BrewKeep.git
cd BrewKeep

# Build release
cargo build --release

# The binary is at:
# target\release\brewkeep.exe
```

### Optional: Self-Sign

```powershell
.\scripts\self_sign.ps1
```

### Optional: Regenerate Icon

```powershell
.\scripts\make_icon.ps1
```

## Project Structure

```
BrewKeep/
├── Cargo.toml          # Dependencies and release profile
├── build.rs            # Embeds icon into the .exe
├── brewkeep.rc         # Windows resource file for the icon
├── icon.ico            # Tray icon (screen awake)
├── icon_off.ico        # Tray icon (screen off / paused)
├── scripts/
│   ├── make_icon.ps1   # Generates icon.ico from emoji
│   └── self_sign.ps1   # Creates self-signed certificate and signs the exe
└── src/
    ├── main.rs         # Entry point
    ├── power.rs        # Win32 Power API wrapper
    ├── autostart.rs    # Registry Run key management
    ├── state.rs        # Mode state machine
    └── tray.rs         # System tray icon, menus, event loop
```

## Tech Stack

| Component | Choice |
|---|---|
| Language | Rust (edition 2021) |
| Tray | [`tray-icon`](https://crates.io/crates/tray-icon) + [`tao`](https://crates.io/crates/tao) event loop |
| Win32 API | [`windows`](https://crates.io/crates/windows) crate |
| Icon | [`ico`](https://crates.io/crates/ico) crate |
| Build | `embed-resource` for icon embedding |
| Release | LTO + strip + opt-level=z (~600 KB binary) |

## License

[MIT](LICENSE)
