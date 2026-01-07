use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shortcuts: ShortcutConfig,
    pub launch_at_login: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutConfig {
    pub left_half: String,
    pub right_half: String,
    pub top_half: String,
    pub bottom_half: String,
    pub top_left: String,
    pub top_right: String,
    pub bottom_left: String,
    pub bottom_right: String,
    pub left_third: String,
    pub center_third: String,
    pub right_third: String,
    pub left_two_thirds: String,
    pub right_two_thirds: String,
    pub center: String,
    pub maximize: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shortcuts: ShortcutConfig::default(),
            launch_at_login: false,
        }
    }
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            left_half: "CommandOrControl+Alt+Left".to_string(),
            right_half: "CommandOrControl+Alt+Right".to_string(),
            top_half: "CommandOrControl+Alt+Up".to_string(),
            bottom_half: "CommandOrControl+Alt+Down".to_string(),
            top_left: "CommandOrControl+Alt+U".to_string(),
            top_right: "CommandOrControl+Alt+I".to_string(),
            bottom_left: "CommandOrControl+Alt+J".to_string(),
            bottom_right: "CommandOrControl+Alt+K".to_string(),
            left_third: "CommandOrControl+Alt+D".to_string(),
            center_third: "CommandOrControl+Alt+F".to_string(),
            right_third: "CommandOrControl+Alt+G".to_string(),
            left_two_thirds: "CommandOrControl+Alt+E".to_string(),
            right_two_thirds: "CommandOrControl+Alt+R".to_string(),
            center: "CommandOrControl+Alt+C".to_string(),
            maximize: "CommandOrControl+Alt+Enter".to_string(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("snaptowindow");

        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path();

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
