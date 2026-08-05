use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuItem {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub desktop_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatteryState {
    pub conservation_mode_enabled: bool,
    pub percentage: Option<u8>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdleState {
    pub is_inhibited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSaver,
    Unknown(String),
}

impl PowerProfile {
    pub fn as_str(&self) -> &str {
        match self {
            PowerProfile::Performance => "performance",
            PowerProfile::Balanced => "balanced",
            PowerProfile::PowerSaver => "power-saver",
            PowerProfile::Unknown(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "performance" | "perf" => PowerProfile::Performance,
            "balanced" | "bal" => PowerProfile::Balanced,
            "power-saver" | "saver" | "quiet" => PowerProfile::PowerSaver,
            other => PowerProfile::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisplayLayout {
    Dual,
    PcOnly,
    ExternalOnly,
    Swap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub wallpaper_folder: String,
    pub screenshot_folder: String,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/user"));
        Self {
            wallpaper_folder: home.join("Pictures/Wallpapers").to_string_lossy().to_string(),
            screenshot_folder: home.join("Pictures/screenshots").to_string_lossy().to_string(),
            theme: "dark".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_profile_parsing() {
        assert_eq!(PowerProfile::from_str("performance"), PowerProfile::Performance);
        assert_eq!(PowerProfile::from_str("perf"), PowerProfile::Performance);
        assert_eq!(PowerProfile::from_str("balanced"), PowerProfile::Balanced);
        assert_eq!(PowerProfile::from_str("power-saver"), PowerProfile::PowerSaver);
        assert_eq!(PowerProfile::from_str("saver"), PowerProfile::PowerSaver);
        assert_eq!(PowerProfile::from_str("custom"), PowerProfile::Unknown("custom".to_string()));
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "dark");
        assert!(config.wallpaper_folder.contains("Wallpapers"));
        assert!(config.screenshot_folder.contains("screenshots"));
    }
}
