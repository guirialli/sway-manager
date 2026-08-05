use crate::domain::entities::PowerProfile;
use crate::domain::traits::PowerRepository;
use std::process::Command;

pub struct SystemPowerRepository;

impl SystemPowerRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemPowerRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerRepository for SystemPowerRepository {
    fn get_profile(&self) -> Result<PowerProfile, String> {
        let output = Command::new("powerprofilesctl")
            .arg("get")
            .output()
            .map_err(|e| format!("Failed to execute powerprofilesctl: {}", e))?;

        if output.status.success() {
            let str_val = String::from_utf8_lossy(&output.stdout);
            Ok(PowerProfile::from_str(&str_val))
        } else {
            Ok(PowerProfile::Balanced)
        }
    }

    fn set_profile(&self, profile: PowerProfile) -> Result<(), String> {
        let profile_str = profile.as_str();
        let status = Command::new("powerprofilesctl")
            .arg("set")
            .arg(profile_str)
            .status()
            .map_err(|e| format!("Failed to run powerprofilesctl set: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Failed to set power profile to {}", profile_str))
        }
    }
}
