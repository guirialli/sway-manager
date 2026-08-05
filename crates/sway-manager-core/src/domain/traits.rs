use crate::domain::entities::*;
use std::path::Path;

pub trait ConfigRepository: Send + Sync {
    fn load(&self) -> Result<AppConfig, String>;
    fn save(&self, config: &AppConfig) -> Result<(), String>;
}

pub trait BatteryRepository: Send + Sync {
    fn get_state(&self) -> Result<BatteryState, String>;
    fn toggle_conservation_mode(&self) -> Result<bool, String>;
}

pub trait IdleRepository: Send + Sync {
    fn get_state(&self) -> Result<IdleState, String>;
    fn toggle_idle(&self) -> Result<bool, String>;
}

pub trait PowerRepository: Send + Sync {
    fn get_profile(&self) -> Result<PowerProfile, String>;
    fn set_profile(&self, profile: PowerProfile) -> Result<(), String>;
}

pub trait DisplayRepository: Send + Sync {
    fn apply_layout(&self, layout: DisplayLayout) -> Result<(), String>;
}

pub trait WallpaperRepository: Send + Sync {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), String>;
}
