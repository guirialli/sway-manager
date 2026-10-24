//! `power status|toggle` — power-profiles-daemon wrapper.
//!
//! Mirrors `PowerProfilesRepository`
//! (`src/infrastructure/power/powerprofiles_repository.py:11-34`),
//! `TogglePowerProfileUseCase.execute` flag mapping (`-p`/`-b`/`-s`/rotate)
//! and `PowerProfileState` JSON + friendly names
//! (`src/domain/power/entities.py:50-63`).

use crate::process::{CommandRunner, notify};

pub const PROFILES: &[&str] = &["power-saver", "balanced", "performance"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerProfileState {
    pub active_profile: String,
}

impl PowerProfileState {
    pub fn icon(&self) -> &'static str {
        match self.active_profile.as_str() {
            "power-saver" => "\u{f06c}",
            "balanced" => "\u{f24e}",
            _ => "\u{f0e7}",
        }
    }

    pub fn to_waybar_json(&self) -> serde_json::Value {
        serde_json::json!({
            "text": self.icon(),
            "tooltip": format!("Power Profile: {}", self.active_profile),
        })
    }

    /// Python `json.dumps` bytes: `{"text", "tooltip"}` order.
    pub fn to_waybar_string(&self) -> String {
        crate::waybar::WaybarJson::new(vec![
            ("text", self.icon().to_string()),
            ("tooltip", format!("Power Profile: {}", self.active_profile)),
        ])
        .dumps_ascii()
    }
}

pub fn friendly_name(profile: &str) -> &str {
    match profile {
        "power-saver" => "economia de energia",
        "balanced" => "equilibrado",
        "performance" => "desempenho máximo",
        _ => profile,
    }
}

pub fn get_state(runner: &dyn CommandRunner) -> PowerProfileState {
    let output = runner.run("powerprofilesctl", &["get"]);
    let val = output.stdout.trim().to_string();
    if output.succeeded() && PROFILES.contains(&val.as_str()) {
        PowerProfileState { active_profile: val }
    } else {
        PowerProfileState {
            active_profile: "balanced".to_string(),
        }
    }
}

/// Mirrors the use-case rotation + `set_profile` notification/output.
pub fn execute(runner: &dyn CommandRunner, flag: Option<&str>) -> String {
    let current = get_state(runner).active_profile;
    let target = match flag {
        Some("-p") => "performance".to_string(),
        Some("-b") => "balanced".to_string(),
        Some("-s") => "power-saver".to_string(),
        _ => {
            let idx = PROFILES.iter().position(|item| *item == current).unwrap_or(0);
            PROFILES[(idx + 1) % PROFILES.len()].to_string()
        }
    };
    runner.run("powerprofilesctl", &["set", &target]);
    notify(runner, &format!("{} setado", friendly_name(&target)), "", "normal");
    format!("Power profile set to: {target}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    #[test]
    fn json_and_friendly_names_match_python() {
        let state = PowerProfileState {
            active_profile: "power-saver".to_string(),
        };
        assert_eq!(
            state.to_waybar_json(),
            serde_json::json!({"text": "\u{f06c}", "tooltip": "Power Profile: power-saver"})
        );
        assert_eq!(friendly_name("balanced"), "equilibrado");
        assert_eq!(friendly_name("performance"), "desempenho máximo");
    }

    #[test]
    fn rotation_and_flags() {
        let runner = FakeRunner::new()
            .with_success("powerprofilesctl", &["get"], "balanced\n")
            .with_success("powerprofilesctl", &["set", "performance"], "")
            .with_success("notify-send", &[], "");
        assert_eq!(
            execute(&runner, None),
            "Power profile set to: performance"
        );
        let runner = FakeRunner::new()
            .with_success("powerprofilesctl", &["get"], "balanced\n")
            .with_success("powerprofilesctl", &["set", "power-saver"], "")
            .with_success("notify-send", &[], "");
        assert_eq!(
            execute(&runner, Some("-s")),
            "Power profile set to: power-saver"
        );
    }

    #[test]
    fn unknown_profile_falls_back_to_balanced() {
        let runner = FakeRunner::new()
            .with_failure("powerprofilesctl", &["get"], 1);
        assert_eq!(get_state(&runner).active_profile, "balanced");
    }
}
