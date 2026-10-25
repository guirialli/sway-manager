//! Settings domain: persistent user configuration surfaces.
//!
//! Wallpaper, GTK/Qt appearance, monitors, LightDM greeter, theme toggle
//! and the JSON `app_config` store — everything the ConfigCenter,
//! wallpaper picker and monitor windows read or mutate.

pub mod app_config;
pub mod appearance;
pub mod display;
pub mod lightdm;
pub mod theme;
pub mod wallpaper;
pub mod wallpaper_thumbs;
