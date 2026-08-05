use crate::domain::traits::WallpaperRepository;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct SystemWallpaperRepository;

impl SystemWallpaperRepository {
    pub fn new() -> Self {
        Self
    }

    fn get_sway_wallpaper_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("sway");
        path.push("config.d");
        path.push("42-wallpaper");
        path
    }
}

impl Default for SystemWallpaperRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperRepository for SystemWallpaperRepository {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), String> {
        if !image_path.exists() {
            return Err(format!("Image file at {:?} does not exist", image_path));
        }

        let abs_path = image_path
            .canonicalize()
            .map_err(|e| format!("Failed to resolve path {:?}: {}", image_path, e))?;

        let config_path = Self::get_sway_wallpaper_config_path();
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let config_content = format!("output * bg \"{}\" fill\n", abs_path.display());
        fs::write(&config_path, config_content)
            .map_err(|e| format!("Failed to write wallpaper config at {:?}: {}", config_path, e))?;

        // Apply wallpaper live using swaymsg or oguri/swaybg
        let _ = Command::new("swaymsg")
            .arg("output")
            .arg("*")
            .arg("bg")
            .arg(abs_path.to_string_lossy().as_ref())
            .arg("fill")
            .status();

        Ok(())
    }
}
