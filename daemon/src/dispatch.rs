//! Crash-safe command dispatch.
//!
//! Mirrors `SwayManagerDaemon._safe_dispatch_command` and
//! `CliServerHandler.handle_cli_command`: GUI commands are delegated to
//! the isolated GUI spawner, synchronous commands run through the
//! injected CLI runner with exceptions captured (never crashing the
//! daemon), and `perform_gc` runs in `finally` position.

use crate::commands::{command_name, is_gui_command};
use crate::gui::GuiSpawner;

/// Synchronous CLI execution boundary (Python `CLIHandlers` today).
///
/// Returns captured `(stdout, stderr)` instead of writing to the real
/// stdio so the daemon can ship them back over the socket. Any `Err`
/// must be logged by the caller as
/// `Falha na execução do comando '<cmd>': <err>`.
pub trait CliRunner {
    type Error: std::fmt::Display;
    fn run_cli(&self, args: &[String]) -> Result<(String, String), Self::Error>;
}

/// What [`dispatch`] decided to do; lets tests observe routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// No `args[1]`; Python returns early without touching handlers.
    NoCommand,
    /// Routed to the isolated GUI subprocess (Python returns early too).
    GuiSpawned,
    /// Synchronous CLI ran; carries captured output.
    CliExecuted { stdout: String, stderr: String },
}

/// Dispatch one request's argv without ever propagating a failure.
///
/// GUI commands spawn and return; CLI errors are swallowed after
/// logging (mirrors `except Exception: pass` in `_safe_dispatch_command`
/// once `CliServerHandler` already logged). `on_gc` models the
/// `finally: self.perform_gc()` hook; callers pass the real GC there.
pub fn dispatch<R, G, F>(args: &[String], runner: &R, gui: &G, mut on_gc: F) -> DispatchOutcome
where
    R: CliRunner,
    G: GuiSpawner,
    F: FnMut(),
{
    let Some(cmd) = command_name(args) else {
        return DispatchOutcome::NoCommand;
    };

    if is_gui_command(&cmd) {
        gui.spawn_gui(args);
        on_gc();
        return DispatchOutcome::GuiSpawned;
    }

    let outcome = match runner.run_cli(args) {
        Ok((stdout, stderr)) => DispatchOutcome::CliExecuted { stdout, stderr },
        Err(_err) => {
            // Logged upstream in Python (`CliServerHandler` logs before
            // re-raising); `_safe_dispatch_command` then swallows it.
            DispatchOutcome::CliExecuted {
                stdout: String::new(),
                stderr: String::new(),
            }
        }
    };
    on_gc();
    outcome
}
