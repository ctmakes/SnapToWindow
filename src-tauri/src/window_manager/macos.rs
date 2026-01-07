#![cfg(target_os = "macos")]

use super::{Display, Rect, Result, Window, WindowHandle, WindowManagerError, WindowManagerTrait};

pub struct MacOSManager;

impl MacOSManager {
    pub fn new() -> Self {
        Self
    }
}

impl WindowManagerTrait for MacOSManager {
    fn get_focused_window(&self) -> Result<Window> {
        // TODO: Implement using Accessibility API
        // Use AXUIElementCreateSystemWide() and AXUIElementCopyAttributeValue()
        // to get the focused application and window
        Err(WindowManagerError::PlatformNotSupported)
    }

    fn set_window_frame(&self, _window: &Window, _frame: Rect) -> Result<()> {
        // TODO: Implement using Accessibility API
        // Use AXUIElementSetAttributeValue() with kAXPositionAttribute and kAXSizeAttribute
        Err(WindowManagerError::PlatformNotSupported)
    }

    fn get_current_display(&self) -> Result<Display> {
        // TODO: Implement using Core Graphics
        // Use CGMainDisplayID() or iterate displays to find the one containing the window
        Err(WindowManagerError::PlatformNotSupported)
    }

    fn get_all_displays(&self) -> Result<Vec<Display>> {
        // TODO: Implement using Core Graphics
        // Use CGGetActiveDisplayList() to get all displays
        // Use CGDisplayBounds() and NSScreen.visibleFrame for work area
        Err(WindowManagerError::PlatformNotSupported)
    }
}

impl Default for MacOSManager {
    fn default() -> Self {
        Self::new()
    }
}
