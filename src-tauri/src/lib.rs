mod commands;
mod config;
mod hotkeys;
mod tray;
mod window_manager;

use tauri::Manager;

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
        .invoke_handler(tauri::generate_handler![
            commands::snap_window,
            commands::get_config,
            commands::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
