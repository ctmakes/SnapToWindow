#![cfg(target_os = "windows")]

use super::{Display, Rect, Result, Window, WindowHandle, WindowManagerError, WindowManagerTrait};
use std::mem;
use std::ptr;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsIconic,
    IsWindowVisible, IsZoomed, SetWindowPos, ShowWindow, HWND_TOP, SET_WINDOW_POS_FLAGS,
    SWP_NOACTIVATE, SWP_NOZORDER, SW_RESTORE,
};

pub struct WindowsManager;

impl WindowsManager {
    pub fn new() -> Self {
        Self
    }

    /// Get the window title
    fn get_window_title(&self, hwnd: HWND) -> String {
        unsafe {
            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                return String::new();
            }

            let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
            let copied = GetWindowTextW(hwnd, &mut buffer);

            if copied == 0 {
                return String::new();
            }

            String::from_utf16_lossy(&buffer[..copied as usize])
        }
    }

    /// Get the window rectangle
    fn get_window_rect(&self, hwnd: HWND) -> Result<RECT> {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect)
                .map_err(|_| WindowManagerError::MoveError("Failed to get window rect".into()))?;
            Ok(rect)
        }
    }

    /// Check if window is maximized
    fn is_maximized(&self, hwnd: HWND) -> bool {
        unsafe { IsZoomed(hwnd).as_bool() }
    }

    /// Check if window is minimized
    fn is_minimized(&self, hwnd: HWND) -> bool {
        unsafe { IsIconic(hwnd).as_bool() }
    }

    /// Restore window if minimized or maximized
    fn restore_window(&self, hwnd: HWND) {
        unsafe {
            if self.is_minimized(hwnd) || self.is_maximized(hwnd) {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }
        }
    }

    /// Get the *visible* window bounds as drawn by DWM.
    ///
    /// Since Windows 10 most top-level windows carry an invisible resize border
    /// (the drop-shadow margin) that `GetWindowRect` includes but the user never
    /// sees. This attribute reports the frame the user actually perceives.
    fn get_extended_frame_bounds(&self, hwnd: HWND) -> Option<RECT> {
        unsafe {
            let mut rect = RECT::default();
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                &mut rect as *mut _ as *mut _,
                mem::size_of::<RECT>() as u32,
            )
            .ok()?;
            Some(rect)
        }
    }

    /// Measure the invisible border as `(left, top, right, bottom)` deltas from
    /// the `GetWindowRect` rect to the visible DWM rect.
    ///
    /// Typically `(9, 0, -9, -9)` at 150% scaling, `(0, 0, 0, 0)` for windows
    /// without a sizing frame. The values depend on the DPI of the monitor the
    /// window currently sits on, so this must be re-measured after every move.
    fn frame_offsets(&self, hwnd: HWND) -> (i32, i32, i32, i32) {
        let Ok(window_rect) = self.get_window_rect(hwnd) else {
            return (0, 0, 0, 0);
        };
        let Some(visible) = self.get_extended_frame_bounds(hwnd) else {
            return (0, 0, 0, 0);
        };

        // A minimized window parks at (-32000, -32000) and reports nonsense
        // bounds; refuse to derive offsets from it.
        if window_rect.left <= -32000 || visible.right <= visible.left {
            return (0, 0, 0, 0);
        }

        (
            visible.left - window_rect.left,
            visible.top - window_rect.top,
            visible.right - window_rect.right,
            visible.bottom - window_rect.bottom,
        )
    }

    /// Position a window so its *visible* frame lands exactly on `frame`,
    /// compensating for the invisible border measured by `offsets`.
    fn apply_frame(&self, hwnd: HWND, frame: Rect, offsets: (i32, i32, i32, i32)) -> Result<()> {
        let (dl, dt, dr, db) = offsets;

        // Solving `visible == frame` for the outer rect SetWindowPos expects:
        //   outer.left = frame.left - dl, outer.right = frame.right - dr
        let x = frame.x - dl;
        let y = frame.y - dt;
        let width = frame.width as i32 + (dl - dr);
        let height = frame.height as i32 + (dt - db);

        unsafe {
            let flags: SET_WINDOW_POS_FLAGS = SWP_NOZORDER | SWP_NOACTIVATE;

            SetWindowPos(hwnd, HWND_TOP, x, y, width.max(1), height.max(1), flags)
                .map_err(|e| WindowManagerError::MoveError(format!("SetWindowPos failed: {}", e)))?;
        }

        Ok(())
    }

    /// Convert RECT to our Rect type
    fn rect_from_win32(&self, rect: &RECT) -> Rect {
        Rect::new(
            rect.left,
            rect.top,
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32,
        )
    }

    /// Get monitor info from HMONITOR
    fn get_monitor_info(&self, hmonitor: HMONITOR) -> Result<MONITORINFOEXW> {
        unsafe {
            let mut info: MONITORINFOEXW = mem::zeroed();
            info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

            let result = GetMonitorInfoW(hmonitor, &mut info.monitorInfo);

            if result.as_bool() {
                Ok(info)
            } else {
                Err(WindowManagerError::DisplayError)
            }
        }
    }
}

impl WindowManagerTrait for WindowsManager {
    fn get_focused_window(&self) -> Result<Window> {
        unsafe {
            let hwnd = GetForegroundWindow();

            if hwnd.0 == ptr::null_mut() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            // Check if window is visible
            if !IsWindowVisible(hwnd).as_bool() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            let title = self.get_window_title(hwnd);

            // Report the visible frame so `Window.frame` means the same thing it
            // does on macOS — what the user sees, not the shadow-padded rect.
            let rect = match self.get_extended_frame_bounds(hwnd) {
                Some(visible) => visible,
                None => self.get_window_rect(hwnd)?,
            };

            Ok(Window {
                handle: WindowHandle::Windows(hwnd.0 as isize),
                title,
                frame: self.rect_from_win32(&rect),
            })
        }
    }

    fn set_window_frame(&self, window: &Window, frame: Rect) -> Result<()> {
        let hwnd = match window.handle {
            WindowHandle::Windows(h) => HWND(h as *mut _),
        };

        // Restore window first if it's minimized or maximized. The border width
        // differs between the maximized and restored states, so this has to
        // happen before we measure.
        self.restore_window(hwnd);

        let offsets = self.frame_offsets(hwnd);
        self.apply_frame(hwnd, frame, offsets)?;

        // The move may have landed the window on a monitor with a different
        // scale factor, which changes both the border width and — via
        // WM_DPICHANGED — the size the app settles on. Re-measure and correct
        // once; a no-op SetWindowPos is cheap when nothing changed.
        let settled = self.frame_offsets(hwnd);
        if settled != offsets {
            self.apply_frame(hwnd, frame, settled)?;
        }

        Ok(())
    }

    fn get_current_display(&self) -> Result<Display> {
        use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};

        let window = self.get_focused_window()?;
        let hwnd = match window.handle {
            WindowHandle::Windows(h) => HWND(h as *mut _),
        };

        unsafe {
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

            if hmonitor.is_invalid() {
                return Err(WindowManagerError::DisplayError);
            }

            let info = self.get_monitor_info(hmonitor)?;

            let is_primary = (info.monitorInfo.dwFlags & 1) != 0; // MONITORINFOF_PRIMARY = 1
            let name = String::from_utf16_lossy(
                &info.szDevice[..info.szDevice.iter().position(|&c| c == 0).unwrap_or(info.szDevice.len())]
            );

            Ok(Display {
                name,
                bounds: self.rect_from_win32(&info.monitorInfo.rcMonitor),
                work_area: self.rect_from_win32(&info.monitorInfo.rcWork),
                is_primary,
            })
        }
    }

    fn get_all_displays(&self) -> Result<Vec<Display>> {
        // We need to collect monitors using EnumDisplayMonitors
        // Using a static mut is not ideal, but EnumDisplayMonitors requires a callback

        struct MonitorCollector {
            monitors: Vec<HMONITOR>,
        }

        unsafe extern "system" fn enum_callback(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            unsafe {
                let collector = &mut *(lparam.0 as *mut MonitorCollector);
                collector.monitors.push(hmonitor);
            }
            TRUE
        }

        let mut collector = MonitorCollector {
            monitors: Vec::new(),
        };

        unsafe {
            let result = EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(enum_callback),
                LPARAM(&mut collector as *mut _ as isize),
            );

            if !result.as_bool() {
                return Err(WindowManagerError::DisplayError);
            }
        }

        let mut displays = Vec::new();

        for hmonitor in collector.monitors {
            let info = self.get_monitor_info(hmonitor)?;

            let is_primary = (info.monitorInfo.dwFlags & 1) != 0;
            let name = String::from_utf16_lossy(
                &info.szDevice[..info.szDevice.iter().position(|&c| c == 0).unwrap_or(info.szDevice.len())]
            );

            displays.push(Display {
                name,
                bounds: self.rect_from_win32(&info.monitorInfo.rcMonitor),
                work_area: self.rect_from_win32(&info.monitorInfo.rcWork),
                is_primary,
            });
        }

        Ok(displays)
    }
}

impl Default for WindowsManager {
    fn default() -> Self {
        Self::new()
    }
}
