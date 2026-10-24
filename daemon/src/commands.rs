//! Command classification shared by the CLI and GUI dispatch paths.
//!
//! Mirrors `INTERACTIVE_COMMANDS` in
//! `src/infrastructure/daemon/server/cli.py` and `GUI_COMMANDS` in
//! `src/infrastructure/daemon/server/gui.py`. Matching is
//! case-insensitive; callers lowercase before lookup just like Python.

/// Commands answered immediately (<1ms) while execution is deferred.
///
/// Matches `CliServerHandler.is_interactive` (`server/cli.py`).
pub const INTERACTIVE_COMMANDS: &[&str] = &[
    "menu",
    "screenshot",
    "clipboard",
    "clip",
    "settings",
    "config",
    "config-center",
    "monitor",
    "wallpaper",
    "osd",
    "brilho",
    "brightness",
    "lock",
];

/// Commands rendered in an isolated `SwayManagerGUI` subprocess.
///
/// Matches `GuiServerHandler.is_gui_command` (`server/gui.py`).
pub const GUI_COMMANDS: &[&str] = &[
    "settings",
    "config",
    "config-center",
    "monitor",
    "wallpaper",
    "osd",
    "brilho",
    "brightness",
    "screenshot",
];

/// Lowercase-normalized command name from `args` (`args[1]`), if present.
pub fn command_name(args: &[String]) -> Option<String> {
    args.get(1).map(|cmd| cmd.to_lowercase())
}

/// `true` when the command must be acknowledged immediately and run deferred.
///
/// Mirrors `CliServerHandler.is_interactive`.
pub fn is_interactive(cmd: &str) -> bool {
    INTERACTIVE_COMMANDS.contains(&cmd.to_lowercase().as_str())
}

/// `true` when the command must run in an isolated GUI subprocess.
///
/// Mirrors `GuiServerHandler.is_gui_command` and takes precedence over
/// synchronous dispatch in `_safe_dispatch_command`.
pub fn is_gui_command(cmd: &str) -> bool {
    GUI_COMMANDS.contains(&cmd.to_lowercase().as_str())
}
