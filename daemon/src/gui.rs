//! Isolated GUI subprocess spawning.
//!
//! Mirrors `GuiServerHandler.handle_gui_command` (`server/gui.py`):
//! resolve `SwayManagerGUI` next to the current executable and spawn it
//! with the original subcommand (`cmd_args[1:]`), detached. Failures are
//! reported on stderr and logged, never raised.

use crate::binary::binary_command;

/// Spawning boundary so tests can observe without forking.
pub trait GuiSpawner {
    fn spawn_gui(&self, args: &[String]) -> Vec<String>;
}

/// Build the detached GUI command: `[SwayManagerGUI, ...args[1:]]`.
pub fn gui_command(args: &[String]) -> Vec<String> {
    let mut cmd = binary_command("SwayManagerGUI");
    if args.len() > 1 {
        cmd.extend(args[1..].iter().cloned());
    }
    cmd
}

fn log_spawn_error(message: &str) {
    eprintln!("{message}");
}

/// Default spawner used by the server (detached `Popen` equivalent).
pub struct SubprocessSpawner;

impl GuiSpawner for SubprocessSpawner {
    fn spawn_gui(&self, args: &[String]) -> Vec<String> {
        let cmd = gui_command(args);
        if let Some((program, rest)) = cmd.split_first() {
            match std::process::Command::new(program)
                .args(rest)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => {}
                Err(err) => log_spawn_error(&format!(
                    "Erro ao disparar processo GUI isolado: {err}"
                )),
            }
        }
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gui_command_appends_subcommand() {
        let cmd = gui_command(&["SwayManager".to_string(), "settings".to_string()]);
        assert!(cmd[0].contains("SwayManagerGUI"));
        assert_eq!(&cmd[1..], &["settings".to_string()]);
    }

    #[test]
    fn gui_command_without_subcommand_is_bare_binary() {
        let cmd = gui_command(&["SwayManager".to_string()]);
        assert_eq!(cmd.len(), 1);
        assert!(cmd[0].contains("SwayManagerGUI"));
    }

    #[test]
    fn spawner_never_panics_on_missing_binary() {
        // `DefinitelyNotInstalled` fallback: spawn fails, error logged,
        // command returned — mirrors `except Exception` in Python.
        let spawner = SubprocessSpawner;
        let cmd = spawner.spawn_gui(&["SwayManager".to_string(), "settings".to_string()]);
        assert!(cmd.iter().any(|arg| arg.contains("SwayManagerGUI")));
    }
}
