//! `idle status|toggle` — swayidle supervisor state.
//!
//! Mirrors `SwayIdleRepository`
//! (`src/infrastructure/power/swayidle_repository.py:13-66`) and
//! `IdleState.to_waybar_json` (`src/domain/power/entities.py:32-48`).

use crate::process::{CommandRunner, notify};

pub const STATE_FILE_SUFFIX: &str = ".cache/swayidle_enabled";
pub const TIMEOUT_SLEEP: u32 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdleState {
    pub is_running: bool,
}

impl IdleState {
    pub fn to_waybar_json(&self) -> serde_json::Value {
        if !self.is_running {
            serde_json::json!({
                "text": "\u{f06e}",
                "tooltip": "Suspensão desativada",
                "class": "idle-off",
            })
        } else {
            serde_json::json!({
                "text": "\u{f070}",
                "tooltip": "Suspensão ativada",
                "class": "idle-on",
            })
        }
    }

    /// Python `json.dumps` bytes: `{"text", "tooltip", "class"}` order.
    pub fn to_waybar_string(&self) -> String {
        let icon = if !self.is_running { "\u{f06e}" } else { "\u{f070}" };
        let (tooltip, class) = if !self.is_running {
            ("Suspensão desativada", "idle-off")
        } else {
            ("Suspensão ativada", "idle-on")
        };
        crate::waybar::WaybarJson::new(vec![
            ("text", icon.to_string()),
            ("tooltip", tooltip.to_string()),
            ("class", class.to_string()),
        ])
        .dumps_ascii()
    }
}

pub fn state_file(home: &str) -> String {
    format!("{home}/{STATE_FILE_SUFFIX}")
}

pub fn get_state(runner: &dyn CommandRunner) -> IdleState {
    let running = runner.run("pgrep", &["-x", "swayidle"]).succeeded();
    IdleState { is_running: running }
}

/// Mirrors `SwayIdleRepository.toggle`, flags `-n`/`-s`/`-r`/default.
pub fn toggle(runner: &dyn CommandRunner, home: &str, flag: Option<&str>) -> String {
    let running = runner.run("pgrep", &["-x", "swayidle"]).succeeded();
    if flag == Some("-n") {
        runner.run("systemctl", &["suspend"]);
    }
    let state = if flag == Some("-n") || flag == Some("-s") {
        "off"
    } else if flag == Some("-r") || flag.is_none() {
        if running { "off" } else { "on" }
    } else if running {
        "off"
    } else {
        "on"
    };
    // `killall -q swayidle` unconditionally (Python line 42).
    runner.run("killall", &["-q", "swayidle"]);

    let state_path = state_file(home);
    if let Some(parent) = std::path::Path::new(&state_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let msg = if state == "off" {
        let _ = std::fs::write(&state_path, "off");
        "Suspensão desativada!"
    } else {
        runner.spawn_detached(
            "swayidle",
            &[
                "-w",
                "timeout",
                &TIMEOUT_SLEEP.to_string(),
                "swaymsg \"output * dpms off\"",
                "resume",
                "swaymsg \"output * dpms on\"",
            ],
        );
        let _ = std::fs::write(&state_path, "on");
        "Suspensão ativada!"
    };

    if flag != Some("-s") {
        notify(runner, msg, msg, "normal");
    }
    msg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    fn home() -> String {
        let dir = tempfile::tempdir().unwrap().keep();
        dir.to_string_lossy().into_owned()
    }

    #[test]
    fn idle_json_literals() {
        assert_eq!(
            IdleState { is_running: false }.to_waybar_json(),
            serde_json::json!({"text": "\u{f06e}", "tooltip": "Suspensão desativada", "class": "idle-off"})
        );
        assert_eq!(
            IdleState { is_running: true }.to_waybar_json()["class"],
            serde_json::json!("idle-on")
        );
    }

    #[test]
    fn toggle_on_spawns_swayidle_and_writes_state() {
        let home = home();
        let runner = FakeRunner::new()
            .with_failure("pgrep", &["-x", "swayidle"], 1)
            .with_success("killall", &["-q", "swayidle"], "")
            .with_success("notify-send", &[], "");
        let msg = toggle(&runner, &home, None);
        assert_eq!(msg, "Suspensão ativada!");
        assert_eq!(
            std::fs::read_to_string(state_file(&home)).unwrap(),
            "on"
        );
        let spawned = runner.spawned();
        assert_eq!(spawned.len(), 1);
        assert_eq!(spawned[0][0], "swayidle");
        assert!(spawned[0].contains(&"300".to_string()));
    }

    #[test]
    fn toggle_off_with_silent_flag_skips_notification() {
        let home = home();
        let runner = FakeRunner::new()
            .with_success("pgrep", &["-x", "swayidle"], "")
            .with_success("killall", &["-q", "swayidle"], "");
        let msg = toggle(&runner, &home, Some("-s"));
        assert_eq!(msg, "Suspensão desativada!");
        assert!(runner.spawned().is_empty());
    }
}
