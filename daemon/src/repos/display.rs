//! `monitor` helpers — swaymsg layouts and monitor discovery.
//!
//! Mirrors `SwayDisplayRepository`
//! (`src/infrastructure/display/sway_display_repository.py:1-78`): exact
//! `swaymsg` argv per `DisplaySwitchType`, the `eDP`/suffix
//! interno-vs-externo heuristic (`get_monitors`), `validar_monitores`,
//! `recarregar_sway` (`swaymsg reload`), `get_connected_monitors_count`
//! (`swaymsg -t get_outputs` JSON length, fallback `1`) and
//! `get_current_layout` (`PC_ONLY`/`MONITOR_ONLY`/`EXTEND`/`DUPLICATE`
//! via active outputs + `pgrep wl-mirror`).
//!
//! `DUPLICATE` runs the `swaymsg output ... enable` argv, kills the stale
//! mirror, then spawns `wl-mirror <interno>` detached; the other modes run
//! the exact `swaymsg output ...` argv.

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
        rest = &rest[1.min(rest.len())..];
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

/// Mirrors `validar_monitores` (lines 20-25): at least two names or the
/// Python `ValueError` literal.
pub fn validate_monitors(runner: &dyn CommandRunner) -> Result<(), String> {
    if query_outputs(runner).len() < 2 {
        return Err("Apenas um monitor encontrado!".to_string());
    }
    Ok(())
}

/// Mirrors `get_monitors` (lines 9-18): validate then classify.
pub fn get_monitors(runner: &dyn CommandRunner) -> Result<Monitors, String> {
    validate_monitors(runner)?;
    classify_monitors(&query_outputs(runner))
}

/// Mirrors `recarregar_sway` (lines 43-44): `swaymsg reload`.
pub fn reload_sway(runner: &dyn CommandRunner) {
    runner.run("swaymsg", &["reload"]);
}

/// Mirrors `get_connected_monitors_count` (lines 46-55): length of the
/// `swaymsg -t get_outputs` JSON array, `1` on any failure.
pub fn connected_monitors_count(runner: &dyn CommandRunner) -> usize {
    let out = runner.run("swaymsg", &["-t", "get_outputs"]);
    if out.succeeded()
        && !out.stdout.trim().is_empty()
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&out.stdout)
        && let Some(items) = value.as_array()
    {
        return items.len();
    }
    1
}

fn output_active(outputs: &[serde_json::Value], name: &str) -> bool {
    outputs.iter().any(|o| {
        o.get("name").and_then(|n| n.as_str()) == Some(name)
            && o.get("active").and_then(|a| a.as_bool()).unwrap_or(true)
    })
}

/// Mirrors `get_current_layout` (lines 57-78): active flags from
/// `swaymsg -t get_outputs` plus `pgrep wl-mirror` to split
/// `EXTEND`/`DUPLICATE`; any failure falls back to `EXTEND`.
pub fn current_layout(runner: &dyn CommandRunner) -> DisplaySwitchType {
    let out = runner.run("swaymsg", &["-t", "get_outputs"]);
    if out.succeeded()
        && !out.stdout.trim().is_empty()
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&out.stdout)
        && let Some(outputs) = value.as_array()
        && let Ok(monitors) = get_monitors(runner)
    {
        let int_active = output_active(outputs, &monitors.interno);
        let ext_active = output_active(outputs, &monitors.externo);
        if int_active && !ext_active {
            return DisplaySwitchType::PcOnly;
        }
        if ext_active && !int_active {
            return DisplaySwitchType::MonitorOnly;
        }
        if int_active && ext_active {
            if runner.run("pgrep", &["wl-mirror"]).succeeded() {
                return DisplaySwitchType::Duplicate;
            }
            return DisplaySwitchType::Extend;
        }
    }
    DisplaySwitchType::Extend
}

/// Mirrors `apply_config` (lines 27-41): resolve monitors, run the exact
/// `swaymsg output ...` argv; `DUPLICATE` additionally kills the stale
/// `wl-mirror` and spawns `wl-mirror <interno>` detached.
pub fn apply_config(runner: &dyn CommandRunner, mode: DisplaySwitchType) -> Result<(), String> {
    let monitors = get_monitors(runner)?;
    let (i, e) = (monitors.interno.as_str(), monitors.externo.as_str());
    match mode {
        DisplaySwitchType::PcOnly => {
            runner.run("swaymsg", &["output", i, "enable,", "output", e, "disable"]);
        }
        DisplaySwitchType::MonitorOnly => {
            runner.run("swaymsg", &["output", i, "disable,", "output", e, "enable"]);
        }
        DisplaySwitchType::Extend => {
            runner.run("swaymsg", &["output", i, "enable,", "output", e, "enable"]);
        }
        DisplaySwitchType::Duplicate => {
            runner.run("swaymsg", &["output", i, "enable,", "output", e, "enable"]);
            runner.run("pkill", &["wl-mirror"]);
            runner.spawn_detached("wl-mirror", &[i]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

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

    fn grep_runner(names_stdout: &str) -> FakeRunner {
        FakeRunner::new().with_success(
            "sh",
            &["-c", "swaymsg -t get_outputs | grep \"\\\"name\\\":\""],
            names_stdout,
        )
    }

    #[test]
    fn get_monitors_validates_and_classifies() {
        // Mirrors SwayDisplayRepository.get_monitors / validar_monitores.
        let runner = grep_runner("\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n");
        let monitors = get_monitors(&runner).unwrap();
        assert_eq!(monitors.interno, "eDP-1");
        assert_eq!(monitors.externo, "HDMI-A-1");
        assert!(validate_monitors(&runner).is_ok());
    }

    #[test]
    fn get_monitors_rejects_single_output() {
        // Mirrors `ValueError("Apenas um monitor encontrado!")`.
        let runner = grep_runner("\"name\": \"eDP-1\"\n");
        assert_eq!(
            get_monitors(&runner),
            Err("Apenas um monitor encontrado!".to_string())
        );
        assert_eq!(
            validate_monitors(&runner),
            Err("Apenas um monitor encontrado!".to_string())
        );
    }

    #[test]
    fn reload_sway_runs_reload_argv() {
        // Mirrors recarregar_sway: `subprocess.run(["swaymsg", "reload"])`.
        let runner = FakeRunner::new().with_success("swaymsg", &["reload"], "");
        reload_sway(&runner);
        assert!(runner.run("swaymsg", &["reload"]).succeeded());
    }

    #[test]
    fn connected_count_parses_json_outputs() {
        // Mirrors tests/test_display_repository.py::test_get_connected_monitors_count.
        let runner = FakeRunner::new().with_success(
            "swaymsg",
            &["-t", "get_outputs"],
            r#"[{"name": "eDP-1"}, {"name": "HDMI-A-1"}]"#,
        );
        assert_eq!(connected_monitors_count(&runner), 2);
    }

    #[test]
    fn connected_count_falls_back_to_one() {
        let runner = FakeRunner::new().with_failure("swaymsg", &["-t", "get_outputs"], 1);
        assert_eq!(connected_monitors_count(&runner), 1);
        let bad_json = FakeRunner::new().with_success("swaymsg", &["-t", "get_outputs"], "nope");
        assert_eq!(connected_monitors_count(&bad_json), 1);
    }

    #[test]
    fn current_layout_pc_only() {
        // Mirrors tests/test_display_repository.py::test_get_current_layout_pc_only.
        let runner = grep_runner("\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n").with_success(
            "swaymsg",
            &["-t", "get_outputs"],
            r#"[{"name": "eDP-1", "active": true}, {"name": "HDMI-A-1", "active": false}]"#,
        );
        assert_eq!(current_layout(&runner), DisplaySwitchType::PcOnly);
    }

    #[test]
    fn current_layout_duplicate_needs_wl_mirror() {
        let both_active =
            r#"[{"name": "eDP-1", "active": true}, {"name": "HDMI-A-1", "active": true}]"#;
        let base = |pgrep_ok: bool| {
            let mut r = grep_runner("\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n")
                .with_success("swaymsg", &["-t", "get_outputs"], both_active);
            if pgrep_ok {
                r = r.with_success("pgrep", &["wl-mirror"], "123\n");
            } else {
                r = r.with_failure("pgrep", &["wl-mirror"], 1);
            }
            r
        };
        assert_eq!(current_layout(&base(true)), DisplaySwitchType::Duplicate);
        assert_eq!(current_layout(&base(false)), DisplaySwitchType::Extend);
    }

    #[test]
    fn current_layout_monitor_only_and_fallback() {
        let runner = grep_runner("\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n").with_success(
            "swaymsg",
            &["-t", "get_outputs"],
            r#"[{"name": "eDP-1", "active": false}, {"name": "HDMI-A-1", "active": true}]"#,
        );
        assert_eq!(current_layout(&runner), DisplaySwitchType::MonitorOnly);
        let broken = FakeRunner::new().with_failure("swaymsg", &["-t", "get_outputs"], 1);
        assert_eq!(current_layout(&broken), DisplaySwitchType::Extend);
    }

    #[test]
    fn apply_config_duplicate_spawns_mirror() {
        let runner = grep_runner("\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n")
            .with_success(
                "swaymsg",
                &["output", "eDP-1", "enable,", "output", "HDMI-A-1", "enable"],
                "",
            )
            .with_success("pkill", &["wl-mirror"], "");
        apply_config(&runner, DisplaySwitchType::Duplicate).unwrap();
        assert_eq!(
            runner.spawned(),
            vec![vec!["wl-mirror".to_string(), "eDP-1".to_string()]]
        );
    }

    #[test]
    fn apply_config_pc_only_runs_swaymsg_argv() {
        let runner = grep_runner("\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n").with_success(
            "swaymsg",
            &["output", "eDP-1", "enable,", "output", "HDMI-A-1", "disable"],
            "",
        );
        apply_config(&runner, DisplaySwitchType::PcOnly).unwrap();
        assert!(runner.spawned().is_empty());
    }
}
