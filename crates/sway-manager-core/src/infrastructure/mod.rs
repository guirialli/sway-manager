pub mod battery;
pub mod config;
pub mod display;
pub mod idle;
pub mod power_profile;
pub mod wallpaper;

pub use battery::SystemBatteryRepository;
pub use config::SystemConfigRepository;
pub use display::SystemDisplayRepository;
pub use idle::SystemIdleRepository;
pub use power_profile::SystemPowerRepository;
pub use wallpaper::SystemWallpaperRepository;
