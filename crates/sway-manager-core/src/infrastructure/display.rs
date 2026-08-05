use crate::domain::entities::DisplayLayout;
use crate::domain::traits::DisplayRepository;
use swayipc::Connection;

pub struct SystemDisplayRepository;

impl SystemDisplayRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemDisplayRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayRepository for SystemDisplayRepository {
    fn apply_layout(&self, layout: DisplayLayout) -> Result<(), String> {
        let mut connection = Connection::new().map_err(|e| format!("Failed to connect to Sway IPC: {}", e))?;
        let outputs = connection.get_outputs().map_err(|e| format!("Failed to get Sway outputs: {}", e))?;

        let internal = outputs.iter().find(|o| o.name.starts_with("eDP") || o.name.starts_with("LVDS"));
        let external = outputs.iter().find(|o| !o.name.starts_with("eDP") && !o.name.starts_with("LVDS"));

        match layout {
            DisplayLayout::Dual => {
                if let (Some(int), Some(ext)) = (internal, external) {
                    let cmd = format!("output {} enable; output {} enable right_of {}", int.name, ext.name, int.name);
                    let _ = connection.run_command(cmd);
                }
            }
            DisplayLayout::PcOnly => {
                if let Some(int) = internal {
                    let cmd = format!("output {} enable", int.name);
                    let _ = connection.run_command(cmd);
                }
                if let Some(ext) = external {
                    let cmd = format!("output {} disable", ext.name);
                    let _ = connection.run_command(cmd);
                }
            }
            DisplayLayout::ExternalOnly => {
                if let Some(ext) = external {
                    let cmd = format!("output {} enable", ext.name);
                    let _ = connection.run_command(cmd);
                }
                if let Some(int) = internal {
                    let cmd = format!("output {} disable", int.name);
                    let _ = connection.run_command(cmd);
                }
            }
            DisplayLayout::Swap => {
                if let (Some(int), Some(ext)) = (internal, external) {
                    let cmd = format!("output {} toggle; output {} toggle", int.name, ext.name);
                    let _ = connection.run_command(cmd);
                }
            }
        }
        Ok(())
    }
}
