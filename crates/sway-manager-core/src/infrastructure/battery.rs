use crate::domain::entities::BatteryState;
use crate::domain::traits::BatteryRepository;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct SystemBatteryRepository;

impl SystemBatteryRepository {
    pub fn new() -> Self {
        Self
    }

    fn find_conservation_mode_path() -> Option<PathBuf> {
        let candidate_paths = [
            "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode",
            "/sys/devices/platform/ideapad/conservation_mode",
            "/sys/bus/platform/drivers/ideapad_acpi/conservation_mode",
        ];

        for path_str in candidate_paths {
            let path = Path::new(path_str);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }
        None
    }

    fn read_battery_percentage() -> Option<u8> {
        let bat_paths = ["/sys/class/power_supply/BAT0/capacity", "/sys/class/power_supply/BAT1/capacity"];
        for path_str in bat_paths {
            if let Ok(content) = fs::read_to_string(path_str) {
                if let Ok(val) = content.trim().parse::<u8>() {
                    return Some(val);
                }
            }
        }
        None
    }

    fn read_battery_status() -> String {
        let status_paths = ["/sys/class/power_supply/BAT0/status", "/sys/class/power_supply/BAT1/status"];
        for path_str in status_paths {
            if let Ok(content) = fs::read_to_string(path_str) {
                return content.trim().to_string();
            }
        }
        "Unknown".to_string()
    }
}

impl Default for SystemBatteryRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryRepository for SystemBatteryRepository {
    fn get_state(&self) -> Result<BatteryState, String> {
        let conservation_path = Self::find_conservation_mode_path();
        let enabled = if let Some(path) = conservation_path {
            fs::read_to_string(&path)
                .map(|val| val.trim() == "1")
                .unwrap_or(false)
        } else {
            false
        };

        Ok(BatteryState {
            conservation_mode_enabled: enabled,
            percentage: Self::read_battery_percentage(),
            status: Self::read_battery_status(),
        })
    }

    fn toggle_conservation_mode(&self) -> Result<bool, String> {
        let path = Self::find_conservation_mode_path()
            .ok_or_else(|| "Lenovo IdeaPad conservation_mode sysfs path not found".to_string())?;

        let current = fs::read_to_string(&path)
            .map(|val| val.trim() == "1")
            .unwrap_or(false);

        let target_val = if current { "0" } else { "1" };

        // Try direct write first
        if fs::write(&path, target_val).is_ok() {
            return Ok(!current);
        }

        // If direct write fails (permission denied), try pkexec / tee
        let status = Command::new("pkexec")
            .arg("tee")
            .arg(&path)
            .input_text(target_val)
            .status()
            .map_err(|e| format!("Failed to run pkexec tee to write conservation_mode: {}", e))?;

        if status.success() {
            Ok(!current)
        } else {
            Err("Permission denied while toggling battery conservation mode".to_string())
        }
    }
}

trait CommandInputExt {
    fn input_text(&mut self, text: &str) -> &mut Self;
}

impl CommandInputExt for Command {
    fn input_text(&mut self, text: &str) -> &mut Self {
        use std::io::Write;
        use std::process::Stdio;

        self.stdin(Stdio::piped());
        self.stdout(Stdio::null());
        let mut child = match self.spawn() {
            Ok(c) => c,
            Err(_) => return self,
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        self
    }
}
