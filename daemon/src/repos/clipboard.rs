//! Clipboard daemon guarantee + action literals.
//!
//! The interactive `clipboard` flow renders Qt/Wofi menus and stays in
//! Python. The daemon's contract — the part this port owns — is
//! `ensure_daemon_running` (`cliphist_repository.py:40-62`): warn when
//! tools are missing, reuse a running `cliphist store`, else spawn
//! `wl-paste --watch cliphist store` detached. Use-case literals that
//! surface over the socket (`manage_clipboard_use_case.py`) live here.

use crate::process::CommandRunner;

pub const WATCH_ARGV: &[&str] = &["wl-paste", "--watch", "cliphist", "store"];

pub const EMPTY_HISTORY_MSG: &str = "Nenhum item encontrado no histórico do clipboard.";
pub const EMPTY_PIN_MSG: &str = "Nenhum item disponível para fixar/desfixar.";
pub const MISSING_TOOLS_MSG: &str =
    "Aviso: 'cliphist' ou 'wl-paste' não estão instalados no sistema.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsureOutcome {
    AlreadyRunning,
    Spawned,
    MissingTools,
    Failed,
}

/// Mirrors `CliphistRepository.ensure_daemon_running` incl. prints.
pub fn ensure_daemon_running(runner: &dyn CommandRunner) -> EnsureOutcome {
    if runner.which("cliphist").is_none() || runner.which("wl-paste").is_none() {
        println!("{MISSING_TOOLS_MSG}");
        return EnsureOutcome::MissingTools;
    }
    let probe = runner.run("pgrep", &["-f", "cliphist store"]);
    if probe.succeeded() && !probe.stdout.trim().is_empty() {
        return EnsureOutcome::AlreadyRunning;
    }
    runner.spawn_detached(WATCH_ARGV[0], &WATCH_ARGV[1..]);
    EnsureOutcome::Spawned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    #[test]
    fn warns_when_tools_missing() {
        let runner = FakeRunner::new();
        assert_eq!(ensure_daemon_running(&runner), EnsureOutcome::MissingTools);
    }

    #[test]
    fn reuses_running_store() {
        let runner = FakeRunner::new()
            .with_which("cliphist", Some("/usr/bin/cliphist"))
            .with_which("wl-paste", Some("/usr/bin/wl-paste"))
            .with_success("pgrep", &["-f", "cliphist store"], "1234\n");
        assert_eq!(
            ensure_daemon_running(&runner),
            EnsureOutcome::AlreadyRunning
        );
        assert!(runner.spawned().is_empty());
    }

    #[test]
    fn spawns_watch_when_absent() {
        let runner = FakeRunner::new()
            .with_which("cliphist", Some("/usr/bin/cliphist"))
            .with_which("wl-paste", Some("/usr/bin/wl-paste"))
            .with_failure("pgrep", &["-f", "cliphist store"], 1);
        assert_eq!(ensure_daemon_running(&runner), EnsureOutcome::Spawned);
        assert_eq!(
            runner.spawned(),
            vec![WATCH_ARGV.iter().map(|arg| arg.to_string()).collect::<Vec<_>>()]
        );
    }
}
