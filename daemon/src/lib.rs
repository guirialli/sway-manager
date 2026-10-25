//! Rust port of the SwayManager daemon core.
//!
//! Mirrors `src/infrastructure/daemon/` from the Python implementation:
//! Unix-socket path resolution, line-delimited JSON protocol, IPC client,
//! single-instance probe, command classification (interactive vs GUI) and
//! crash-safe dispatch. The GUI remains PySide6 and is spawned as an
//! isolated subprocess; the async server event loop is the next step.

pub mod binary;
pub mod client;
pub mod commands;
pub mod dispatch;
pub mod domains;
pub mod handlers;
pub mod logging;
pub mod process;
pub mod protocol;
pub mod reaper;
pub mod runtime;
pub mod server;
pub mod socket;
pub mod waybar;
