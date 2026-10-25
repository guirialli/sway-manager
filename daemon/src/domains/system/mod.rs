//! System domain: machine-level operations.
//!
//! Power/battery/idle/lock, screenshots, the XDG portal chooser, the
//! polkit authentication agent and shared icon resolution.

pub mod battery;
pub mod icons;
pub mod idle;
pub mod lock;
pub mod polkit;
pub mod portal;
pub mod power;
pub mod screenshot;
