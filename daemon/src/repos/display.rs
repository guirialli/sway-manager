//! `monitor` helpers — swaymsg layouts and monitor discovery.
//!
//! The `MonitorSwapWindow` stays in Python. The ported contract is the
//! exact `swaymsg` argv per `DisplaySwitchType` and the `eDP`/suffix
//! interno-vs-externo heuristic (`SwayDisplayRepository.get_monitors`).

use crate::process::CommandRunner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySwitchType {
    PcOnly,
    MonitorOnly,
    Extend,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitors {
    pub interno: String,
    pub externo: String,
}

/// Mirrors the name extraction `re.findall(r'"name":\s*"([^"]+)"', ...)`.
pub fn parse_monitor_names(swaymsg_output: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = swaymsg_output;
    while let Some(start) = rest.find("\"name\"") {
        rest = &rest[start + 6..];
        let rest_trimmed = rest.trim_start_matches([':', ' ', '\t']);
        if let Some(quoted) = rest_trimmed.strip_prefix('"')
            && let Some(end) = quoted.find('"')
        {
            names.push(quoted[..end].to_string());
            rest = &quoted[end + 1..];
            continue;
        }
    }
    names
}

/// Mirrors the `get_monitors` interno/externo heuristic.
pub fn classify_monitors(names: &[String]) -> Result<Monitors, String> {
    if names.len() < 2 {
        return Err("Apenas um monitor encontrado!".to_string());
    }
    if names[1].starts_with("eDP") || names[0].ends_with('2') {
        Ok(Monitors { interno: names[1].clone(), externo: names[0].clone() })
    } else {
        Ok(Monitors { interno: names[0].clone(), externo: names[1].clone() })
    }
}

/// Exact swaymsg command strings per mode (`apply_config` lines 30-45).
pub fn apply_command(mode: DisplaySwitchType, monitors: &Monitors) -> String {
    let (i, e) = (monitors.interno.as_str(), monitors.externo.as_str());
    match mode {
        DisplaySwitchType::PcOnly => {
            format!("swaymsg output {i} enable, output {e} disable")
        }
        DisplaySwitchType::MonitorOnly => {
            format!("swaymsg output {i} disable, output {e} enable")
        }
        DisplaySwitchType::Extend => {
            format!("swaymsg output {i} enable, output {e} enable")
        }
        DisplaySwitchType::Duplicate => {
            format!("swaymsg output {i} enable, output {e} enable; pkill wl-mirror; wl-mirror {i}")
        }
    }
}

pub fn query_outputs(runner: &dyn CommandRunner) -> Vec<String> {
    let output = runner.run("sh", &["-c", "swaymsg -t get_outputs | grep \"\\\"name\\\":\""]);
    parse_monitor_names(&output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_and_layout_strings() {
        let names = vec!["HDMI-1".to_string(), "eDP-1".to_string()];
        let monitors = classify_monitors(&names).unwrap();
        assert_eq!(monitors.interno, "eDP-1");
        assert_eq!(
            apply_command(DisplaySwitchType::PcOnly, &monitors),
            "swaymsg output eDP-1 enable, output HDMI-1 disable"
        );
        assert!(
            apply_command(DisplaySwitchType::Duplicate, &monitors)
                .contains("pkill wl-mirror; wl-mirror eDP-1")
        );
        assert!(classify_monitors(&["only".to_string()]).is_err());
    }

    #[test]
    fn parse_names_from_swaymsg() {
        let out = "{\"name\": \"HDMI-A-1\"}, {\"name\": \"eDP-1\"}";
        assert_eq!(parse_monitor_names(out), vec!["HDMI-A-1", "eDP-1"]);
    }
}
