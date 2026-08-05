slint::include_modules!();

pub fn show_osd(title: &str, icon: &str, value: i32) -> Result<(), String> {
    let window = OsdWindow::new().map_err(|e| format!("Failed to create OSD window: {}", e))?;
    window.set_osd_title(title.into());
    window.set_osd_icon(icon.into());
    window.set_value(value);

    let window_handle = window.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(1200), move || {
        if let Some(w) = window_handle.upgrade() {
            let _ = w.hide();
        }
    });

    window.run().map_err(|e| format!("Error running OSD window: {}", e))
}

pub fn show_wallpaper_picker() -> Result<Option<String>, String> {
    let window = WallpaperPickerWindow::new().map_err(|e| format!("Failed to create WallpaperPicker: {}", e))?;
    window.run().map_err(|e| format!("Error running WallpaperPicker: {}", e))?;
    Ok(None)
}

pub fn show_config_center() -> Result<(), String> {
    let window = ConfigCenterWindow::new().map_err(|e| format!("Failed to create ConfigCenter: {}", e))?;
    window.run().map_err(|e| format!("Error running ConfigCenter: {}", e))
}

pub fn show_freeze_overlay() -> Result<(), String> {
    let window = FreezeSelectionOverlay::new().map_err(|e| format!("Failed to create FreezeOverlay: {}", e))?;
    window.run().map_err(|e| format!("Error running FreezeOverlay: {}", e))
}
