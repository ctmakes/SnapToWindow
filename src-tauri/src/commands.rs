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
