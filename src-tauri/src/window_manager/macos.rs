#![cfg(target_os = "macos")]

use super::{Display, Rect, Result, Window, WindowHandle, WindowManagerError, WindowManagerTrait};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::display::{
    CGDirectDisplayID, CGDisplay, CGGetActiveDisplayList, CGMainDisplayID,
};
use std::ffi::c_void;
use std::ptr;

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
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
    fn AXIsProcessTrusted() -> bool;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGRectContainsPoint(rect: CGRect, point: CGPoint) -> bool;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

// NSScreen bindings for work area
#[link(name = "Foundation", kind = "framework")]
extern "C" {}

pub struct MacOSManager;

impl MacOSManager {
    pub fn new() -> Self {
        Self
    }

    /// Check if we have accessibility permissions
    pub fn is_trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    /// Get the PID of the frontmost application
    fn get_frontmost_app_pid(&self) -> Result<i32> {
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
        use cocoa::appkit::NSScreen;
        use cocoa::base::nil;
        use cocoa::foundation::NSArray;
        use objc::runtime::Object;

        unsafe {
            let screens: *mut Object = NSScreen::screens(nil);
            let count = NSArray::count(screens);

            for i in 0..count {
                let screen: *mut Object = NSArray::objectAtIndex(screens, i);
                let screen_dict: *mut Object =
                    objc::msg_send![screen, deviceDescription];
                let screen_number_key = CFString::new("NSScreenNumber");
                let screen_number: *mut Object = objc::msg_send![
                    screen_dict,
                    objectForKey: screen_number_key.as_concrete_TypeRef()
                ];

                if !screen_number.is_null() {
                    let num: u32 = objc::msg_send![screen_number, unsignedIntValue];
                    if num == display_id {
                        let visible_frame: cocoa::foundation::NSRect =
                            objc::msg_send![screen, visibleFrame];
                        let frame: cocoa::foundation::NSRect = objc::msg_send![screen, frame];

                        // NSScreen uses bottom-left origin, convert to top-left
                        let menu_bar_height =
                            (frame.size.height - visible_frame.size.height - visible_frame.origin.y)
                                as u32;

                        return Ok(Rect::new(
                            visible_frame.origin.x as i32,
                            menu_bar_height as i32,
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

        // Set position first, then size
        let position = CGPoint {
            x: frame.x as f64,
            y: frame.y as f64,
        };
        let size = CGSize {
            width: frame.width as f64,
            height: frame.height as f64,
        };

        self.set_window_position(window_element, position)?;
        self.set_window_size(window_element, size)?;

        unsafe {
            core_foundation::base::CFRelease(window_element as *const c_void);
        }

        Ok(())
    }

    fn get_current_display(&self) -> Result<Display> {
        // Get the focused window position to determine which display it's on
        let window = self.get_focused_window()?;
        let window_center = CGPoint {
            x: window.frame.x as f64 + (window.frame.width / 2) as f64,
            y: window.frame.y as f64 + (window.frame.height / 2) as f64,
        };

        let displays = self.get_all_displays()?;

        // Find the display containing the window center
        for display in &displays {
            let bounds = CGRect {
                origin: CGPoint {
                    x: display.bounds.x as f64,
                    y: display.bounds.y as f64,
                },
                size: CGSize {
                    width: display.bounds.width as f64,
                    height: display.bounds.height as f64,
                },
            };

            if unsafe { CGRectContainsPoint(bounds, window_center) } {
                return Ok(display.clone());
            }
        }

        // Fallback to primary display
        displays
            .into_iter()
            .find(|d| d.is_primary)
            .ok_or(WindowManagerError::DisplayError)
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
