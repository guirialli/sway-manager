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

/// Lowercase-normalized command name from `args` (`args[1]`), if present.
pub fn command_name(args: &[String]) -> Option<String> {
    args.get(1).map(|cmd| cmd.to_lowercase())
}

/// `true` when the command must be acknowledged immediately and run deferred.
///
/// Mirrors `CliServerHandler.is_interactive`, with argv-aware exceptions:
/// `wallpaper get [pasta]` prints the resolved folder on stdout, so it
/// runs synchronously (its output ships back over the socket). A bare
/// `wallpaper` still opens the picker (interactive + spawn).
pub fn is_interactive(cmd: &str) -> bool {
    INTERACTIVE_COMMANDS.contains(&cmd.to_lowercase().as_str())
}

/// Argv-aware variant: sync subcommands of interactive commands.
pub fn is_interactive_args(cmd: &str, args: &[String]) -> bool {
    if !is_interactive(cmd) {
        return false;
    }
    if cmd.eq_ignore_ascii_case("wallpaper")
        && args.get(2).is_some_and(|sub| sub.eq_ignore_ascii_case("get"))
    {
        return false;
    }
    true
}
