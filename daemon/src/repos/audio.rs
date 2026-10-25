//! `osd volume` / `brightness` — wpctl + brightnessctl wrappers.
//!
//! The OSD windows stay in Python; the daemon-executed half (status
//! queries and step changes) is ported 1:1 from
//! `MixerAudioRepository`, `AdjustVolumeUseCase`,
//! `BrightnessctlRepository` and `SetBrightnessUseCase`.

use crate::process::CommandRunner;

#[derive(Debug, Clone, PartialEq)]
pub struct VolumeState {
    pub percentage: f64,
    pub is_muted: bool,
}

/// Mirrors `MixerAudioRepository.get_volume_state` parsing.
///
/// Leitura clampada em 100%: o sink pode estar acima (setado por outro
/// app), mas o SwayManager nunca exibe nem opera acima do teto.
pub fn get_volume_state(runner: &dyn CommandRunner) -> VolumeState {
    let output = runner.run("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"]);
    let text = output.stdout;
    let is_muted = text.contains("[MUTED]");
    let mut volume = 0.0;
    for word in text.split_whitespace() {
        if let Ok(value) = word.parse::<f64>() {
            volume = (value * 100.0).min(100.0);
            break;
        }
    }
    VolumeState { percentage: volume, is_muted }
}

pub fn volume_icon(state: &VolumeState) -> &'static str {
    if state.is_muted { "X" } else { "Y" }
}

/// Mirrors `AdjustVolumeUseCase.execute_action` (`5%+`/`5%-`/toggle).
///
/// Desvio intencional da paridade: `up` usa `wpctl -l 1.0` para travar
/// em 100% (o wpctl puro deixa passar — `1.00 + 5%+ = 1.05`).
pub fn adjust_volume(runner: &dyn CommandRunner, action: &str) -> VolumeState {
    match action {
        "up" => runner.run("wpctl", &["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"]),
        "down" => runner.run("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"]),
        "mute" => runner.run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]),
        _ => crate::process::CmdOutput::success(""),
    };
    get_volume_state(runner)
}

/// Mirrors `BrightnessctlRepository.get_current_percentage`.
pub fn brightness_percent(runner: &dyn CommandRunner) -> u32 {
    let cur = runner.run("brightnessctl", &["g"]).stdout.trim().parse::<i64>().unwrap_or(0);
    let max = runner.run("brightnessctl", &["m"]).stdout.trim().parse::<i64>().unwrap_or(0);
    if max == 0 {
        return 0;
    }
    ((cur * 100) / max).clamp(0, 100) as u32
}

/// Mirrors `SetBrightnessUseCase.execute_change` + clamped `set_brightness`.
pub fn change_brightness(runner: &dyn CommandRunner, direction: &str, step: i64) -> u32 {
    let cur = brightness_percent(runner) as i64;
    let target = match direction {
        "up" => cur + step,
        "down" => cur - step,
        _ => cur,
    }
    .clamp(1, 100) as u32;
    runner.run("brightnessctl", &["set", &format!("{target}%")]);
    target
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    #[test]
    fn volume_parse_muted_and_level() {
        let runner = FakeRunner::new()
            .with_success("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 0.42 [MUTED]\n");
        let state = get_volume_state(&runner);
        assert!(state.is_muted);
        assert!((state.percentage - 42.0).abs() < f64::EPSILON);
        assert_eq!(volume_icon(&state), "X");
    }

    #[test]
    fn volume_up_uses_step_argv() {
        let runner = FakeRunner::new()
            .with_success("wpctl", &["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"], "")
            .with_success("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 0.50\n");
        let state = adjust_volume(&runner, "up");
        assert!(!state.is_muted);
        assert!((state.percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn volume_up_never_exceeds_100() {
        // Regressão: `1.00 + 5%+` ia a 1.05 — o `-l 1.0` trava no wpctl
        // e a leitura clampa (sink pode estar acima via outro app).
        let runner = FakeRunner::new()
            .with_success("wpctl", &["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"], "")
            .with_success("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 1.00\n");
        let state = adjust_volume(&runner, "up");
        assert!(state.percentage <= 100.0, "teto: {}", state.percentage);
        let runs = runner.runs();
        assert!(
            runs.iter().any(|c| c.contains(&"-l".to_string()) && c.contains(&"1.0".to_string())),
            "up sem limitador: {runs:?}"
        );
    }

    #[test]
    fn volume_read_clamps_overdriven_sink() {
        let runner = FakeRunner::new().with_success(
            "wpctl",
            &["get-volume", "@DEFAULT_AUDIO_SINK@"],
            "Volume: 1.50\n",
        );
        assert_eq!(get_volume_state(&runner).percentage, 100.0);
    }

    #[test]
    fn brightness_clamped_and_stepped() {
        let runner = FakeRunner::new()
            .with_success("brightnessctl", &["g"], "48000\n")
            .with_success("brightnessctl", &["m"], "96000\n")
            .with_success("brightnessctl", &["set", "55%"], "");
        assert_eq!(change_brightness(&runner, "up", 5), 55);
        let runner = FakeRunner::new()
            .with_success("brightnessctl", &["g"], "x\n")
            .with_success("brightnessctl", &["m"], "0\n");
        assert_eq!(brightness_percent(&runner), 0);
    }

    #[test]
    fn brightness_up_never_exceeds_100() {
        // Regressão: teto simétrico ao volume — em 100%, `up` mantém 100%.
        let runner = FakeRunner::new()
            .with_success("brightnessctl", &["g"], "21333\n")
            .with_success("brightnessctl", &["m"], "21333\n")
            .with_success("brightnessctl", &["set", "100%"], "");
        assert_eq!(change_brightness(&runner, "up", 5), 100);
        let runs = runner.runs();
        assert!(
            runs.iter().any(|c| c == &["brightnessctl", "set", "100%"]),
            "set fora do teto: {runs:?}"
        );
    }
}
