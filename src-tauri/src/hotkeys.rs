use crate::config::Config;
use crate::window_manager::{DisplayDirection, SnapPosition, WindowManager};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub fn register_hotkeys(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let shortcuts = &config.shortcuts;

    let shortcut_mappings = [
        (&shortcuts.left_half, SnapPosition::LeftHalf),
        (&shortcuts.right_half, SnapPosition::RightHalf),
        (&shortcuts.top_half, SnapPosition::TopHalf),
        (&shortcuts.bottom_half, SnapPosition::BottomHalf),
        (&shortcuts.top_left, SnapPosition::TopLeft),
        (&shortcuts.top_right, SnapPosition::TopRight),
        (&shortcuts.bottom_left, SnapPosition::BottomLeft),
        (&shortcuts.bottom_right, SnapPosition::BottomRight),
        (&shortcuts.left_third, SnapPosition::LeftThird),
        (&shortcuts.center_third, SnapPosition::CenterThird),
        (&shortcuts.right_third, SnapPosition::RightThird),
        (&shortcuts.left_two_thirds, SnapPosition::LeftTwoThirds),
        (&shortcuts.right_two_thirds, SnapPosition::RightTwoThirds),
        (&shortcuts.center, SnapPosition::Center),
        (&shortcuts.maximize, SnapPosition::Maximize),
    ];

    for (shortcut_str, position) in shortcut_mappings {
        let shortcut: Shortcut = shortcut_str.parse()?;
        let pos = position.clone();

        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
            let manager = WindowManager::new();
            if let Err(e) = manager.snap_to(pos.clone()) {
                eprintln!("Failed to snap window: {}", e);
            }
        })?;
    }

    // Register display movement shortcuts
    let display_mappings = [
        (&shortcuts.next_display, DisplayDirection::Next),
        (&shortcuts.previous_display, DisplayDirection::Previous),
    ];

    for (shortcut_str, direction) in display_mappings {
        let shortcut: Shortcut = shortcut_str.parse()?;
        let dir = direction.clone();

        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
            let manager = WindowManager::new();
            if let Err(e) = manager.move_to_display(dir.clone()) {
                eprintln!("Failed to move window to display: {}", e);
            }
        })?;
    }

    Ok(())
}
