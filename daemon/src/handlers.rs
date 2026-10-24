//! Synchronous command handlers — exact stdout/stderr parity.
//!
//! Mirrors `CLIHandlers` methods that run inside the daemon
//! (`src/presentation/cli/handlers.py:91-164`) and the `handle_cli_command`
//! routing (`src/infrastructure/daemon/server/cli.py:36-62`), including
//! the `reload-cache` branch and the unknown-command stderr literal.
//!
//! Interactive/GUI commands (`menu`, `screenshot`, `clipboard`, `lock`,
//! `settings`, ...) are answered with the immediate empty ack by
//! [`route_request`](crate::server::route_request) and executed on the
//! deferred path — identical to `_process_request` +
//! `QTimer.singleShot(0, ...)` in Python. Portal always runs locally
//! in the CLI process (never via the socket); its chooser UI stays in
//! Python.

use crate::repos::{battery, idle, lock, menu, power, theme};

pub const UNKNOWN_COMMAND_SUFFIX: &str = "não reconhecido pelo SwayManager.\n";

/// Dependencies for the synchronous commands (repos + process runner).
pub trait SyncDeps: Send + Sync {
    fn runner(&self) -> &dyn crate::process::CommandRunner;
    fn battery_fs(&self) -> &dyn battery::BatteryFs;
    fn theme_fs(&self) -> &dyn theme::ThemeFs;
    fn theme_applier(&self) -> &dyn theme::ThemeApplier;
    fn lock_ctx(&self) -> &dyn lock::LockContext;
    fn home(&self) -> &str;
    /// Deferred `menu` execution (interactive path; output discarded).
    fn run_menu(&self, filter: Option<&str>);
    /// Deferred `clipboard` execution (interactive path; output discarded).
    fn run_clipboard(&self, action: Option<&str>);
}

impl<D: SyncDeps> SyncDeps for std::sync::Arc<D> {
    fn runner(&self) -> &dyn crate::process::CommandRunner {
        (**self).runner()
    }
    fn battery_fs(&self) -> &dyn battery::BatteryFs {
        (**self).battery_fs()
    }
    fn theme_fs(&self) -> &dyn theme::ThemeFs {
        (**self).theme_fs()
    }
    fn theme_applier(&self) -> &dyn theme::ThemeApplier {
        (**self).theme_applier()
    }
    fn lock_ctx(&self) -> &dyn lock::LockContext {
        (**self).lock_ctx()
    }
    fn home(&self) -> &str {
        (**self).home()
    }
    fn run_menu(&self, filter: Option<&str>) {
        (**self).run_menu(filter);
    }
    fn run_clipboard(&self, action: Option<&str>) {
        (**self).run_clipboard(action);
    }
}

fn get_arg<'a>(args: &'a [String], index: usize, default: &'a str) -> &'a str {
    args.get(index).map(String::as_str).unwrap_or(default)
}

/// Lowercased `args[2]` with `default` — mirrors
/// `ArrayUtils.getSafe(args, 2, default).lower()`.
pub fn action_arg(args: &[String], default: &str) -> String {
    get_arg(args, 2, default).to_lowercase()
}

/// Run one synchronous command; returns captured `(stdout, stderr)`.
/// Errors are returned as values (never panics) so the server can ship
/// them back with `exit_code` intact — mirroring `_safe_dispatch_command`
/// swallowing after `CliServerHandler` logs.
pub fn handle_sync(deps: &dyn SyncDeps, args: &[String]) -> (String, String) {
    let cmd = get_arg(args, 1, "").to_lowercase();
    match cmd.as_str() {
        "battery" => {
            if action_arg(args, "toggle") == "status" {
                let state = battery::get_state(deps.runner(), deps.battery_fs());
                (format!("{}\n", state.to_waybar_string()), String::new())
            } else {
                let (_, msg) = battery::toggle(deps.runner(), deps.battery_fs());
                (format!("{msg}\n"), String::new())
            }
        }
        "idle" => {
            if action_arg(args, "toggle") == "status" {
                let state = idle::get_state(deps.runner());
                (format!("{}\n", state.to_waybar_string()), String::new())
            } else {
                let flag = args.get(3).map(String::as_str);
                let msg = idle::toggle(deps.runner(), deps.home(), flag);
                (format!("{msg}\n"), String::new())
            }
        }
        "theme" => {
            if action_arg(args, "toggle") == "status" {
                let state = theme::get_state(deps.theme_fs(), deps.home());
                (format!("{}\n", state.to_waybar_string()), String::new())
            } else {
                let msg = theme::toggle(deps.runner(), deps.theme_fs(), deps.theme_applier(), deps.home());
                (format!("{msg}\n"), String::new())
            }
        }
        "power" => {
            if action_arg(args, "toggle") == "status" {
                let state = power::get_state(deps.runner());
                (format!("{}\n", state.to_waybar_string()), String::new())
            } else {
                let flag = args.get(3).map(String::as_str);
                let msg = power::execute(deps.runner(), flag);
                (format!("{msg}\n"), String::new())
            }
        }
        "lock" => {
            lock::lock_screen(deps.lock_ctx(), deps.runner());
            (String::new(), String::new())
        }
        "menu" => {
            let filter = args.get(2).map(String::as_str);
            deps.run_menu(filter);
            (String::new(), String::new())
        }
        "clipboard" | "clip" => {
            let action = args.get(2).map(String::as_str);
            deps.run_clipboard(action);
            (String::new(), String::new())
        }
        "reload-cache" | "refresh-cache" => {
            let items = menu::SYSTEM_ACTIONS.len();
            (format!("{}\n", menu::reload_cache_message(items)), String::new())
        }
        other => (String::new(), format!("Comando '{other}' {UNKNOWN_COMMAND_SUFFIX}")),
    }
}
