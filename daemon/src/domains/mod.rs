//! System-domain ports used by the daemon dispatch path.
//!
//! Each submodule mirrors one legacy repository 1:1 — same file paths,
//! same external commands, same stdout/stderr literals — with the
//! [`CommandRunner`](crate::process::CommandRunner) seam standing in for
//! `subprocess` and the filesystem probes abstracted for tests.
//!
//! Layout: [`osd`] (volume/brightness indicators), [`menus`] (app +
//! clipboard launchers), [`settings`] (wallpaper/appearance/monitors/
//! LightDM/theme/app_config), [`system`] (power/battery/idle/lock,
//! screenshots, portal, polkit, icons).

pub mod menus;
pub mod osd;
pub mod settings;
pub mod system;
