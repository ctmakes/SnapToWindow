use crate::config::Config;
use crate::window_manager::{SnapPosition, WindowManager};

#[tauri::command]
pub fn snap_window(position: SnapPosition) -> Result<(), String> {
    let manager = WindowManager::new();
    manager.snap_to(position).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config() -> Result<Config, String> {
    Config::load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }

        unsafe { AXIsProcessTrusted() }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows and Linux don't need special accessibility permissions
        true
    }
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows doesn't need this, but we can open settings if needed
        std::process::Command::new("ms-settings:easeofaccess")
            .spawn()
            .ok();
    }

    Ok(())
}
