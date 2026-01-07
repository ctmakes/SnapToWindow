#[cfg(target_os = "macos")]
extern crate objc;

mod commands;
mod config;
mod hotkeys;
mod tray;
mod window_manager;

use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            // Initialize the system tray
            tray::setup_tray(app.handle())?;

            // Register global hotkeys
            hotkeys::register_hotkeys(app.handle())?;

            // Sync autostart state with config
            if let Ok(config) = config::Config::load() {
                let autostart_manager = app.autolaunch();
                if config.launch_at_login {
                    let _ = autostart_manager.enable();
                } else {
                    let _ = autostart_manager.disable();
                }
            }

            // Check for updates on startup (with delay) and periodically
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Small delay to let the app fully initialize
                std::thread::sleep(std::time::Duration::from_secs(2));
                tauri::async_runtime::block_on(tray::check_for_updates_startup(&app_handle));

                // Check for updates every hour
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(60 * 60));
                    tauri::async_runtime::block_on(tray::check_for_updates_startup(&app_handle));
                }
            });

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
            commands::refresh_tray,
            commands::set_update_available,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
