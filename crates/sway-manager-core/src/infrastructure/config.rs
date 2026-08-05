use crate::domain::entities::AppConfig;
use crate::domain::traits::ConfigRepository;
use std::fs;
use std::path::PathBuf;

pub struct SystemConfigRepository;

impl SystemConfigRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn get_config_path() -> PathBuf {
        if std::env::var("SWAY_MANAGER_TEST_MODE").unwrap_or_default() == "1" {
            PathBuf::from("/tmp/sway_manager_tests/test_config.json")
        } else {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
            path.push("sway-manager");
            path.push("config.json");
            path
        }
    }
}

impl Default for SystemConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigRepository for SystemConfigRepository {
    fn load(&self) -> Result<AppConfig, String> {
        let path = Self::get_config_path();
        if !path.exists() {
            let default_config = AppConfig::default();
            let _ = self.save(&default_config);
            return Ok(default_config);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file at {:?}: {}", path, e))?;
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file at {:?}: {}", path, e))?;
        Ok(config)
    }

    fn save(&self, config: &AppConfig) -> Result<(), String> {
        let path = Self::get_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir {:?}: {}", parent, e))?;
        }
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write config file at {:?}: {}", path, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_test_mode_isolation() {
        std::env::set_var("SWAY_MANAGER_TEST_MODE", "1");
        let repo = SystemConfigRepository::new();
        let path = SystemConfigRepository::get_config_path();
        assert_eq!(path, PathBuf::from("/tmp/sway_manager_tests/test_config.json"));

        let config = AppConfig {
            wallpaper_folder: "/test/wallpapers".to_string(),
            screenshot_folder: "/test/screenshots".to_string(),
            theme: "dark".to_string(),
        };
        repo.save(&config).unwrap();
        let loaded = repo.load().unwrap();
        assert_eq!(loaded, config);

        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
}
