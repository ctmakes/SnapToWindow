# SnapToWindow

A cross-platform (windows, macOS) window management application inspired by [Rectangle](https://rectangleapp.com/). Snap windows to predefined positions using keyboard shortcuts.

Built with **Tauri**, **Rust**, and **TypeScript** for native performance and a small footprint.

## Features

- **Window Snapping**: Snap windows to halves, quarters, thirds, and more
- **Keyboard Shortcuts**: Customizable hotkeys for all window actions
- **System Tray**: Runs quietly in the background with a system tray icon
- **Cross-Platform**: Works on Windows and macOS (Linux support planned)
- **Lightweight**: Minimal resource usage thanks to Tauri
- **Auto-Update**: Seamless background updates with Tauri's updater plugin
- **Launch at Login**: Optional auto-start when your computer boots

## Supported Window Positions

| Action | Windows | macOS |
|--------|---------|-------|
| Left Half | `Ctrl + Alt + Left` | `⌃ + ⌥ + Left` |
| Right Half | `Ctrl + Alt + Right` | `⌃ + ⌥ + Right` |
| Top Half | `Ctrl + Alt + Up` | `⌃ + ⌥ + Up` |
| Bottom Half | `Ctrl + Alt + Down` | `⌃ + ⌥ + Down` |
| Top Left Quarter | `Ctrl + Alt + U` | `⌃ + ⌥ + U` |
| Top Right Quarter | `Ctrl + Alt + I` | `⌃ + ⌥ + I` |
| Bottom Left Quarter | `Ctrl + Alt + J` | `⌃ + ⌥ + J` |
| Bottom Right Quarter | `Ctrl + Alt + K` | `⌃ + ⌥ + K` |
| Left Third | `Ctrl + Alt + D` | `⌃ + ⌥ + D` |
| Center Third | `Ctrl + Alt + F` | `⌃ + ⌥ + F` |
| Right Third | `Ctrl + Alt + G` | `⌃ + ⌥ + G` |
| Left Two Thirds | `Ctrl + Alt + E` | `⌃ + ⌥ + E` |
| Right Two Thirds | `Ctrl + Alt + R` | `⌃ + ⌥ + R` |
| Center | `Ctrl + Alt + C` | `⌃ + ⌥ + C` |
| Maximize | `Ctrl + Alt + Enter` | `⌃ + ⌥ + Enter` |

> **Note:** ⌃ = Control, ⌥ = Option

## Architecture

### Cross-Platform Design

The application uses Rust traits to abstract platform-specific window management APIs:

```
src-tauri/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports & plugin initialization
│   ├── commands.rs          # Tauri commands (IPC)
│   ├── hotkeys.rs           # Global hotkey registration
│   ├── tray.rs              # System tray management
│   ├── config.rs            # User configuration & shortcuts
│   └── window_manager/
│       ├── mod.rs           # WindowManager trait definition
│       ├── types.rs         # Rect, Window, Display, SnapPosition types
│       ├── windows.rs       # Windows implementation (Win32 API)
│       ├── macos.rs         # macOS implementation (Accessibility API)
│       └── linux.rs         # Linux implementation (X11/Wayland) [stub]
```

### Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | TypeScript, React, Tailwind CSS |
| Backend | Rust, Tauri 2.0 |
| Windows API | windows-rs (Win32) |
| macOS API | core-foundation, core-graphics, cocoa, objc |
| Build | Cargo, npm, Vite |

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (20+)
- Platform-specific requirements:
  - **Windows**: Visual Studio Build Tools 2022
  - **macOS**: Xcode Command Line Tools
  - **Linux**: To be confirmed

### Setup

```bash
# Clone the repository
git clone https://github.com/chitaoling/snaptowindow.git
cd snaptowindow

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build for production
npm run tauri build
```

Binaries will be output to `src-tauri/target/release/bundle/`.

## Configuration

Configuration is stored in:
- **Windows**: `%APPDATA%\snaptowindow\config.json`
- **macOS**: `~/Library/Application Support/snaptowindow/config.json`
- **Linux**: `~/.config/snaptowindow/config.json`

## Platform Notes

### macOS

Requires Accessibility permissions. On first launch, you'll be prompted to grant access in **System Preferences > Privacy & Security > Accessibility**.

### Windows

No special permissions required. Works out of the box.

### Linux (Planned)

Will support both X11 and Wayland compositors through conditional compilation.

## Roadmap

- [x] Project setup
- [x] Core window management trait
- [x] macOS implementation (Accessibility API)
- [x] Windows implementation (Win32 API)
- [x] Global hotkey registration
- [x] System tray integration
- [x] Multi-monitor support
- [x] Settings UI
- [ ] Linux support (X11)
- [ ] Linux support (Wayland)
- [ ] Custom shortcut configuration UI
- [x] Automatic updates
- [x] Launch at Login

## Changelog

### v0.1.17
- Added "Launch at Login" toggle in system tray menu
- Uses native checkmark indicator for toggle state
- Syncs with system autostart settings (macOS Login Items, Windows Startup)

## Contributing

Contributions are welcome!

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.

## Acknowledgments

- [Rectangle](https://rectangleapp.com/) - The inspiration for this project
- [Tauri](https://tauri.app/) - The framework making this possible
