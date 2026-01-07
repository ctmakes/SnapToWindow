use crate::window_manager::{SnapPosition, WindowManager};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Halves
    let left_half = MenuItem::with_id(app, "left_half", "Left Half\t\t\t⌃⌥←", true, None::<&str>)?;
    let right_half = MenuItem::with_id(app, "right_half", "Right Half\t\t\t⌃⌥→", true, None::<&str>)?;
    let top_half = MenuItem::with_id(app, "top_half", "Top Half\t\t\t⌃⌥↑", true, None::<&str>)?;
    let bottom_half = MenuItem::with_id(app, "bottom_half", "Bottom Half\t\t⌃⌥↓", true, None::<&str>)?;

    // Quarters
    let top_left = MenuItem::with_id(app, "top_left", "Top Left\t\t\t⌃⌥U", true, None::<&str>)?;
    let top_right = MenuItem::with_id(app, "top_right", "Top Right\t\t\t⌃⌥I", true, None::<&str>)?;
    let bottom_left = MenuItem::with_id(app, "bottom_left", "Bottom Left\t\t⌃⌥J", true, None::<&str>)?;
    let bottom_right = MenuItem::with_id(app, "bottom_right", "Bottom Right\t\t⌃⌥K", true, None::<&str>)?;

    // Thirds
    let left_third = MenuItem::with_id(app, "left_third", "Left Third", true, None::<&str>)?;
    let center_third = MenuItem::with_id(app, "center_third", "Center Third", true, None::<&str>)?;
    let right_third = MenuItem::with_id(app, "right_third", "Right Third", true, None::<&str>)?;
    let left_two_thirds = MenuItem::with_id(app, "left_two_thirds", "Left Two Thirds", true, None::<&str>)?;
    let right_two_thirds = MenuItem::with_id(app, "right_two_thirds", "Right Two Thirds", true, None::<&str>)?;

    // Other actions
    let maximize = MenuItem::with_id(app, "maximize", "Maximize\t\t\t⌃⌥↵", true, None::<&str>)?;
    let center = MenuItem::with_id(app, "center", "Center\t\t\t\t⌃⌥C", true, None::<&str>)?;

    // Separators
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;

    // Settings and Quit
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit SnapToWindow", true, None::<&str>)?;

    let menu = Menu::with_items(
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
    )?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let position = match event.id.as_ref() {
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
