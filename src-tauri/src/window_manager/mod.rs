mod types;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

pub use types::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowManagerError {
    #[error("Failed to get focused window")]
    NoFocusedWindow,

    #[error("Failed to get display information")]
    DisplayError,

    #[error("Failed to move window: {0}")]
    MoveError(String),

    #[error("No adjacent display in that direction")]
    NoAdjacentDisplay,
}

pub type Result<T> = std::result::Result<T, WindowManagerError>;

/// Trait defining the platform-specific window management operations.
pub trait WindowManagerTrait: Send + Sync {
    /// Get the currently focused window.
    fn get_focused_window(&self) -> Result<Window>;

    /// Move and resize a window to the specified frame.
    fn set_window_frame(&self, window: &Window, frame: Rect) -> Result<()>;

    /// Get the display/monitor containing the focused window.
    fn get_current_display(&self) -> Result<Display>;

    /// Get all available displays.
    fn get_all_displays(&self) -> Result<Vec<Display>>;
}

/// The main WindowManager struct that delegates to platform-specific implementations.
pub struct WindowManager {
    #[cfg(target_os = "windows")]
    inner: windows::WindowsManager,

    #[cfg(target_os = "macos")]
    inner: macos::MacOSManager,

    #[cfg(target_os = "linux")]
    inner: linux::LinuxManager,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            inner: windows::WindowsManager::new(),

            #[cfg(target_os = "macos")]
            inner: macos::MacOSManager::new(),

            #[cfg(target_os = "linux")]
            inner: linux::LinuxManager::new(),
        }
    }

    /// Snap the focused window to the specified position.
    pub fn snap_to(&self, position: SnapPosition) -> Result<()> {
        let window = self.inner.get_focused_window()?;
        let display = self.inner.get_current_display()?;
        let frame = position.calculate_frame(&display.work_area);

        self.inner.set_window_frame(&window, frame)
    }

    /// Move the focused window to the next or previous display (maximized).
    pub fn move_to_display(&self, direction: DisplayDirection) -> Result<()> {
        let window = self.inner.get_focused_window()?;
        let current_display = self.inner.get_current_display()?;
        let mut displays = self.inner.get_all_displays()?;

        if displays.len() < 2 {
            return Err(WindowManagerError::NoAdjacentDisplay);
        }

        // Sort displays by X coordinate, then Y coordinate (left-to-right, top-to-bottom)
        displays.sort_by(|a, b| {
            a.bounds.x.cmp(&b.bounds.x).then(a.bounds.y.cmp(&b.bounds.y))
        });

        // Find the index of the current display
        let current_idx = displays
            .iter()
            .position(|d| d.bounds.x == current_display.bounds.x && d.bounds.y == current_display.bounds.y)
            .ok_or(WindowManagerError::DisplayError)?;

        // Calculate target display index based on direction (wrap around)
        let target_idx = match direction {
            DisplayDirection::Next => (current_idx + 1) % displays.len(),
            DisplayDirection::Previous => {
                if current_idx == 0 {
                    displays.len() - 1
                } else {
                    current_idx - 1
                }
            }
        };

        let target_display = &displays[target_idx];

        // Maximize window on target display
        let new_frame = Rect::new(
            target_display.work_area.x,
            target_display.work_area.y,
            target_display.work_area.width,
            target_display.work_area.height,
        );

        self.inner.set_window_frame(&window, new_frame)
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}
