use serde::{Deserialize, Serialize};

/// Represents a rectangle with position and size.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// Represents a window with a platform-specific handle.
#[derive(Debug, Clone)]
pub struct Window {
    /// Platform-specific window handle.
    pub handle: WindowHandle,
    #[allow(dead_code)]
    pub title: String,
    pub frame: Rect,
}

/// Platform-specific window handle.
#[derive(Debug, Clone, Copy)]
pub enum WindowHandle {
    #[cfg(target_os = "windows")]
    Windows(isize),

    #[cfg(target_os = "macos")]
    MacOS(u32),

    #[cfg(target_os = "linux")]
    Linux(u64),
}

/// Represents a display/monitor.
#[derive(Debug, Clone)]
pub struct Display {
    #[allow(dead_code)]
    pub name: String,
    /// The full bounds of the display.
    pub bounds: Rect,
    /// The usable work area (excluding taskbar/dock/menubar).
    pub work_area: Rect,
    pub is_primary: bool,
}

/// The snap positions supported by the application.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapPosition {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Maximize,
    LeftThird,
    CenterThird,
    RightThird,
    LeftTwoThirds,
    RightTwoThirds,
}

impl SnapPosition {
    /// Calculate the frame for this snap position within the given work area.
    pub fn calculate_frame(&self, work_area: &Rect) -> Rect {
        let x = work_area.x;
        let y = work_area.y;
        let w = work_area.width;
        let h = work_area.height;

        match self {
            SnapPosition::LeftHalf => Rect::new(x, y, w / 2, h),
            SnapPosition::RightHalf => Rect::new(x + (w / 2) as i32, y, w / 2, h),
            SnapPosition::TopHalf => Rect::new(x, y, w, h / 2),
            SnapPosition::BottomHalf => Rect::new(x, y + (h / 2) as i32, w, h / 2),

            SnapPosition::TopLeft => Rect::new(x, y, w / 2, h / 2),
            SnapPosition::TopRight => Rect::new(x + (w / 2) as i32, y, w / 2, h / 2),
            SnapPosition::BottomLeft => Rect::new(x, y + (h / 2) as i32, w / 2, h / 2),
            SnapPosition::BottomRight => {
                Rect::new(x + (w / 2) as i32, y + (h / 2) as i32, w / 2, h / 2)
            }

            SnapPosition::Center => {
                let center_w = w * 2 / 3;
                let center_h = h * 2 / 3;
                Rect::new(
                    x + ((w - center_w) / 2) as i32,
                    y + ((h - center_h) / 2) as i32,
                    center_w,
                    center_h,
                )
            }

            SnapPosition::Maximize => Rect::new(x, y, w, h),

            SnapPosition::LeftThird => Rect::new(x, y, w / 3, h),
            SnapPosition::CenterThird => Rect::new(x + (w / 3) as i32, y, w / 3, h),
            SnapPosition::RightThird => Rect::new(x + (w * 2 / 3) as i32, y, w / 3, h),
            SnapPosition::LeftTwoThirds => Rect::new(x, y, w * 2 / 3, h),
            SnapPosition::RightTwoThirds => Rect::new(x + (w / 3) as i32, y, w * 2 / 3, h),
        }
    }
}
