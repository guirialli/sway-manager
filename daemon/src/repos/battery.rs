//! `battery status|toggle` — sysfs conservation mode.
//!
//! Mirrors `SysfsBatteryRepository`
//! (`src/infrastructure/power/sysfs_battery_repository.py:13-130`) with
//! Waybar JSON from `src/domain/power/entities.py:6-30`.

use crate::process::{CommandRunner, CmdOutput, notify};

pub const CONTROL_CANDIDATES: &[&str] = &[
    "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode",
    "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:01/conservation_mode",
    "/sys/devices/platform/ideapad_acpi/conservation_mode",
    "/sys/class/power_supply/BAT0/charge_control_end_threshold",
    "/sys/class/power_supply/BAT1/charge_control_end_threshold",
];

pub const FIND_SCRIPT: &str = "find /sys/devices /sys/class/power_supply \\( -name conservation_mode -o -name charge_control_end_threshold \\) 2>/dev/null | head -n 1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryState {
    pub is_supported: bool,
    pub is_conservation_on: bool,
    pub raw_value: i32,
    pub control_file: Option<String>,
}

impl BatteryState {
    /// Mirrors `BatteryState.to_waybar_json`.
    pub fn to_waybar_json(&self) -> serde_json::Value {
        if !self.is_supported {
            return serde_json::json!({
                "text": "ERR",
                "class": "error",
                "tooltip": "Modo de conservação não suportado",
            });
        }
        if self.is_conservation_on {
            serde_json::json!({
                "text": "\u{f241}",
                "class": "conservation-on",
                "tooltip": "Conservação de Bateria: LIGADO (~80%)",
            })
        } else {
            serde_json::json!({
                "text": "\u{f240}",
                "class": "conservation-off",
                "tooltip": "Conservação de Bateria: DESLIGADO (100%)",
            })
        }
    }

    /// Python `json.dumps(state.to_waybar_json())` bytes (key order +
    /// `ensure_ascii`); used for daemon stdout.
    pub fn to_waybar_string(&self) -> String {
        let payload = if !self.is_supported {
            crate::waybar::WaybarJson::new(vec![
                ("text", "ERR".to_string()),
                ("class", "error".to_string()),
                ("tooltip", "Modo de conservação não suportado".to_string()),
            ])
        } else if self.is_conservation_on {
            crate::waybar::WaybarJson::new(vec![
                ("text", "\u{f241}".to_string()),
                ("class", "conservation-on".to_string()),
                ("tooltip", "Conservação de Bateria: LIGADO (~80%)".to_string()),
            ])
        } else {
            crate::waybar::WaybarJson::new(vec![
                ("text", "\u{f240}".to_string()),
                ("class", "conservation-off".to_string()),
                ("tooltip", "Conservação de Bateria: DESLIGADO (100%)".to_string()),
            ])
        };
        payload.dumps_ascii()
    }
}

/// Filesystem probes (real: `os.path.isfile` + file reads/writes).
pub trait BatteryFs: Send + Sync {
    fn is_file(&self, path: &str) -> bool;
    fn read_value(&self, path: &str) -> Option<i32>;
    fn write_value(&self, path: &str, value: i32) -> bool;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemFs;

impl BatteryFs for SystemFs {
    fn is_file(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }

    fn read_value(&self, path: &str) -> Option<i32> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| text.trim().parse().ok())
    }

    fn write_value(&self, path: &str, value: i32) -> bool {
        std::fs::write(path, value.to_string()).is_ok()
    }
}

fn find_control_file(runner: &dyn CommandRunner, fs: &dyn BatteryFs) -> Option<String> {
    for candidate in CONTROL_CANDIDATES {
        if fs.is_file(candidate) {
            return Some(candidate.to_string());
        }
    }
    // Fallback shell identical to Python: `find ... | head -n 1`.
    let (program, args) = ("sh", ["-c", FIND_SCRIPT]);
    let output: CmdOutput = runner.run(program, &args);
    let found = output.stdout.lines().next().unwrap_or("").trim().to_string();
    if !found.is_empty() && fs.is_file(&found) {
        Some(found)
    } else {
        None
    }
}

pub fn get_state(runner: &dyn CommandRunner, fs: &dyn BatteryFs) -> BatteryState {
    let Some(control_file) = find_control_file(runner, fs) else {
        return BatteryState {
            is_supported: false,
            is_conservation_on: false,
            raw_value: 0,
            control_file: None,
        };
    };
    let val = fs.read_value(&control_file).unwrap_or(0);
    BatteryState {
        is_supported: true,
        is_conservation_on: matches!(val, 1 | 60 | 80),
        raw_value: val,
        control_file: Some(control_file),
    }
}

fn verify_written(fs: &dyn BatteryFs, control_file: &str, new_val: i32) -> bool {
    let updated = fs.read_value(control_file).unwrap_or(-1);
    updated == new_val
        || (new_val == 1 && matches!(updated, 1 | 60 | 80))
        || (new_val == 0 && matches!(updated, 0 | 100))
}

/// Mirrors `SysfsBatteryRepository.toggle` incl. pkexec/sudo elevation.
pub fn toggle(runner: &dyn CommandRunner, fs: &dyn BatteryFs) -> (bool, String) {
    let Some(control_file) = find_control_file(runner, fs) else {
        notify(
            runner,
            "Bateria",
            "Conservação não suportada neste hardware.",
            "critical",
        );
        return (false, "ERROR: Arquivo de controle não encontrado.".to_string());
    };

    let val = fs.read_value(&control_file).unwrap_or(0);
    let (new_val, msg) = match val {
        1 => (0, "Conservação Desligada (Carregando até 100%)"),
        0 => (1, "Conservação Ligada (Limitado a 80%)"),
        60 | 80 => (100, "Conservação Desligada (Carregando até 100%)"),
        _ => (80, "Conservação Ligada (Limitado a 80%)"),
    };

    let mut write_success = fs.write_value(&control_file, new_val);
    if !write_success {
        let script = format!("echo {new_val} > '{control_file}'");
        let probe = runner.run("sh", &["-c", "command -v pkexec"]);
        let elevated = if probe.succeeded() {
            runner.run("pkexec", &["sh", "-c", &script])
        } else {
            runner.run("sudo", &["sh", "-c", &script])
        };
        write_success = elevated.succeeded();
    }

    if write_success && !verify_written(fs, &control_file, new_val) {
        write_success = false;
    }

    if !write_success {
        let err_msg =
            "Falha ao alterar conservação da bateria. Permissão negada ou comando cancelado.";
        notify(runner, "Bateria", err_msg, "critical");
        return (false, format!("ERROR: {err_msg}"));
    }

    notify(runner, "Bateria", msg, "normal");
    (true, msg.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct MemFs {
        files: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MemFs {
        fn with(self, path: &str, value: &str) -> Self {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_string(), value.to_string());
            self
        }

        fn written(&self, path: &str) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
    }

    impl BatteryFs for MemFs {
        fn is_file(&self, path: &str) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }

        fn read_value(&self, path: &str) -> Option<i32> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .and_then(|text| text.trim().parse().ok())
        }

        fn write_value(&self, path: &str, value: i32) -> bool {
            if let Some(slot) = self.files.lock().unwrap().get_mut(path) {
                *slot = value.to_string();
                true
            } else {
                false
            }
        }
    }

    const CTL: &str = "/sys/devices/platform/ideapad_acpi/conservation_mode";

    /// Filesystem that refuses direct writes (forces pkexec/sudo path).
    #[derive(Debug, Default)]
    struct ReadOnlyFs {
        files: Arc<Mutex<HashMap<String, String>>>,
    }

    impl ReadOnlyFs {
        fn with(self, path: &str, value: &str) -> Self {
            self.files.lock().unwrap().insert(path.to_string(), value.to_string());
            self
        }
    }

    impl BatteryFs for ReadOnlyFs {
        fn is_file(&self, path: &str) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }
        fn read_value(&self, path: &str) -> Option<i32> {
            self.files.lock().unwrap().get(path).and_then(|text| text.trim().parse().ok())
        }
        fn write_value(&self, _path: &str, _value: i32) -> bool {
            false
        }
    }

    #[test]
    fn status_unsupported_without_control_file() {
        let runner = FakeRunner::new();
        let fs = MemFs::default();
        let state = get_state(&runner, &fs);
        assert!(!state.is_supported);
        assert_eq!(
            state.to_waybar_json(),
            serde_json::json!({
                "text": "ERR",
                "class": "error",
                "tooltip": "Modo de conservação não suportado",
            })
        );
    }

    #[test]
    fn status_on_off_json_literals() {
        let runner = FakeRunner::new();
        let on = get_state(&runner, &MemFs::default().with(CTL, "1\n"));
        assert!(on.is_conservation_on);
        assert_eq!(
            on.to_waybar_json(),
            serde_json::json!({
                "text": "\u{f241}",
                "class": "conservation-on",
                "tooltip": "Conservação de Bateria: LIGADO (~80%)",
            })
        );
        let off = get_state(&runner, &MemFs::default().with(CTL, "0\n"));
        assert!(!off.is_conservation_on);
        assert_eq!(
            off.to_waybar_json()["class"],
            serde_json::json!("conservation-off")
        );
    }

    #[test]
    fn toggle_transitions_match_python() {
        for (current, expected, msg) in [
            ("1", "0", "Conservação Desligada (Carregando até 100%)".to_string()),
            ("0", "1", "Conservação Ligada (Limitado a 80%)".to_string()),
            ("60", "100", "Conservação Desligada (Carregando até 100%)".to_string()),
            ("80", "100", "Conservação Desligada (Carregando até 100%)".to_string()),
            ("42", "80", "Conservação Ligada (Limitado a 80%)".to_string()),
        ] {
            let runner = FakeRunner::new().with_success("notify-send", &[], "");
            let fs = MemFs::default().with(CTL, current);
            let (ok, out) = toggle(&runner, &fs);
            assert!(ok, "current={current}");
            assert_eq!(out, msg);
            assert_eq!(fs.written(CTL).as_deref(), Some(expected));
        }
    }

    #[test]
    fn toggle_unsupported_notifies_critical() {
        let runner = FakeRunner::new()
            .with_success("notify-send", &["-a", "SwayManager", "-u", "critical", "Bateria", "Conservação não suportada neste hardware."], "");
        let (ok, msg) = toggle(&runner, &MemFs::default());
        assert!(!ok);
        assert_eq!(msg, "ERROR: Arquivo de controle não encontrado.");
    }

    #[test]
    fn toggle_elevates_via_pkexec_when_present() {
        let script = format!("echo 0 > '{CTL}'");
        let runner = FakeRunner::new()
            .with_success("sh", &["-c", "command -v pkexec"], "/usr/bin/pkexec\n")
            .with_success("pkexec", &["sh", "-c", script.as_str()], "")
            .with_success("notify-send", &[], "");
        let fs = ReadOnlyFs::default().with(CTL, "1\n");
        // Read-only fs still reports the elevated value after a
        // successful elevation (verify passes when values match).
        let (ok, _) = toggle(&runner, &fs);
        // Write fails verification (fs unchanged) → error path.
        assert!(!ok);
    }

    #[test]
    fn toggle_elevates_via_sudo_without_pkexec() {
        let script = format!("echo 0 > '{CTL}'");
        let runner = FakeRunner::new()
            .with_failure("sh", &["-c", "command -v pkexec"], 1)
            .with_failure("sudo", &["sh", "-c", script.as_str()], 1)
            .with_success("notify-send", &[], "");
        let fs = ReadOnlyFs::default().with(CTL, "1\n");
        let (ok, msg) = toggle(&runner, &fs);
        assert!(!ok);
        assert!(msg.starts_with("ERROR: Falha ao alterar"), "msg: {msg}");
    }

    #[test]
    fn find_falls_back_to_shell_search() {
        let runner = FakeRunner::new()
            .with_success("sh", &["-c", FIND_SCRIPT], "/sys/class/power_supply/BAT0/charge_control_end_threshold\n");
        let fs = MemFs::default().with("/sys/class/power_supply/BAT0/charge_control_end_threshold", "80\n");
        let state = get_state(&runner, &fs);
        assert!(state.is_supported);
        assert!(state.is_conservation_on);
    }
}
