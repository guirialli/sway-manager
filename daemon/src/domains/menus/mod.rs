//! Menus domain: wofi launchers (app menu + clipboard history).
//!
//! [`menu`](self::menu) + [`menu_exec`](self::menu_exec) +
//! [`desktop_index`](self::desktop_index) own the `.desktop` index and app
//! launcher; [`clipboard`](self::clipboard) +
//! [`clipboard_exec`](self::clipboard_exec) own the `cliphist` history,
//! image thumbnails and typed `wl-copy`.

pub mod clipboard;
pub mod clipboard_exec;
pub mod desktop_index;
pub mod menu;
pub mod menu_exec;
