use crate::config::Config;
use crate::window_manager::{SnapPosition, WindowManager};
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
        (&shortcuts.center, SnapPosition::Center),
        (&shortcuts.maximize, SnapPosition::Maximize),
    ];

    for (shortcut_str, position) in shortcut_mappings {
        let shortcut: Shortcut = shortcut_str.parse()?;
        let pos = position.clone();

        app.global_shortcut().on_shortcut(shortcut, move |_app, _event| {
            let manager = WindowManager::new();
            if let Err(e) = manager.snap_to(pos.clone()) {
                eprintln!("Failed to snap window: {}", e);
            }
        })?;
    }

    Ok(())
}
