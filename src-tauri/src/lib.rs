#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod commands;
mod config;
mod hotkeys;
mod tray;
mod window_manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Initialize the system tray
            tray::setup_tray(app.handle())?;

            // Register global hotkeys
            hotkeys::register_hotkeys(app.handle())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window instead of closing - app stays in tray
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().ok();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::snap_window,
            commands::get_config,
            commands::save_config,
            commands::check_accessibility,
            commands::open_accessibility_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
