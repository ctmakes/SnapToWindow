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

## Supported Window Positions

| Action | Default Shortcut |
|--------|------------------|
| Left Half | `Ctrl/Cmd + Option + Left` |
| Right Half | `Ctrl/Cmd + Option + Right` |
| Top Half | `Ctrl/Cmd + Option + Up` |
| Bottom Half | `Ctrl/Cmd + Option + Down` |
| Top Left Quarter | `Ctrl/Cmd + Option + U` |
| Top Right Quarter | `Ctrl/Cmd + Option + I` |
| Bottom Left Quarter | `Ctrl/Cmd + Option + J` |
| Bottom Right Quarter | `Ctrl/Cmd + Option + K` |
| Center | `Ctrl/Cmd + Option + C` |
| Maximize | `Ctrl/Cmd + Option + Enter` |

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

### Core Trait

```rust
pub trait WindowManagerTrait: Send + Sync {
    /// Get the currently focused window
    fn get_focused_window(&self) -> Result<Window>;

    /// Move and resize a window to the specified frame
    fn set_window_frame(&self, window: &Window, frame: Rect) -> Result<()>;

    /// Get the display/monitor containing the focused window
    fn get_current_display(&self) -> Result<Display>;

    /// Get all available displays
    fn get_all_displays(&self) -> Result<Vec<Display>>;
}
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
  - **Linux**: `build-essential`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

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
- [ ] Settings UI
- [ ] Linux support (X11)
- [ ] Linux support (Wayland)
- [ ] Custom shortcut configuration UI
- [ ] Automatic updates

## Contributing

Contributions are welcome!

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [Rectangle](https://rectangleapp.com/) - The inspiration for this project
- [Tauri](https://tauri.app/) - The framework making this possible
