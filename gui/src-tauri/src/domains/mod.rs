//! GUI domains mirroring `sway_manager_daemon::domains`:
//!
//! [`osd`] (indicators), [`menus`] (launchers — reserved), [`settings`]
//! (ConfigCenter/monitors/wallpaper), [`system`] (portal/freeze/polkit).

pub mod menus;
pub mod osd;
pub mod settings;
pub mod system;
