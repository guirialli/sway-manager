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

use crate::repos::{audio, battery, idle, lock, menu, portal, power, screenshot, theme, wallpaper};

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
    /// Spawn the isolated GUI subprocess (`SwayManagerGUI ...`).
    fn spawn_gui(&self, _args: &[String]) {}
    /// Wallpaper folder probe (`is_dir` seam for tests).
    fn wallpaper_dir_exists(&self, _path: &str) -> bool {
        false
    }
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
    fn spawn_gui(&self, args: &[String]) {
        (**self).spawn_gui(args);
    }
    fn wallpaper_dir_exists(&self, path: &str) -> bool {
        (**self).wallpaper_dir_exists(path)
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

/// OSD `(kind, action)`: `osd volume up` → `("volume", Some("up"))`,
/// `brilho up` → `("brightness", Some("up"))`, `osd brilho` → `("brightness", None)`.
fn osd_kind_action<'a>(cmd: &str, args: &'a [String]) -> (Option<&'a str>, Option<&'a str>) {
    if cmd == "osd" {
        let kind = match args.get(2).map(String::as_str) {
            Some("volume") => Some("volume"),
            Some("brilho") | Some("brightness") => Some("brightness"),
            _ => None,
        };
        return (kind, args.get(3).map(String::as_str));
    }
    let action = args.get(2).map(String::as_str);
    match action {
        // `brilho` bare/popup abre o popup (sem ação executável).
        None | Some("popup") => (None, None),
        _ => (Some("brightness"), action),
    }
}

/// Executa a ação OSD no daemon (paridade `VolumeOSD`/`BrightnessOSD`).
fn osd_apply_action(runner: &dyn crate::process::CommandRunner, kind: &str, action: Option<&str>) {
    let Some(action) = action else { return };
    if kind == "volume" {
        audio::adjust_volume(runner, action);
    } else if matches!(action, "up" | "down") {
        audio::change_brightness(runner, action, 5);
    }
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
        "screenshot" => {
            // Deferred like the other GUI commands: spawn `SwayManagerGUI
            // screenshot [mode]` and ack immediately. Mirrors
            // `dispatch_cli_command` → `handle_screenshot` routed to the
            // GUI process (the Qt freeze overlay stays in Python); the
            // headless `grim` fallback lives in `repos::screenshot`
            // for standalone/headless use.
            let _mode = screenshot::parse_mode(args.get(2).map(String::as_str));
            deps.spawn_gui(args);
            (String::new(), String::new())
        }
        "osd" | "brilho" | "brightness" => {
            // Mirrors `handle_osd`/`handle_brightness` routing: the OSD
            // windows stay in the GUI process. `osd` without a type prints
            // `OSD não informado!` on stdout like Python.
            if cmd.as_str() == "osd" && args.get(2).is_none() {
                return ("OSD não informado!\n".to_string(), String::new());
            }
            // Latência: executa a ação AQUI (~10ms via socket) e spawna a
            // GUI sem ela (só exibe o estado). Antes, a ação rodava no
            // `invoke` do frontend — depois de ~330ms de boot Tauri/WebKit.
            let (kind, action) = osd_kind_action(&cmd, args);
            match (kind, action) {
                (Some(kind), Some(_)) => {
                    osd_apply_action(deps.runner(), kind, action);
                    deps.spawn_gui(&[
                        args[0].clone(),
                        "osd".to_string(),
                        kind.to_string(),
                    ]);
                }
                _ => deps.spawn_gui(args),
            }
            (String::new(), String::new())
        }
        "wallpaper" => {
            // Mirrors `handle_wallpaper`: explicit folder → GUI picker;
            // bare `wallpaper` resolves the folder then opens the picker.
            // `wallpaper get [pasta]` prints the resolved folder on
            // stdout (sync: `is_interactive_args` excludes it, so this
            // output ships back over the socket instead of spawning).
            if args.get(2).is_some_and(|arg| arg.eq_ignore_ascii_case("get")) {
                let folder = match args.get(3) {
                    Some(_) => args[3..].join(" "),
                    None => wallpaper::wallpaper_folder(deps.home(), &|path| {
                        deps.wallpaper_dir_exists(path)
                    }),
                };
                if !folder.is_empty() {
                    return (format!("{folder}\n"), String::new());
                }
            }
            deps.spawn_gui(args);
            (String::new(), String::new())
        }
        "monitor" | "settings" | "config" | "config-center" => {
            // Mirrors `handle_monitor`/`handle_settings`: GUI windows.
            // Spawn detached, exit 0 — ack immediately via the socket.
            deps.spawn_gui(args);
            (String::new(), String::new())
        }
        "volume" => {
            // `MixerAudioRepository` volume path (sibling of the OSD
            // popups): forward to the GUI command with argv intact.
            deps.spawn_gui(args);
            (String::new(), String::new())
        }
        "portal" => match args.get(2).map(|sub| sub.to_lowercase()).as_deref() {
            // Mirrors `handle_portal`: `status` diagnostics print the
            // report; `test` runs the chooser in test mode; `install`
            // writes the portal configs (used by `install.sh`); the bare
            // chooser spawns the GUI (attached only via `local_invoke`).
            Some("status") => {
                let report = portal::diagnose_with(
                    deps.runner(),
                    &portal::DiagnosticsEnv::live(),
                );
                (portal::status_text(&report), String::new())
            }
            Some("test") => (String::from("[TESTE] Cancelado pelo usuário\n"), String::new()),
            Some("install") => match portal::install_config(deps.home(), deps.runner()) {
                Ok(log) => (log.join("\n") + "\n", String::new()),
                Err(err) => (String::new(), format!("{err}\n")),
            },
            _ => {
                deps.spawn_gui(args);
                (String::new(), String::new())
            }
        },
        "reload-cache" | "refresh-cache" => {
            let items = menu::SYSTEM_ACTIONS.len();
            (format!("{}\n", menu::reload_cache_message(items)), String::new())
        }
        other => (String::new(), format!("Comando '{other}' {UNKNOWN_COMMAND_SUFFIX}")),
    }
}
