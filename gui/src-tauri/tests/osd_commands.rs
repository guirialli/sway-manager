//! Testes dos comandos OSD — oráculo: `tests/test_gui_components.py`.

use std::collections::HashMap;
use sway_manager_daemon::process::{CmdOutput, CommandRunner};
use sway_manager_gui::{brightness_osd, volume_osd, BrightnessAction, VolumeAction};

#[derive(Default)]
struct FakeRunner {
    outputs: HashMap<(String, String), CmdOutput>,
}

impl FakeRunner {
    fn with(mut self, prog: &str, args: &[&str], stdout: &str) -> Self {
        self.outputs.insert(
            (prog.to_string(), args.join(" ")),
            CmdOutput::success(stdout),
        );
        self
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
        self.outputs
            .get(&(program.to_string(), args.join(" ")))
            .cloned()
            .unwrap_or(CmdOutput::failure(1, "", "no stub"))
    }
    fn spawn_detached(&self, _program: &str, _args: &[&str]) {}
    fn which(&self, _binary: &str) -> Option<String> {
        None
    }
}

#[test]
fn volume_up_returns_state_with_icon() {
    let runner = FakeRunner::default()
        .with("wpctl", &["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"], "")
        .with("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 0.55\n");
    let state = volume_osd(&runner, VolumeAction::Up).unwrap();
    assert!((state.percentage - 55.0).abs() < 1e-9);
    assert!(!state.is_muted);
    assert_eq!(state.icon, "Y");
}

#[test]
fn volume_mute_sets_muted_icon() {
    let runner = FakeRunner::default()
        .with("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", "toggle"], "")
        .with("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 0.00 [MUTED]\n");
    let state = volume_osd(&runner, VolumeAction::Mute).unwrap();
    assert!(state.is_muted);
    assert_eq!(state.icon, "X");
}

#[test]
fn volume_unknown_returns_current_state() {
    let runner = FakeRunner::default()
        .with("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 0.30\n");
    let state = volume_osd(&runner, VolumeAction::Unknown).unwrap();
    assert!((state.percentage - 30.0).abs() < 1e-9);
}

#[test]
fn brightness_up_steps_five_percent() {
    let runner = FakeRunner::default()
        .with("brightnessctl", &["g"], "48000\n")
        .with("brightnessctl", &["m"], "96000\n")
        .with("brightnessctl", &["set", "55%"], "");
    assert_eq!(brightness_osd(&runner, BrightnessAction::Up).unwrap(), 55);
}

#[test]
fn brightness_set_clamps_to_range() {
    let runner = FakeRunner::default().with("brightnessctl", &["set", "100%"], "");
    assert_eq!(brightness_osd(&runner, BrightnessAction::Set(250)).unwrap(), 100);
}

#[test]
fn brightness_unknown_reads_current() {
    let runner = FakeRunner::default()
        .with("brightnessctl", &["g"], "96000\n")
        .with("brightnessctl", &["m"], "96000\n");
    assert_eq!(brightness_osd(&runner, BrightnessAction::Unknown).unwrap(), 100);
}
