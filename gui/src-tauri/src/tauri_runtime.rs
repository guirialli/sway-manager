//! Runtime Tauri v2: FS real + wrappers `#[tauri::command]`.
//!
//! Os módulos `commands`/`config_center`/`mon_wall`/`portal_freeze` recebem
//! seams (`CommandRunner`, `*Fs`) para teste. Este módulo instancia as
//! implementações reais (`SystemRunner`, `RealFs`, `SystemApplier`) e expõe
//! um comando Tauri por função que o frontend invoca via
//! `window.__TAURI__.core.invoke(name, args)` (nomes snake_case).

use std::collections::BTreeMap;
use sway_manager_daemon::process::SystemRunner;
use sway_manager_daemon::domains::{settings::{appearance, lightdm, theme}, system::battery};
use sway_manager_gui::domains::{settings::{config_center, mon_wall}, system::portal_freeze};
use sway_manager_gui::{BrightnessAction, VolumeAction};

fn home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

/// Implementação real dos três traits de FS (`std::fs` direto).
#[derive(Debug, Default, Clone, Copy)]
pub struct RealFs;

impl appearance::AppearanceFs for RealFs {
    fn is_dir(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }
    fn is_file(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }
    fn list_dir(&self, path: &str) -> Option<Vec<String>> {
        std::fs::read_dir(path).ok().map(|entries| {
            entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
    }
    fn path_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    fn read_index_theme(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn ini_setting(&self, path: &str, key: &str) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let line = line.trim();
            if let Some((k, v)) = line.split_once('=')
                && k.trim() == key
            {
                return Some(v.trim().to_string());
            }
        }
        None
    }
    fn update_ini(&self, path: &str, section: &str, key: &str, value: &str) {
        let mut out: Vec<String> = Vec::new();
        let mut in_section = false;
        let mut done = false;
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    if in_section && !done {
                        out.push(format!("{key}={value}"));
                        done = true;
                    }
                    in_section = trimmed == format!("[{section}]");
                    out.push(line.to_string());
                    continue;
                }
                if in_section && !done && let Some((k, _)) = trimmed.split_once('=') && k.trim() == key {
                    out.push(format!("{key}={value}"));
                    done = true;
                    continue;
                }
                out.push(line.to_string());
            }
        }
        if !done {
            if !in_section {
                out.push(format!("[{section}]"));
            }
            out.push(format!("{key}={value}"));
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, out.join("\n") + "\n");
    }
    fn read_text(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn write_text(&self, path: &str, content: &str) -> bool {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(path, content).is_ok()
    }
}

impl theme::ThemeFs for RealFs {
    fn read_theme(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn write_theme(&self, path: &str, theme: &str) {
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, theme);
    }
    fn kitty_source(&self, path: &str) -> bool {
        std::fs::read_to_string(path).is_ok_and(|c| c.contains("kitty"))
    }
    fn copy_kitty_theme(&self, src: &str, dst: &str) -> Result<(), String> {
        std::fs::copy(src, dst).map(|_| ()).map_err(|e| e.to_string())
    }
}

impl lightdm::LightDmFs for RealFs {
    fn is_file(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }
    fn read_text(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn write_text(&self, path: &str, content: &str) -> bool {
        std::fs::write(path, content).is_ok()
    }
    fn remove(&self, path: &str) {
        let _ = std::fs::remove_file(path);
    }
    fn path_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    fn is_dir(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }
    fn list_dir(&self, path: &str) -> Option<Vec<String>> {
        std::fs::read_dir(path).ok().map(|entries| {
            entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
    }
    fn read_passwd(&self) -> Option<String> {
        std::fs::read_to_string("/etc/passwd").ok()
    }
    fn current_user(&self) -> String {
        std::env::var("USER").unwrap_or_else(|_| "user".to_string())
    }
}

fn runner() -> SystemRunner {
    SystemRunner
}

// --- OSD / popup (routes/osd/{osd,brightness-popup}.html) ---

#[tauri::command]
fn volume_osd(action: VolumeAction) -> Result<sway_manager_gui::VolumeOsd, String> {
    sway_manager_gui::volume_osd(&runner(), action)
}

#[tauri::command]
fn brightness_osd(action: BrightnessAction) -> Result<u32, String> {
    sway_manager_gui::brightness_osd(&runner(), action)
}

#[tauri::command]
fn brightness_popup_get() -> Result<u32, String> {
    sway_manager_gui::domains::osd::commands::brightness_popup_get(&runner())
}

#[tauri::command]
fn brightness_popup_set(value: u32) -> Result<u32, String> {
    sway_manager_gui::domains::osd::commands::brightness_popup_set(&runner(), value)
}

// --- ConfigCenter (routes/settings/config-center.html, 6 páginas) ---

#[tauri::command]
fn monitors_page() -> config_center::MonitorsPage {
    config_center::monitors_page(&runner(), &home())
}

#[tauri::command]
fn monitors_apply(mode: mon_wall::DisplayMode) -> Result<(), String> {
    let daemon_mode = sway_manager_daemon::domains::settings::display::DisplaySwitchType::from(mode);
    config_center::monitors_apply(&runner(), config_center::DisplayMode::from(daemon_mode))
}

#[tauri::command]
fn monitors_set_screenshot_folder(folder: String) -> String {
    config_center::monitors_set_screenshot_folder(&folder, &home())
}

#[tauri::command]
fn wallpaper_page() -> config_center::WallpaperPage {
    config_center::wallpaper_page(&home())
}

#[tauri::command]
fn wallpaper_set_folder(folder: String) -> String {
    config_center::wallpaper_set_folder(&folder, &home())
}

#[tauri::command]
fn appearance_page() -> config_center::AppearancePage {
    let fs = RealFs;
    config_center::appearance_page(&runner(), &fs, &fs, &home())
}

#[tauri::command]
fn appearance_apply(settings: config_center::AppearanceSettingsDto) -> bool {
    let fs = RealFs;
    config_center::appearance_apply(&runner(), &fs, &home(), settings)
}

#[tauri::command]
fn theme_toggle() -> String {
    let fs = RealFs;
    config_center::theme_toggle(&runner(), &fs, &home())
}

#[tauri::command]
fn power_page() -> config_center::PowerPage {
    config_center::power_page(&runner(), &battery::SystemFs)
}

#[tauri::command]
fn battery_toggle() -> config_center::BatteryToggle {
    config_center::battery_toggle(&runner(), &battery::SystemFs)
}

#[tauri::command]
fn power_apply(flag: Option<String>) -> String {
    config_center::power_apply(&runner(), flag)
}

#[tauri::command]
fn idle_page() -> config_center::IdlePage {
    config_center::idle_page(&runner())
}

#[tauri::command]
fn idle_toggle(flag: Option<String>) -> String {
    config_center::idle_toggle(&runner(), &home(), flag)
}
#[tauri::command]
fn portal_sources(stdin_text: Option<String>) -> Result<portal_freeze::PortalSources, String> {
    portal_freeze::portal_sources(&runner(), stdin_text)
}

#[tauri::command]
fn portal_select(app: tauri::AppHandle, result_json: String) -> Result<String, String> {
    // Paridade `PortalResultWriter.write_result`: o xdpw lê a linha no
    // stdout do chooser. Sem isso, Compartilhar validava e não fazia nada.
    // Falhas vão para stderr (nunca stdout: quebraria o protocolo xdpw).
    let line = match portal_freeze::portal_select(&result_json) {
        Ok(line) => line,
        Err(err) => {
            eprintln!("[portal] seleção rejeitada: {err}");
            return Err(err);
        }
    };
    use std::io::Write as _;
    let _ = writeln!(std::io::stdout(), "{line}");
    let _ = std::io::stdout().flush();
    app.exit(0);
    Ok(line)
}

#[tauri::command]
fn portal_cancel(app: tauri::AppHandle) -> Option<String> {
    // Paridade `cancel()`: nada no stdout + exit 0.
    app.exit(0);
    portal_freeze::portal_cancel()
}

#[tauri::command]
fn parse_dmenu_sources(content: String) -> (Vec<portal_freeze::PortalSourceDto>, Vec<portal_freeze::PortalSourceDto>) {
    let (monitors, windows) = portal_freeze::parse_dmenu_sources(&content);
    (
        monitors.iter().map(portal_freeze::PortalSourceDto::from_daemon).collect(),
        windows.iter().map(portal_freeze::PortalSourceDto::from_daemon).collect(),
    )
}

#[tauri::command]
fn freeze_capture(output: String, dst: String) -> Result<String, String> {
    portal_freeze::freeze_capture(&runner(), &output, &dst)
}

#[tauri::command]
fn freeze_image_data(path: String) -> Result<String, String> {
    portal_freeze::freeze_image_data(&path)
}

#[tauri::command]
fn wallpaper_default_folder() -> String {
    mon_wall::wallpaper_default_folder(&home())
}

#[tauri::command]
fn wallpaper_apply(path: String) -> Result<String, String> {
    mon_wall::wallpaper_apply(&runner(), &path)
}

/// Picker: aplica e fecha a janela pelo backend (`window.close()` no
/// WebKitGTK descarrega a página mas deixa o frame fantasma no sway).
#[tauri::command]
fn wallpaper_apply_and_close(window: tauri::Window, path: String) -> Result<String, String> {
    let result = mon_wall::wallpaper_apply(&runner(), &path);
    if result.is_ok() {
        let _ = window.close();
    }
    result
}

// --- LightDM (página 5) + MonitorSwap + WallpaperPicker ---

#[tauri::command]
fn lightdm_page() -> config_center::LightdmPage {
    config_center::lightdm_page(&RealFs)
}

#[tauri::command]
fn lightdm_save(settings: BTreeMap<String, String>, image_source: Option<String>) -> bool {
    config_center::lightdm_save(&runner(), &RealFs, settings, image_source)
}

#[tauri::command]
fn monitor_layouts() -> mon_wall::MonitorLayouts {
    mon_wall::monitor_layouts(&runner())
}

#[tauri::command]
fn monitor_apply(mode: mon_wall::DisplayMode) -> Result<(), String> {
    mon_wall::monitor_apply(&runner(), mode)
}

#[tauri::command]
fn monitor_reload() {
    mon_wall::monitor_reload(&runner());
}

#[tauri::command]
fn wallpaper_list(dir: String) -> Vec<mon_wall::WallpaperEntry> {
    mon_wall::wallpaper_list(&dir)
}
#[tauri::command]
fn freeze_compose(
    selection: portal_freeze::Rect,
    screens: Vec<portal_freeze::FrozenScreenDto>,
) -> Option<portal_freeze::Composition> {
    portal_freeze::freeze_compose(selection, &screens)
}

#[tauri::command]
fn freeze_screens() -> Vec<portal_freeze::FrozenScreenDto> {
    portal_freeze::freeze_screens(&runner())
}

#[tauri::command]
fn freeze_cancel(app: tauri::AppHandle) -> Option<String> {
    // Paridade Qt: cancelar em QUALQUER overlay fecha o app inteiro
    // (o Qt destruía o MultiMonitorSelectionController todo).
    app.exit(0);
    portal_freeze::freeze_cancel()
}

/// OSD auto-close: `window.close()` no WebKitGTK descarrega a página mas
/// deixa o frame fantasma (preto) — o backend encerra o processo.
#[tauri::command]
fn osd_close(app: tauri::AppHandle) {
    app.exit(0);
}
#[tauri::command]
fn screenshot_save_png(data_url: String, folder: Option<String>) -> Result<String, String> {
    portal_freeze::screenshot_save_png(&runner(), &data_url, folder.as_deref().unwrap_or(""), &home())
}

#[tauri::command]
fn screenshot_compose_save(
    selection: portal_freeze::Rect,
    screens: Vec<portal_freeze::FrozenScreenDto>,
    folder: Option<String>,
) -> Result<String, String> {
    portal_freeze::screenshot_compose_save(
        &runner(),
        selection,
        &screens,
        folder.as_deref().unwrap_or(""),
        &home(),
    )
}

#[tauri::command]
fn screenshot_full() -> Result<String, String> {
    portal_freeze::screenshot_full(&runner(), &home())
}

#[tauri::command]
fn screenshot_window() -> Result<String, String> {
    portal_freeze::screenshot_window(&runner(), &home())
}

// --- Polkit (routes/system/polkit.html) ---

#[tauri::command]
fn polkit_state(state: tauri::State<sway_manager_gui::domains::system::polkit_dialog::PolkitDialogState>) -> Result<sway_manager_gui::domains::system::polkit_dialog::PolkitDialogState, String> {
    Ok(state.inner().clone())
}

#[tauri::command]
fn polkit_submit(password: String, pipe: tauri::State<std::sync::Mutex<std::fs::File>>) -> Result<(), String> {
    use std::io::Write as _;
    // INV-POLKIT-4: nenhuma cópia do segredo além de `buf` — `submit_line`
    // recebe `&str` que referencia `buf`, e `line` carrega só o JSON já
    // serializado (que também é zerado abaixo antes do drop).
    let mut buf = password.into_bytes();
    let result = (|| {
        let text = std::str::from_utf8(&buf).map_err(|_| "polkit: senha inválida".to_string())?;
        let mut line = sway_manager_gui::domains::system::polkit_dialog::submit_line(text)
            .ok_or_else(|| "polkit: senha com quebra de linha rejeitada".to_string())?
            .into_bytes();
        let mut pipe = pipe.inner().lock().map_err(|err| format!("polkit: pipe bloqueado: {err}"))?;
        let wrote = pipe.write_all(&line).and_then(|()| pipe.flush()).map_err(|err| format!("polkit: escrita no pipe: {err}"));
        for byte in line.iter_mut() {
            *byte = 0;
        }
        wrote
    })();
    for byte in buf.iter_mut() {
        *byte = 0;
    }
    drop(buf);
    result
}

#[tauri::command]
fn polkit_cancel(pipe: tauri::State<std::sync::Mutex<std::fs::File>>, app: tauri::AppHandle) {
    use std::io::Write as _;
    if let Ok(mut pipe) = pipe.inner().lock() {
        let _ = pipe.write_all(sway_manager_gui::domains::system::polkit_dialog::cancel_line().as_bytes());
        let _ = pipe.flush();
    }
    app.exit(0);
}

/// Todos os comandos invocáveis pelo frontend (`invoke_handler`).
pub fn invoke_handler() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool {
    tauri::generate_handler![
        volume_osd,
        brightness_osd,
        brightness_popup_get,
        brightness_popup_set,
        monitors_page,
        monitors_apply,
        monitors_set_screenshot_folder,
        wallpaper_page,
        wallpaper_set_folder,
        appearance_page,
        appearance_apply,
        theme_toggle,
        power_page,
        battery_toggle,
        power_apply,
        idle_page,
        idle_toggle,
        lightdm_page,
        lightdm_save,
        monitor_layouts,
        monitor_apply,
        monitor_reload,
        wallpaper_list,
        wallpaper_default_folder,
        wallpaper_apply,
        wallpaper_apply_and_close,
        portal_sources,
        portal_select,
        portal_cancel,
        parse_dmenu_sources,
        freeze_capture,
        freeze_compose,
        freeze_screens,
        freeze_cancel,
        osd_close,
        freeze_image_data,
        screenshot_save_png,
        screenshot_compose_save,
        screenshot_full,
        screenshot_window,
        polkit_state,
        polkit_submit,
        polkit_cancel,
    ]
}
