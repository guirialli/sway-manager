//! Repository ports used by the daemon dispatch path.
//!
//! Each submodule mirrors one Python repository 1:1 — same file paths,
//! same external commands, same stdout/stderr literals — with the
//! [`CommandRunner`](crate::process::CommandRunner) seam standing in for
//! `subprocess` and the filesystem probes abstracted for tests.

pub mod audio;
pub mod battery;
pub mod clipboard;
pub mod clipboard_exec;
pub mod desktop_index;
pub mod display;
pub mod icons;
pub mod idle;
pub mod lock;
pub mod menu;
pub mod menu_exec;
pub mod power;
pub mod screenshot;
pub mod theme;
pub mod wallpaper;
