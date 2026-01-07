use crate::window_manager::{SnapPosition, WindowManager};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

#[cfg(target_os = "macos")]
fn check_accessibility() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
fn check_accessibility() -> bool {
    true
}

fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .ok();
    }
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let accessibility_enabled = check_accessibility();

    // Warning item (only shown if accessibility not enabled)
    let warning = MenuItem::with_id(
        app,
        "accessibility_warning",
        "⚠️ Accessibility Required",
        true,
        None::<&str>,
    )?;
    let warning_sep = PredefinedMenuItem::separator(app)?;

    // Halves
    let left_half = MenuItem::with_id(
        app,
        "left_half",
        "Left Half",
        accessibility_enabled,
        Some("ctrl+alt+left"),
    )?;
    let right_half = MenuItem::with_id(
        app,
        "right_half",
        "Right Half",
        accessibility_enabled,
        Some("ctrl+alt+right"),
    )?;
    let top_half = MenuItem::with_id(
        app,
        "top_half",
        "Top Half",
        accessibility_enabled,
        Some("ctrl+alt+up"),
    )?;
    let bottom_half = MenuItem::with_id(
        app,
        "bottom_half",
        "Bottom Half",
        accessibility_enabled,
        Some("ctrl+alt+down"),
    )?;

    // Quarters
    let top_left = MenuItem::with_id(
        app,
        "top_left",
        "Top Left",
        accessibility_enabled,
        Some("ctrl+alt+u"),
    )?;
    let top_right = MenuItem::with_id(
        app,
        "top_right",
        "Top Right",
        accessibility_enabled,
        Some("ctrl+alt+i"),
    )?;
    let bottom_left = MenuItem::with_id(
        app,
        "bottom_left",
        "Bottom Left",
        accessibility_enabled,
        Some("ctrl+alt+j"),
    )?;
    let bottom_right = MenuItem::with_id(
        app,
        "bottom_right",
        "Bottom Right",
        accessibility_enabled,
        Some("ctrl+alt+k"),
    )?;

    // Thirds
    let left_third = MenuItem::with_id(
        app,
        "left_third",
        "Left Third",
        accessibility_enabled,
        Some("ctrl+alt+d"),
    )?;
    let center_third = MenuItem::with_id(
        app,
        "center_third",
        "Center Third",
        accessibility_enabled,
        Some("ctrl+alt+f"),
    )?;
    let right_third = MenuItem::with_id(
        app,
        "right_third",
        "Right Third",
        accessibility_enabled,
        Some("ctrl+alt+g"),
    )?;
    let left_two_thirds = MenuItem::with_id(
        app,
        "left_two_thirds",
        "Left Two Thirds",
        accessibility_enabled,
        Some("ctrl+alt+e"),
    )?;
    let right_two_thirds = MenuItem::with_id(
        app,
        "right_two_thirds",
        "Right Two Thirds",
        accessibility_enabled,
        Some("ctrl+alt+r"),
    )?;

    // Other actions
    let maximize = MenuItem::with_id(
        app,
        "maximize",
        "Maximize",
        accessibility_enabled,
        Some("ctrl+alt+enter"),
    )?;
    let center = MenuItem::with_id(
        app,
        "center",
        "Center",
        accessibility_enabled,
        Some("ctrl+alt+c"),
    )?;

    // Separators
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;

    // Settings and Quit
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit SnapToWindow", true, None::<&str>)?;

    let menu = if accessibility_enabled {
        Menu::with_items(
            app,
            &[
                // Halves
                &left_half,
                &right_half,
                &top_half,
                &bottom_half,
                &sep1,
                // Quarters
                &top_left,
                &top_right,
                &bottom_left,
                &bottom_right,
                &sep2,
                // Thirds
                &left_third,
                &center_third,
                &right_third,
                &left_two_thirds,
                &right_two_thirds,
                &sep3,
                // Other
                &maximize,
                &center,
                &sep4,
                // App controls
                &settings,
                &quit,
            ],
        )?
    } else {
        Menu::with_items(
            app,
            &[
                // Warning at top
                &warning,
                &warning_sep,
                // Halves (disabled)
                &left_half,
                &right_half,
                &top_half,
                &bottom_half,
                &sep1,
                // Quarters (disabled)
                &top_left,
                &top_right,
                &bottom_left,
                &bottom_right,
                &sep2,
                // Thirds (disabled)
                &left_third,
                &center_third,
                &right_third,
                &left_two_thirds,
                &right_two_thirds,
                &sep3,
                // Other (disabled)
                &maximize,
                &center,
                &sep4,
                // App controls
                &settings,
                &quit,
            ],
        )?
    };

    let tooltip = if accessibility_enabled {
        "SnapToWindow"
    } else {
        "SnapToWindow - ⚠️ Accessibility Required"
    };

    let tray_icon =
        Image::from_bytes(include_bytes!("../icons/tray.png")).expect("Failed to load tray icon");

    let mut builder = TrayIconBuilder::new()
        .icon(tray_icon)
        .icon_as_template(true)
        .menu(&menu)
        .tooltip(tooltip)
        .show_menu_on_left_click(true);

    // Show warning indicator next to icon on macOS when accessibility is disabled
    if !accessibility_enabled {
        builder = builder.title("!");
    }

    builder
        .on_menu_event(|app, event| {
            let position = match event.id.as_ref() {
                // Accessibility warning
                "accessibility_warning" => {
                    open_accessibility_settings();
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                    None
                }
                // Halves
                "left_half" => Some(SnapPosition::LeftHalf),
                "right_half" => Some(SnapPosition::RightHalf),
                "top_half" => Some(SnapPosition::TopHalf),
                "bottom_half" => Some(SnapPosition::BottomHalf),
                // Quarters
                "top_left" => Some(SnapPosition::TopLeft),
                "top_right" => Some(SnapPosition::TopRight),
                "bottom_left" => Some(SnapPosition::BottomLeft),
                "bottom_right" => Some(SnapPosition::BottomRight),
                // Thirds
                "left_third" => Some(SnapPosition::LeftThird),
                "center_third" => Some(SnapPosition::CenterThird),
                "right_third" => Some(SnapPosition::RightThird),
                "left_two_thirds" => Some(SnapPosition::LeftTwoThirds),
                "right_two_thirds" => Some(SnapPosition::RightTwoThirds),
                // Other
                "maximize" => Some(SnapPosition::Maximize),
                "center" => Some(SnapPosition::Center),
                // Non-snap actions
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                    None
                }
                "quit" => {
                    app.exit(0);
                    None
                }
                _ => None,
            };

            if let Some(pos) = position {
                let manager = WindowManager::new();
                if let Err(e) = manager.snap_to(pos) {
                    eprintln!("Failed to snap window: {}", e);
                }
            }
        })
        .build(app)?;

    Ok(())
}
