//! SwayManager GUI — backend Tauri v2 (fase 1: comandos OSD).
//!
//! Camada fina sobre `sway_manager_daemon::repos`: cada janela do frontend
//! invoca um comando e recebe JSON pronto para renderizar. Nenhum acesso a
//! `/sys`, `swaymsg` ou binários fora dos repos do daemon.

pub mod commands;
pub mod config_center;
pub mod mon_wall;
pub mod polkit_dialog;
pub mod portal_freeze;
pub use polkit_dialog::{GuiEvent, PolkitDialogState, PolkitLaunch, cancel_line, gui_event_from_json, identity_display, parse_polkit_args, submit_line};
pub use commands::{brightness_osd, volume_osd, BrightnessAction, VolumeAction, VolumeOsd};
pub use config_center::{appearance_apply, appearance_page, battery_toggle, current_wallpaper, idle_page, idle_toggle, lightdm_page, lightdm_save, monitors_apply, monitors_page, monitors_set_screenshot_folder, power_apply, power_page, select_page, theme_toggle, wallpaper_images, wallpaper_page, wallpaper_page_in, wallpaper_set_folder, WallpaperImage, WallpaperPage};
pub use mon_wall::{DisplayMode, MonitorLayouts, WallpaperEntry, monitor_apply, monitor_layouts, monitor_reload, wallpaper_apply, wallpaper_apply_to, wallpaper_default_folder, wallpaper_list, wallpaper_list_in};
pub use portal_freeze::{BlitOp, Composition, FrozenScreenDto, PortalSelection, PortalSourceDto, PortalSources, Rect, SelectionState, freeze_cancel, freeze_capture, freeze_compose, freeze_image_data, freeze_screens, parse_dmenu_sources, portal_cancel, portal_select, portal_sources, screenshot_compose_save, screenshot_full, screenshot_save_png, screenshot_timestamp, screenshot_window};
