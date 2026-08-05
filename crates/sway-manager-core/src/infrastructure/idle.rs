use crate::domain::entities::IdleState;
use crate::domain::traits::IdleRepository;
use std::process::Command;

pub struct SystemIdleRepository;

impl SystemIdleRepository {
    pub fn new() -> Self {
        Self
    }

    fn is_swayidle_running() -> bool {
        Command::new("pgrep")
            .arg("-x")
            .arg("swayidle")
            .output()
            .map(|output| output.status.success() && !output.stdout.is_empty())
            .unwrap_or(false)
    }
}

impl Default for SystemIdleRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl IdleRepository for SystemIdleRepository {
    fn get_state(&self) -> Result<IdleState, String> {
        let is_running = Self::is_swayidle_running();
        Ok(IdleState {
            is_inhibited: !is_running,
        })
    }

    fn toggle_idle(&self) -> Result<bool, String> {
        let is_running = Self::is_swayidle_running();
        if is_running {
            // Kill swayidle to inhibit idle
            let status = Command::new("pkill")
                .arg("-x")
                .arg("swayidle")
                .status()
                .map_err(|e| format!("Failed to run pkill swayidle: {}", e))?;
            if status.success() {
                Ok(true) // Inhibited
            } else {
                Err("Failed to stop swayidle process".to_string())
            }
        } else {
            // Spawn swayidle in background if configuration exists or run default swayidle
            let status = Command::new("sh")
                .arg("-c")
                .arg("swayidle -w timeout 300 'swaylock -f' timeout 600 'swaymsg \"output * dpms off\"' resume 'swaymsg \"output * dpms on\"' &")
                .status()
                .map_err(|e| format!("Failed to start swayidle: {}", e))?;
            if status.success() {
                Ok(false) // Not inhibited
            } else {
                Err("Failed to start swayidle process".to_string())
            }
        }
    }
}
