#![cfg(target_os = "macos")]

use super::{Display, Rect, Result, Window, WindowHandle, WindowManagerError, WindowManagerTrait};
use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::display::{
    CGDirectDisplayID, CGDisplay, CGGetActiveDisplayList, CGMainDisplayID,
};
use core_graphics::window::{
    kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicI32, Ordering};

// Accessibility API types and constants
type AXUIElementRef = *mut c_void;
type AXError = i32;
type AXValueRef = *mut c_void;
type AXValueType = u32;

const AX_VALUE_TYPE_CG_POINT: AXValueType = 1;
const AX_VALUE_TYPE_CG_SIZE: AXValueType = 2;
const K_AX_ERROR_SUCCESS: AXError = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGSize {
    width: f64,
    height: f64,
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut *mut c_void,
    ) -> AXError;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *const c_void,
    ) -> AXError;
    fn AXValueCreate(value_type: AXValueType, value: *const c_void) -> AXValueRef;
    fn AXValueGetValue(value: AXValueRef, value_type: AXValueType, value_out: *mut c_void) -> bool;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

// NSScreen bindings for work area
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}

// Store the last known frontmost app PID for fallback when tray menu steals focus
static LAST_FRONTMOST_PID: AtomicI32 = AtomicI32::new(0);

pub struct MacOSManager;

impl MacOSManager {
    pub fn new() -> Self {
        Self
    }

    /// Get the topmost regular window's PID using CGWindowList (fallback method)
    fn get_topmost_window_pid(&self) -> Result<i32> {
        use core_foundation::base::CFType;

        unsafe {
            let options = kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
            let window_list = CGWindowListCopyWindowInfo(options, kCGNullWindowID);

            if window_list.is_null() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            let windows: CFArray<CFType> = CFArray::wrap_under_create_rule(window_list as _);
            let layer_key = CFString::new("kCGWindowLayer");
            let pid_key = CFString::new("kCGWindowOwnerPID");
            let name_key = CFString::new("kCGWindowOwnerName");

            // Find the first regular window (layer 0) that isn't our app
            for i in 0..windows.len() {
                let window_ptr = windows.get(i).map(|w| w.as_CFTypeRef());
                if window_ptr.is_none() {
                    continue;
                }

                let window_dict: CFDictionary<CFString, CFType> =
                    CFDictionary::wrap_under_get_rule(window_ptr.unwrap() as _);

                let layer = window_dict.find(&layer_key);
                let pid = window_dict.find(&pid_key);

                if let (Some(layer_ref), Some(pid_ref)) = (layer, pid) {
                    let layer_num = CFNumber::wrap_under_get_rule(layer_ref.as_CFTypeRef() as _);
                    let pid_num = CFNumber::wrap_under_get_rule(pid_ref.as_CFTypeRef() as _);

                    // Layer 0 is regular windows
                    if layer_num.to_i32() == Some(0) {
                        if let Some(pid_val) = pid_num.to_i32() {
                            // Skip our own app (SnapToWindow)
                            if let Some(name_ref) = window_dict.find(&name_key) {
                                let name_str =
                                    CFString::wrap_under_get_rule(name_ref.as_CFTypeRef() as _);
                                if name_str.to_string().contains("SnapToWindow") {
                                    continue;
                                }
                            }
                            return Ok(pid_val);
                        }
                    }
                }
            }

            Err(WindowManagerError::NoFocusedWindow)
        }
    }

    /// Get the PID of the frontmost application
    fn get_frontmost_app_pid(&self) -> Result<i32> {
        // Try the standard AX method first
        if let Ok(pid) = self.get_frontmost_app_pid_ax() {
            // Save as last known good PID
            LAST_FRONTMOST_PID.store(pid, Ordering::SeqCst);
            return Ok(pid);
        }

        // Fallback 1: Try to get topmost window from CGWindowList
        if let Ok(pid) = self.get_topmost_window_pid() {
            return Ok(pid);
        }

        // Fallback 2: Use last known frontmost PID
        let last_pid = LAST_FRONTMOST_PID.load(Ordering::SeqCst);
        if last_pid > 0 {
            return Ok(last_pid);
        }

        Err(WindowManagerError::NoFocusedWindow)
    }

    /// Get the PID of the frontmost application using Accessibility API
    fn get_frontmost_app_pid_ax(&self) -> Result<i32> {
        unsafe {
            let system_wide = AXUIElementCreateSystemWide();
            if system_wide.is_null() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            let attr_name = CFString::new("AXFocusedApplication");
            let mut focused_app: *mut c_void = ptr::null_mut();

            let result = AXUIElementCopyAttributeValue(
                system_wide,
                attr_name.as_concrete_TypeRef(),
                &mut focused_app,
            );

            core_foundation::base::CFRelease(system_wide as *const c_void);

            if result != K_AX_ERROR_SUCCESS || focused_app.is_null() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            // Get the PID from the focused application
            let pid_attr = CFString::new("AXPid");
            let mut pid_value: *mut c_void = ptr::null_mut();

            let result = AXUIElementCopyAttributeValue(
                focused_app,
                pid_attr.as_concrete_TypeRef(),
                &mut pid_value,
            );

            core_foundation::base::CFRelease(focused_app as *const c_void);

            if result != K_AX_ERROR_SUCCESS || pid_value.is_null() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            let pid_cf = CFNumber::wrap_under_create_rule(pid_value as _);
            let pid = pid_cf.to_i32().ok_or(WindowManagerError::NoFocusedWindow)?;

            Ok(pid)
        }
    }

    /// Get the focused window AXUIElement for an application
    fn get_focused_window_element(&self, pid: i32) -> Result<AXUIElementRef> {
        unsafe {
            let app_element = AXUIElementCreateApplication(pid);
            if app_element.is_null() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            let attr_name = CFString::new("AXFocusedWindow");
            let mut focused_window: *mut c_void = ptr::null_mut();

            let result = AXUIElementCopyAttributeValue(
                app_element,
                attr_name.as_concrete_TypeRef(),
                &mut focused_window,
            );

            core_foundation::base::CFRelease(app_element as *const c_void);

            if result != K_AX_ERROR_SUCCESS || focused_window.is_null() {
                return Err(WindowManagerError::NoFocusedWindow);
            }

            Ok(focused_window)
        }
    }

    /// Get the position of a window element
    fn get_window_position(&self, window: AXUIElementRef) -> Result<CGPoint> {
        unsafe {
            let attr_name = CFString::new("AXPosition");
            let mut value: *mut c_void = ptr::null_mut();

            let result =
                AXUIElementCopyAttributeValue(window, attr_name.as_concrete_TypeRef(), &mut value);

            if result != K_AX_ERROR_SUCCESS || value.is_null() {
                return Err(WindowManagerError::MoveError("Failed to get position".into()));
            }

            let mut point = CGPoint { x: 0.0, y: 0.0 };
            let success = AXValueGetValue(value, AX_VALUE_TYPE_CG_POINT, &mut point as *mut _ as _);

            core_foundation::base::CFRelease(value as *const c_void);

            if !success {
                return Err(WindowManagerError::MoveError("Failed to parse position".into()));
            }

            Ok(point)
        }
    }

    /// Get the size of a window element
    fn get_window_size(&self, window: AXUIElementRef) -> Result<CGSize> {
        unsafe {
            let attr_name = CFString::new("AXSize");
            let mut value: *mut c_void = ptr::null_mut();

            let result =
                AXUIElementCopyAttributeValue(window, attr_name.as_concrete_TypeRef(), &mut value);

            if result != K_AX_ERROR_SUCCESS || value.is_null() {
                return Err(WindowManagerError::MoveError("Failed to get size".into()));
            }

            let mut size = CGSize {
                width: 0.0,
                height: 0.0,
            };
            let success = AXValueGetValue(value, AX_VALUE_TYPE_CG_SIZE, &mut size as *mut _ as _);

            core_foundation::base::CFRelease(value as *const c_void);

            if !success {
                return Err(WindowManagerError::MoveError("Failed to parse size".into()));
            }

            Ok(size)
        }
    }

    /// Get the title of a window element
    fn get_window_title(&self, window: AXUIElementRef) -> String {
        unsafe {
            let attr_name = CFString::new("AXTitle");
            let mut value: *mut c_void = ptr::null_mut();

            let result =
                AXUIElementCopyAttributeValue(window, attr_name.as_concrete_TypeRef(), &mut value);

            if result != K_AX_ERROR_SUCCESS || value.is_null() {
                return String::new();
            }

            let title = CFString::wrap_under_create_rule(value as CFStringRef);
            title.to_string()
        }
    }

    /// Set the position of a window
    fn set_window_position(&self, window: AXUIElementRef, point: CGPoint) -> Result<()> {
        unsafe {
            let attr_name = CFString::new("AXPosition");
            let value = AXValueCreate(AX_VALUE_TYPE_CG_POINT, &point as *const _ as _);

            if value.is_null() {
                return Err(WindowManagerError::MoveError(
                    "Failed to create position value".into(),
                ));
            }

            let result =
                AXUIElementSetAttributeValue(window, attr_name.as_concrete_TypeRef(), value);

            core_foundation::base::CFRelease(value as *const c_void);

            if result != K_AX_ERROR_SUCCESS {
                return Err(WindowManagerError::MoveError(format!(
                    "Failed to set position: error {}",
                    result
                )));
            }

            Ok(())
        }
    }

    /// Set the size of a window
    fn set_window_size(&self, window: AXUIElementRef, size: CGSize) -> Result<()> {
        unsafe {
            let attr_name = CFString::new("AXSize");
            let value = AXValueCreate(AX_VALUE_TYPE_CG_SIZE, &size as *const _ as _);

            if value.is_null() {
                return Err(WindowManagerError::MoveError(
                    "Failed to create size value".into(),
                ));
            }

            let result =
                AXUIElementSetAttributeValue(window, attr_name.as_concrete_TypeRef(), value);

            core_foundation::base::CFRelease(value as *const c_void);

            if result != K_AX_ERROR_SUCCESS {
                return Err(WindowManagerError::MoveError(format!(
                    "Failed to set size: error {}",
                    result
                )));
            }

            Ok(())
        }
    }

    /// Get work area for a display using NSScreen
    fn get_display_work_area(&self, display_id: CGDirectDisplayID) -> Result<Rect> {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSScreen;
        use objc2_foundation::NSString;

        unsafe {
            // SAFETY: This code is called from the main thread in a Tauri app
            let mtm = MainThreadMarker::new_unchecked();
            let screens = NSScreen::screens(mtm);
            let screen_number_key = NSString::from_str("NSScreenNumber");

            // Get the primary screen height for coordinate conversion
            // NSScreen uses bottom-left origin, CG/AX uses top-left origin
            // The first screen in NSScreen.screens() is always the primary screen
            let primary_screen_height: f64 = screens
                .iter()
                .next()
                .map(|s| s.frame().size.height)
                .unwrap_or(0.0);

            for screen in screens.iter() {
                let device_desc = screen.deviceDescription();

                // Use msg_send for dictionary lookup to avoid type issues
                let screen_number_obj: *mut AnyObject =
                    msg_send![&*device_desc, objectForKey: &*screen_number_key];

                if !screen_number_obj.is_null() {
                    let num: u32 = msg_send![screen_number_obj, unsignedIntValue];

                    if num == display_id {
                        let visible_frame = screen.visibleFrame();

                        // Convert NSScreen coordinates (bottom-left origin) to CG coordinates (top-left origin)
                        // In NSScreen: y=0 is at bottom of primary screen, positive y goes up
                        // In CG/AX: y=0 is at top of primary screen, positive y goes down
                        // CG_y = primary_height - NSScreen_y - height
                        let cg_y = primary_screen_height
                            - visible_frame.origin.y
                            - visible_frame.size.height;

                        return Ok(Rect::new(
                            visible_frame.origin.x as i32,
                            cg_y as i32,
                            visible_frame.size.width as u32,
                            visible_frame.size.height as u32,
                        ));
                    }
                }
            }

            // Fallback to display bounds
            let bounds = CGDisplay::new(display_id).bounds();
            Ok(Rect::new(
                bounds.origin.x as i32,
                bounds.origin.y as i32,
                bounds.size.width as u32,
                bounds.size.height as u32,
            ))
        }
    }
}

impl WindowManagerTrait for MacOSManager {
    fn get_focused_window(&self) -> Result<Window> {
        let pid = self.get_frontmost_app_pid()?;
        let window_element = self.get_focused_window_element(pid)?;

        let position = self.get_window_position(window_element)?;
        let size = self.get_window_size(window_element)?;
        let title = self.get_window_title(window_element);

        unsafe {
            core_foundation::base::CFRelease(window_element as *const c_void);
        }

        Ok(Window {
            handle: WindowHandle::MacOS(pid as u32),
            title,
            frame: Rect::new(
                position.x as i32,
                position.y as i32,
                size.width as u32,
                size.height as u32,
            ),
        })
    }

    fn set_window_frame(&self, window: &Window, frame: Rect) -> Result<()> {
        let pid = match window.handle {
            WindowHandle::MacOS(p) => p as i32,
        };

        let window_element = self.get_focused_window_element(pid)?;

        let target_position = CGPoint {
            x: frame.x as f64,
            y: frame.y as f64,
        };
        let target_size = CGSize {
            width: frame.width as f64,
            height: frame.height as f64,
        };

        // Always use move-then-size to avoid window spanning displays
        // Move to target position first, then resize
        self.set_window_position(window_element, target_position)?;
        self.set_window_size(window_element, target_size)?;

        // Set position again to ensure correct placement after resize
        // (some apps adjust position when resized)
        self.set_window_position(window_element, target_position)?;

        unsafe {
            core_foundation::base::CFRelease(window_element as *const c_void);
        }

        Ok(())
    }

    fn get_current_display(&self) -> Result<Display> {
        let window = self.get_focused_window()?;
        let displays = self.get_all_displays()?;
        display_for_frame(&window.frame, displays).ok_or(WindowManagerError::DisplayError)
    }

    fn get_all_displays(&self) -> Result<Vec<Display>> {
        unsafe {
            let mut display_ids: [CGDirectDisplayID; 16] = [0; 16];
            let mut display_count: u32 = 0;

            let result = CGGetActiveDisplayList(16, display_ids.as_mut_ptr(), &mut display_count);

            if result != 0 {
                return Err(WindowManagerError::DisplayError);
            }

            let main_display = CGMainDisplayID();
            let mut displays = Vec::new();

            for i in 0..display_count as usize {
                let display_id = display_ids[i];
                let cg_display = CGDisplay::new(display_id);
                let bounds = cg_display.bounds();

                let work_area = self.get_display_work_area(display_id)?;

                displays.push(Display {
                    name: format!("Display {}", i + 1),
                    bounds: Rect::new(
                        bounds.origin.x as i32,
                        bounds.origin.y as i32,
                        bounds.size.width as u32,
                        bounds.size.height as u32,
                    ),
                    work_area,
                    is_primary: display_id == main_display,
                });
            }

            Ok(displays)
        }
    }
}

impl Default for MacOSManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pick the display a window is on: the one it overlaps most, otherwise the
/// one nearest its centre (like Windows' `MONITOR_DEFAULTTONEAREST`).
///
/// Checking only whether the centre lies inside a display is not enough. When
/// a Desktop (Space) is moved to another monitor, macOS can leave its windows
/// larger than, or hanging off, the new screen, so the centre may sit outside
/// every display. Falling back to the primary display then snapped the window
/// back onto the old monitor and its other Desktop.
fn display_for_frame(frame: &Rect, displays: Vec<Display>) -> Option<Display> {
    let overlap = |d: &Rect| -> i64 {
        let left = frame.x.max(d.x) as i64;
        let top = frame.y.max(d.y) as i64;
        let right = (frame.x as i64 + frame.width as i64).min(d.x as i64 + d.width as i64);
        let bottom = (frame.y as i64 + frame.height as i64).min(d.y as i64 + d.height as i64);
        (right - left).max(0) * (bottom - top).max(0)
    };

    let cx = frame.x as f64 + frame.width as f64 / 2.0;
    let cy = frame.y as f64 + frame.height as f64 / 2.0;
    let distance_sq = |d: &Rect| -> f64 {
        let dx = (d.x as f64 - cx).max(0.0).max(cx - (d.x as f64 + d.width as f64));
        let dy = (d.y as f64 - cy).max(0.0).max(cy - (d.y as f64 + d.height as f64));
        dx * dx + dy * dy
    };

    if let Some(best) = displays
        .iter()
        .filter(|d| overlap(&d.bounds) > 0)
        .max_by_key(|d| overlap(&d.bounds))
    {
        return Some(best.clone());
    }

    displays
        .into_iter()
        .min_by(|a, b| distance_sq(&a.bounds).total_cmp(&distance_sq(&b.bounds)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(name: &str, x: i32, y: i32, w: u32, h: u32, primary: bool) -> Display {
        Display {
            name: name.into(),
            bounds: Rect::new(x, y, w, h),
            work_area: Rect::new(x, y, w, h),
            is_primary: primary,
        }
    }

    // MacBook (primary) on the left, larger external monitor on the right.
    fn setup() -> Vec<Display> {
        vec![
            display("laptop", 0, 0, 1512, 982, true),
            display("external", 1512, -400, 2560, 1440, false),
        ]
    }

    #[test]
    fn window_inside_external_picks_external() {
        let frame = Rect::new(1700, -300, 800, 600);
        assert_eq!(display_for_frame(&frame, setup()).unwrap().name, "external");
    }

    #[test]
    fn window_hanging_off_external_with_centre_outside_picks_external() {
        // Centre is below the external's bottom edge (y = 1040) and right of the
        // laptop, so no display contains it. The old code fell back to the laptop.
        let frame = Rect::new(2000, 700, 1200, 1000);
        assert_eq!(display_for_frame(&frame, setup()).unwrap().name, "external");
    }

    #[test]
    fn window_spanning_both_picks_larger_overlap() {
        let frame = Rect::new(1300, 0, 1000, 600);
        assert_eq!(display_for_frame(&frame, setup()).unwrap().name, "external");
    }

    #[test]
    fn window_off_every_screen_picks_nearest() {
        let frame = Rect::new(5000, 0, 400, 300);
        assert_eq!(display_for_frame(&frame, setup()).unwrap().name, "external");
    }
}
