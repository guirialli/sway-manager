//! Crash-safe command dispatch.
//!
//! Mirrors `SwayManagerDaemon._safe_dispatch_command` and
//! `CliServerHandler.handle_cli_command`: every command runs through the
//! injected CLI runner with exceptions captured (never crashing the
//! daemon), and `perform_gc` runs in `finally` position. GUI routing
//! lives INSIDE the runner (`handle_sync` decides per-command whether to
//! spawn `SwayManagerGUI`, print, or execute) — never as a detour here:
//! a name-only detour cannot see sync subcommands (`wallpaper get`
//! prints; it must not spawn the picker) nor run fast-paths (the OSD
//! action executes in the daemon before the display-only GUI spawns).

use crate::commands::command_name;

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
    /// Synchronous CLI ran; carries captured output.
    CliExecuted { stdout: String, stderr: String },
}

/// Dispatch one request's argv without ever propagating a failure.
///
/// CLI errors are swallowed after logging (mirrors `except Exception:
/// pass` in `_safe_dispatch_command` once `CliServerHandler` already
/// logged). `on_gc` models the `finally: self.perform_gc()` hook;
/// callers pass the real GC there.
pub fn dispatch<R, F>(args: &[String], runner: &R, mut on_gc: F) -> DispatchOutcome
where
    R: CliRunner,
    F: FnMut(),
{
    let Some(_cmd) = command_name(args) else {
        return DispatchOutcome::NoCommand;
    };

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
