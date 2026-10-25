//! Comandos MonitorSwap + WallpaperPicker (slice B do porte).
//!
//! Espelha `MonitorSwapWindow` (`src/presentation/gui/windows/monitor_swap.py`)
//! e `WallpaperPickerWindow` + `CarregadorDeImagens`
//! (`src/presentation/gui/windows/wallpaper_picker.py`,
//! `src/presentation/gui/components/image_list.py`).
//!
//! Ação → repos do daemon via [`CommandRunner`] → JSON pronto p/ o frontend.
//! Na fase 2 ganham `#[tauri::command]`; as assinaturas já são compatíveis
//! (`Result<_, String>`, args `Deserialize`, frontend `camelCase`).

use serde::{Deserialize, Serialize};
use sway_manager_daemon::process::{notify, CommandRunner};
use sway_manager_daemon::domains::settings::{display, wallpaper, wallpaper_thumbs};

/// Modo de disposição de telas vindo do frontend.
///
/// Paridade `DisplaySwitchType`
/// (`src/domain/display/entities.py`): o frontend envia `camelCase`
/// (`pcOnly` | `monitorOnly` | `extend` | `duplicate`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DisplayMode {
    PcOnly,
    MonitorOnly,
    Extend,
    Duplicate,
}

impl From<DisplayMode> for display::DisplaySwitchType {
    fn from(mode: DisplayMode) -> Self {
        match mode {
            DisplayMode::PcOnly => Self::PcOnly,
            DisplayMode::MonitorOnly => Self::MonitorOnly,
            DisplayMode::Extend => Self::Extend,
            DisplayMode::Duplicate => Self::Duplicate,
        }
    }
}

impl From<display::DisplaySwitchType> for DisplayMode {
    fn from(mode: display::DisplaySwitchType) -> Self {
        match mode {
            display::DisplaySwitchType::PcOnly => Self::PcOnly,
            display::DisplaySwitchType::MonitorOnly => Self::MonitorOnly,
            display::DisplaySwitchType::Extend => Self::Extend,
            display::DisplaySwitchType::Duplicate => Self::Duplicate,
        }
    }
}

/// Estado serializado para a janela MonitorSwap.
///
/// Paridade `MonitorSwapWindow.initUI`: nomes interno/externo p/ rotular os
/// botões, contagem p/ validar e layout atual p/ destacar o ativo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonitorLayouts {
    pub interno: Option<String>,
    pub externo: Option<String>,
    pub count: usize,
    pub current: DisplayMode,
}

/// Levanta monitores (`get_monitors`), contagem (`count`) e layout atual.
///
/// Na fase Tauri: `#[tauri::command] fn monitor_layouts()`.
/// Nunca falha: sem dois monitores, `interno`/`externo` vêm `None`
/// (paridade `ValueError("Apenas um monitor encontrado!")` absorvido aqui;
/// `monitor_apply` o propaga como `Err`).
pub fn monitor_layouts(runner: &dyn CommandRunner) -> MonitorLayouts {
    let monitors = display::get_monitors(runner).ok();
    MonitorLayouts {
        interno: monitors.as_ref().map(|m| m.interno.clone()),
        externo: monitors.as_ref().map(|m| m.externo.clone()),
        count: display::connected_monitors_count(runner),
        current: DisplayMode::from(display::current_layout(runner)),
    }
}

/// Aplica o modo (`apply_config`); erro vira `String` p/ rejeitar o `invoke`.
///
/// Paridade `MonitorSwapWindow.apply_config`: erro → caixa de mensagem com
/// `str(e)`; aqui o `Err(String)` cumpre esse papel no frontend.
///
/// Na fase Tauri: `#[tauri::command] fn monitor_apply(mode: DisplayMode)`.
pub fn monitor_apply(runner: &dyn CommandRunner, mode: DisplayMode) -> Result<(), String> {
    display::apply_config(runner, display::DisplaySwitchType::from(mode))
}

/// Recarrega o sway (`swaymsg reload`) — revert da confirmação de 15 s.
///
/// Paridade `MonitorSwapWindow.confirmar_ou_reverter` → `recarregar_sway`.
///
/// Na fase Tauri: `#[tauri::command] fn monitor_reload()`.
pub fn monitor_reload(runner: &dyn CommandRunner) {
    display::reload_sway(runner);
}

/// Extensões aceitas, paridade `CarregadorDeImagens.run`.
const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

fn is_image_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    IMAGE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

/// Item da galeria: full-res + preview pronta para exibir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperEntry {
    /// Caminho full-res (aplicar).
    pub path: String,
    /// Preview 320px em data URL, pronto para `<img>` (via daemon).
    pub thumb: String,
}

/// Lista imagens do diretório, ordenadas (paridade `CarregadorDeImagens`).
///
/// Dir inexistente → `vec![]` (paridade
/// `test_loader_run_with_missing_dir_does_not_crash`).
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_list(dir: String)`.
pub fn wallpaper_list(dir: &str) -> Vec<WallpaperEntry> {
    wallpaper_list_in(dir, &wallpaper_thumbs::default_cache_dir(&home_dir()))
}

/// `wallpaper_list` com cache de thumbs injetável (testes).
pub fn wallpaper_list_in(dir: &str, cache_dir: &std::path::Path) -> Vec<WallpaperEntry> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<WallpaperEntry> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_image_file)
        })
        .map(|p| {
            let path = p.to_string_lossy().into_owned();
            let thumb = wallpaper_thumbs::thumbnail_data_url(&p, cache_dir);
            WallpaperEntry { path, thumb }
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// Pasta inicial da galeria (paridade `SetWallpaperUseCase.get_wallpaper_folder`).
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_default_folder()`.
pub fn wallpaper_default_folder(home: &str) -> String {
    wallpaper::wallpaper_folder(home, &|p| std::path::Path::new(p).is_dir())
}

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "~".to_string())
}

/// Atualiza o symlink `~/.config/sway/wallpaper` (best-effort, paridade
/// `SwayWallpaperRepository.set_wallpaper`: aviso em vez de erro).
fn refresh_wallpaper_symlink(home: &str, image_path: &str) {
    let link = format!("{home}/{}", wallpaper::WALLPAPER_SYMLINK_SUFFIX);
    let link_path = std::path::Path::new(&link);
    if let Some(parent) = link_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if link_path.is_symlink() || link_path.exists() {
        let _ = std::fs::remove_file(link_path);
    }
    #[cfg(unix)]
    let _ = std::os::unix::fs::symlink(image_path, link_path);
}

/// Aplica o papel de parede: valida, escreve a stanza `42-wallpaper`,
/// atualiza o symlink e notifica (paridade `set_wallpaper` + notify).
///
/// Caminho vazio/inexistente → `Err` e nada é escrito (paridade
/// `test_apply_without_path_is_noop`).
pub fn wallpaper_apply_to(
    runner: &dyn CommandRunner,
    path: &str,
    home: &str,
) -> Result<String, String> {
    if path.is_empty() || !std::path::Path::new(path).is_file() {
        return Err(format!("Arquivo de imagem não encontrado: {path}"));
    }
    let config_d = format!("{home}/.config/sway/config.d");
    let stanza_path = wallpaper::write_wallpaper_stanza(runner, &config_d, path)?;
    refresh_wallpaper_symlink(home, path);
    notify(
        runner,
        "SwayManager",
        &format!("Papel de parede aplicado!\n{path}"),
        "normal",
    );
    Ok(stanza_path)
}

/// Aplica o papel de parede usando `$HOME` (atalho p/ o frontend).
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_apply(path: String)`.
pub fn wallpaper_apply(runner: &dyn CommandRunner, path: &str) -> Result<String, String> {
    wallpaper_apply_to(runner, path, &home_dir())
}
