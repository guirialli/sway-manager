//! OSD domain: on-screen indicators (volume + brightness).
//!
//! [`audio`](self::audio) owns `wpctl` volume state and `brightnessctl`
//! percentage behind the [`CommandRunner`](crate::process::CommandRunner) seam.

pub mod audio;
