//! Comandos OSD (fase 1 do porte): volume + brilho.
//!
//! Espelham `VolumeOSD` / `BrightnessOSD` + `BrightnessPopup`
//! (`src/presentation/gui/osd/*`, `windows/brightness_popup.py`):
//! ação → step executado via repos do daemon → estado serializado p/ o frontend.
//! Na fase 2 ganham `#[tauri::command]`; a assinatura já é compatível
//! (`Result<_, String>`, args `Deserialize`).

use serde::{Deserialize, Serialize};
use sway_manager_daemon::process::CommandRunner;
use sway_manager_daemon::domains::osd::audio::{
    adjust_volume, brightness_percent, change_brightness, get_volume_state,
};

/// Ação de volume vinda do frontend (`up` | `down` | `mute`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VolumeAction {
    Up,
    Down,
    Mute,
    /// Qualquer outra string cai aqui e retorna só o estado (paridade Qt:
    /// ação desconhecida → `get_state()`).
    #[serde(other)]
    Unknown,
}

/// Estado serializado para o OSD de volume.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VolumeOsd {
    pub percentage: f64,
    pub is_muted: bool,
    /// Ícone textual (paridade `MixerAudioRepository`: `X` mudo / `Y` som).
    pub icon: &'static str,
}

/// Ação de brilho (`up` | `down` | valor absoluto 1–100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrightnessAction {
    Up,
    Down,
    /// Valor absoluto; fora de 1–100 é clampado pelo repo.
    Set(u32),
    #[serde(other)]
    Unknown,
}

/// Executa a ação de volume e retorna o estado p/ renderizar o OSD.
///
/// Na fase Tauri: `#[tauri::command] fn volume_osd(action: VolumeAction)`.
pub fn volume_osd(
    runner: &dyn CommandRunner,
    action: VolumeAction,
) -> Result<VolumeOsd, String> {
    let state = match action {
        VolumeAction::Up => adjust_volume(runner, "up"),
        VolumeAction::Down => adjust_volume(runner, "down"),
        VolumeAction::Mute => adjust_volume(runner, "mute"),
        VolumeAction::Unknown => get_volume_state(runner),
    };
    Ok(VolumeOsd {
        percentage: state.percentage,
        is_muted: state.is_muted,
        icon: sway_manager_daemon::domains::osd::audio::volume_icon(&state),
    })
}

/// Executa a ação de brilho e retorna o percentual p/ renderizar o OSD/popup.
///
/// Na fase Tauri: `#[tauri::command] fn brightness_osd(action: BrightnessAction)`.
pub fn brightness_osd(
    runner: &dyn CommandRunner,
    action: BrightnessAction,
) -> Result<u32, String> {
    match action {
        BrightnessAction::Up => Ok(change_brightness(runner, "up", 5)),
        BrightnessAction::Down => Ok(change_brightness(runner, "down", 5)),
        BrightnessAction::Set(v) => {
            let target = v.clamp(1, 100);
            runner.run("brightnessctl", &["set", &format!("{target}%")]);
            Ok(target)
        }
        BrightnessAction::Unknown => Ok(brightness_percent(runner)),
    }
}

/// Percentual atual de brilho p/ inicializar o slider do popup.
///
/// Paridade `BrightnessPopup.__init__` (`use_case.get_current()`).
/// Na fase Tauri: `#[tauri::command] fn brightness_popup_get()`.
pub fn brightness_popup_get(runner: &dyn CommandRunner) -> Result<u32, String> {
    Ok(brightness_percent(runner))
}

/// Ajuste pós-release do popup: clamp + `brightnessctl set` + `ddcutil setvcp`.
///
/// Paridade `BrightnessPopup.set_brightness` + `close_app` /
/// `set_brightness_ext_monitor` (`windows/brightness_popup.py`):
/// o slider aplica `brightnessctl` contínuo e, no release, espelha no monitor
/// externo via `ddcutil setvcp 10 <v-10> --bus=6 --noverify`
/// (`100` vai cheio; resto perde 10; mínimo 1).
/// Na fase Tauri: `#[tauri::command] fn brightness_popup_set(value: u32)`.
pub fn brightness_popup_set(runner: &dyn CommandRunner, value: u32) -> Result<u32, String> {
    let target = value.clamp(1, 100);
    runner.run("brightnessctl", &["set", &format!("{target}%")]);
    let ext = if target == 100 {
        100
    } else {
        (target as i64 - 10).max(1) as u32
    };
    runner.run(
        "ddcutil",
        &["setvcp", "10", &ext.to_string(), "--bus=6", "--noverify"],
    );
    Ok(target)
}
