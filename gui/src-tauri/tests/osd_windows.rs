//! Testes do slice OSD + BrightnessPopup — oráculo: `tests/test_gui_components.py`
//! (`TestVolumeOSD`, `TestBrightnessOSD`, `TestBrightnessPopup`).
//!
//! Não toca em `tests/osd_commands.rs` (slice legado); este arquivo cobre o
//! contrato novo: `brightness_popup_get` / `brightness_popup_set`.

use std::collections::HashMap;
use std::sync::Mutex;
use sway_manager_daemon::process::{CmdOutput, CommandRunner};
use sway_manager_gui::domains::osd::commands::{
    brightness_popup_get, brightness_popup_set, brightness_osd, volume_osd, BrightnessAction,
    VolumeAction,
};

#[derive(Default)]
struct FakeRunner {
    outputs: HashMap<(String, String), CmdOutput>,
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl FakeRunner {
    fn with(mut self, prog: &str, args: &[&str], stdout: &str) -> Self {
        self.outputs.insert(
            (prog.to_string(), args.join(" ")),
            CmdOutput::success(stdout),
        );
        self
    }

    fn calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }

    fn argv_of(&self, prog: &str) -> Vec<Vec<String>> {
        self.calls()
            .into_iter()
            .filter(|(p, _)| p == prog)
            .map(|(_, a)| a)
            .collect()
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
        self.calls
            .lock()
            .unwrap()
            .push((program.to_string(), args.iter().map(|s| s.to_string()).collect()));
        self.outputs
            .get(&(program.to_string(), args.join(" ")))
            .cloned()
            .unwrap_or(CmdOutput::success(""))
    }
    fn spawn_detached(&self, _program: &str, _args: &[&str]) {}
    fn which(&self, _binary: &str) -> Option<String> {
        None
    }
}

// Espelho `TestBrightnessPopup::test_slider_inicia_no_brilho_atual`
// (popup.brilho == 70, slider inicia no brilho atual).
#[test]
fn popup_get_returns_current_percent() {
    let runner = FakeRunner::default()
        .with("brightnessctl", &["g"], "67200\n")
        .with("brightnessctl", &["m"], "96000\n");
    assert_eq!(brightness_popup_get(&runner).unwrap(), 70);
}

// Espelho `TestBrightnessPopup::test_set_brightness_repasa_ao_repositorio` +
// `test_mover_slider_chama_set_brightness` (slider → set_brightness(42/25)).
#[test]
fn popup_set_applies_brightnessctl() {
    let runner = FakeRunner::default().with("brightnessctl", &["set", "42%"], "");
    assert_eq!(brightness_popup_set(&runner, 42).unwrap(), 42);
    let sets: Vec<Vec<String>> = runner.argv_of("brightnessctl");
    assert!(sets.iter().any(|a| a == &["set", "42%"]));
}

// Espelho `TestBrightnessPopup::test_close_app_dispara_ddcutil`
// (set 80 → `ddcutil setvcp 10` com "70" = 80-10 do monitor externo).
#[test]
fn popup_set_release_calls_ddcutil_with_offset() {
    let runner = FakeRunner::default().with("brightnessctl", &["set", "80%"], "");
    assert_eq!(brightness_popup_set(&runner, 80).unwrap(), 80);
    let ddc: Vec<Vec<String>> = runner.argv_of("ddcutil");
    assert_eq!(ddc.len(), 1);
    assert_eq!(ddc[0][..3], ["setvcp", "10", "70"]);
    assert_eq!(ddc[0][3..], ["--bus=6", "--noverify"]);
}

// Clamp paridade `BrightnessAction::Set` (fora de 1–100 é clampado).
#[test]
fn popup_set_clamps_to_range() {
    let runner = FakeRunner::default()
        .with("brightnessctl", &["set", "100%"], "")
        .with("brightnessctl", &["set", "1%"], "");
    assert_eq!(brightness_popup_set(&runner, 250).unwrap(), 100);
    assert_eq!(brightness_popup_set(&runner, 0).unwrap(), 1);
    // 100 vai cheio no externo; 1 não underflowa (mínimo 1).
    let ddc: Vec<Vec<String>> = runner.argv_of("ddcutil");
    assert!(ddc.iter().any(|a| a[2] == "100"));
    assert!(ddc.iter().any(|a| a[2] == "1"));
}

// Espelho `VolumeOSD` ramo `else → get_state()` (ação desconhecida só lê).
#[test]
fn volume_unknown_returns_current_state() {
    let runner = FakeRunner::default()
        .with("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"], "Volume: 0.30\n");
    let state = volume_osd(&runner, VolumeAction::Unknown).unwrap();
    assert!((state.percentage - 30.0).abs() < 1e-9);
    assert!(!state.is_muted);
    assert!(runner.argv_of("wpctl").iter().all(|a| a[0] != "set-volume"));
}

// Espelho `BrightnessOSD` leitura atual (unknown → percentual sem step).
#[test]
fn brightness_unknown_reads_current() {
    let runner = FakeRunner::default()
        .with("brightnessctl", &["g"], "57600\n")
        .with("brightnessctl", &["m"], "96000\n");
    assert_eq!(brightness_osd(&runner, BrightnessAction::Unknown).unwrap(), 60);
    assert!(runner.argv_of("brightnessctl").iter().all(|a| a[0] != "set"));
}
