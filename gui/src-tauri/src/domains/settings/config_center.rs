//! ConfigCenter (slice C do porte): 6 páginas agregando os repos do daemon.
//!
//! Espelha `ConfigCenterWindow` (`src/presentation/gui/windows/config_center.py`):
//! sidebar de 6 itens → `QStackedWidget` de 6 páginas; cada página tem um
//! getter retornando JSON pronto p/ renderizar + applies que despacham aos
//! repos. Nenhum acesso a `/sys`, `swaymsg` ou binários fora dos repos.
//! Na fase 2 cada `pub fn` ganha `#[tauri::command]`; as assinaturas já são
//! compatíveis (`Result<_, String>`, args `Deserialize`, frontend `camelCase`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sway_manager_daemon::process::CommandRunner;
use sway_manager_daemon::domains::{settings::{appearance, app_config, display, lightdm, theme, wallpaper, wallpaper_thumbs}, system::{battery, idle, power, screenshot}};

/// Itens da sidebar, paridade `setup_ui` (`config_center.py:108-115`):
/// rótulos PT-BR idênticos ao Qt, `UserRole` = índice da página.
pub const SIDEBAR_ITEMS: [(&str, usize); 6] = [
    ("🖥️  Monitores", 0),
    ("🖼️  Papel de Parede", 1),
    ("🌓  Aparência & Tema", 2),
    ("🔋  Bateria & Energia", 3),
    ("☕  Suspensão & Idle", 4),
    ("🚦  Login & LightDM", 5),
];

/// Página inicial, paridade `sidebar_list.setCurrentRow(0)` (`:145`).
pub const DEFAULT_PAGE: usize = 0;

/// Guarda do `switch_page` (`:147-149`): índice válido → `Some`, senão `None`.
pub fn select_page(index: i32) -> Option<usize> {
    if (0..SIDEBAR_ITEMS.len() as i32).contains(&index) {
        Some(index as usize)
    } else {
        None
    }
}

/// Modo de disposição de telas vindo do frontend, paridade `DisplaySwitchType`.
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

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "~".to_string())
}

// ---------------------------------------------------------------
// Página 0: Monitores & Telas
// ---------------------------------------------------------------

/// Estado da página 0: layout atual + pasta de screenshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorsPage {
    pub interno: Option<String>,
    pub externo: Option<String>,
    pub count: usize,
    pub current: DisplayMode,
    pub screenshot_folder: String,
}

/// Paridade `create_monitors_page`: nomes p/ rotular botões, contagem,
/// layout atual p/ destacar o ativo + `pasta_screenshots` (`:76-77,:173-246`).
///
/// Na fase Tauri: `#[tauri::command] fn monitors_page()`.
pub fn monitors_page(runner: &dyn CommandRunner, home: &str) -> MonitorsPage {
    let monitors = display::get_monitors(runner).ok();
    let saved = app_config::get_setting(&app_config::default_config_path(home), "screenshot_folder")
        .and_then(|v| v.as_str().map(str::to_string));
    let folder = screenshot::resolve_screenshot_folder(
        runner,
        saved.as_deref(),
        home,
        &|p| std::path::Path::new(p).is_dir(),
    );
    MonitorsPage {
        interno: monitors.as_ref().map(|m| m.interno.clone()),
        externo: monitors.as_ref().map(|m| m.externo.clone()),
        count: display::connected_monitors_count(runner),
        current: DisplayMode::from(display::current_layout(runner)),
        screenshot_folder: folder,
    }
}

/// Paridade `aplicar_layout_monitor` → `display_use_case.execute(mode)` (`:248-253`).
///
/// Na fase Tauri: `#[tauri::command] fn monitors_apply(mode: DisplayMode)`.
pub fn monitors_apply(runner: &dyn CommandRunner, mode: DisplayMode) -> Result<(), String> {
    display::apply_config(runner, display::DisplaySwitchType::from(mode))
}

/// Paridade `selecionar_pasta_screenshot` → `set_screenshot_folder` (`:291-298`):
/// `~`-expande, persiste em `screenshot_folder` e devolve o normalizado.
///
/// Na fase Tauri: `#[tauri::command] fn monitors_set_screenshot_folder(folder: String)`.
pub fn monitors_set_screenshot_folder(folder: &str, home: &str) -> String {
    let expanded = screenshot::normalize_screenshot_folder(folder, home);
    app_config::set_setting(
        &app_config::default_config_path(home),
        "screenshot_folder",
        serde_json::Value::String(expanded.clone()),
    );
    expanded
}

// ---------------------------------------------------------------
// Página 1: Papel de Parede
// ---------------------------------------------------------------

const IMAGE_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

fn is_image_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{ext}")))
}

/// Paridade `CarregadorDeImagens.run`: só imagens, ordenadas; dir ausente → `vec![]`.
pub fn wallpaper_images(dir: &str) -> Vec<String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(is_image_file))
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

/// Read-back do wallpaper atual: symlink `~/.config/sway/wallpaper` e depois
pub fn current_wallpaper(home: &str) -> Option<String> {
    let link = format!("{home}/{}", wallpaper::WALLPAPER_SYMLINK_SUFFIX);
    if let Ok(target) = std::fs::read_link(&link)
        && let Some(target) = target.to_str()
        && std::path::Path::new(target).is_file()
    {
        return Some(target.to_string());
    }
    if std::path::Path::new(&link).is_file() {
        return Some(link);
    }
    let stanza_file = format!("{home}/{}", wallpaper::STANZA_FILE_SUFFIX);
    if let Ok(content) = std::fs::read_to_string(&stanza_file)
        && let Some(path) = wallpaper::parse_bg_path(&content)
        && std::path::Path::new(&path).is_file()
    {
        return Some(path);
    }
    None
}

/// Item da grade: full-res + preview em cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperImage {
    /// Caminho full-res (aplicar, comparar com atual).
    pub path: String,
    /// Preview 320px em data URL, pronto para `<img>` (via daemon).
    pub thumb: String,
}

/// Estado da página 1: pasta + atual + grade.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperPage {
    pub folder: String,
    pub current: Option<String>,
    pub images: Vec<WallpaperImage>,
}

/// Paridade `create_wallpaper_page` + `pasta_imagens` (`:76,:303-357`).
///
/// Thumbs pré-processados pelo daemon (`domains::settings::wallpaper_thumbs`): a GUI só
/// repassa a data URL pronta para exibir.
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_page()`.
pub fn wallpaper_page(home: &str) -> WallpaperPage {
    wallpaper_page_in(home, &wallpaper_thumbs::default_cache_dir(home))
}

/// `wallpaper_page` com cache de thumbs injetável (testes).
pub fn wallpaper_page_in(home: &str, cache_dir: &std::path::Path) -> WallpaperPage {
    let folder = wallpaper::wallpaper_folder(home, &|p| std::path::Path::new(p).is_dir());
    let images = wallpaper_images(&folder)
        .into_iter()
        .map(|path| {
            let thumb = wallpaper_thumbs::thumbnail_data_url(std::path::Path::new(&path), cache_dir);
            WallpaperImage { path, thumb }
        })
        .collect();
    WallpaperPage { folder, current: current_wallpaper(home), images }
}

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

/// Paridade `aplicar_wallpaper` + `set_wallpaper` + notify; sem path → `Err`
/// e nada é escrito (paridade `test_apply_wallpaper_without_path_is_noop`).
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_apply_to(path: String)`.
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
    sway_manager_daemon::process::notify(
        runner,
        "Papel de Parede",
        &format!("Aplicado: {}", std::path::Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path)),
        "normal",
    );
    Ok(stanza_path)
}

/// Atalho `$HOME` p/ o invoke do frontend.
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_apply(path: String)`.
pub fn wallpaper_apply(runner: &dyn CommandRunner, path: &str) -> Result<String, String> {
    wallpaper_apply_to(runner, path, &home_dir())
}

/// Paridade `selecionar_pasta` → `set_wallpaper_folder` (`:359-367`).
///
/// Na fase Tauri: `#[tauri::command] fn wallpaper_set_folder(folder: String)`.
pub fn wallpaper_set_folder(folder: &str, home: &str) -> String {
    let expanded = wallpaper::normalize_wallpaper_folder(folder, home);
    app_config::set_setting(
        &app_config::default_config_path(home),
        "wallpaper_folder",
        serde_json::Value::String(expanded.clone()),
    );
    expanded
}

// ---------------------------------------------------------------
// Página 2: Aparência & Tema
// ---------------------------------------------------------------

/// DTO de `AppearanceSettings` p/ o wire (`camelCase` no JSON).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppearanceSettingsDto {
    pub gtk_theme: String,
    pub icon_theme: String,
    pub cursor_theme: String,
    pub font_name: String,
}

impl From<&appearance::AppearanceSettings> for AppearanceSettingsDto {
    fn from(settings: &appearance::AppearanceSettings) -> Self {
        Self {
            gtk_theme: settings.gtk_theme.clone(),
            icon_theme: settings.icon_theme.clone(),
            cursor_theme: settings.cursor_theme.clone(),
            font_name: settings.font_name.clone(),
        }
    }
}

impl From<&AppearanceSettingsDto> for appearance::AppearanceSettings {
    fn from(dto: &AppearanceSettingsDto) -> Self {
        Self {
            gtk_theme: dto.gtk_theme.clone(),
            icon_theme: dto.icon_theme.clone(),
            cursor_theme: dto.cursor_theme.clone(),
            font_name: dto.font_name.clone(),
        }
    }
}

/// DTO de `AvailableAppearanceOptions` p/ o wire.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceOptionsDto {
    pub gtk_themes: Vec<String>,
    pub icon_themes: Vec<String>,
    pub cursor_themes: Vec<String>,
    pub fonts: Vec<String>,
}

impl From<&appearance::AvailableAppearanceOptions> for AppearanceOptionsDto {
    fn from(options: &appearance::AvailableAppearanceOptions) -> Self {
        Self {
            gtk_themes: options.gtk_themes.clone(),
            icon_themes: options.icon_themes.clone(),
            cursor_themes: options.cursor_themes.clone(),
            fonts: options.fonts.clone(),
        }
    }
}

/// Estado da página 2: tema atual + settings + opções instaladas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearancePage {
    pub current_theme: String,
    pub settings: AppearanceSettingsDto,
    pub options: AppearanceOptionsDto,
}

/// Paridade `create_theme_page`: `get_state` + `get_available_options` +
/// `get_appearance_settings` (`:432-463`).
///
/// Na fase Tauri: `#[tauri::command] fn appearance_page()`.
pub fn appearance_page(
    runner: &dyn CommandRunner,
    appearance_fs: &dyn appearance::AppearanceFs,
    theme_fs: &dyn theme::ThemeFs,
    home: &str,
) -> AppearancePage {
    let settings = appearance::get_appearance_settings(runner, appearance_fs, home);
    let options = appearance::get_available_options(runner, appearance_fs, home);
    AppearancePage {
        current_theme: theme::get_state(theme_fs, home).current_theme,
        settings: AppearanceSettingsDto::from(&settings),
        options: AppearanceOptionsDto::from(&options),
    }
}

/// Paridade `aplicar_config_aparencia` (`:511-522`).
///
/// Na fase Tauri: `#[tauri::command] fn appearance_apply(settings: AppearanceSettingsDto)`.
pub fn appearance_apply(
    runner: &dyn CommandRunner,
    appearance_fs: &dyn appearance::AppearanceFs,
    home: &str,
    settings: AppearanceSettingsDto,
) -> bool {
    appearance::apply_appearance(runner, appearance_fs, home, &appearance::AppearanceSettings::from(&settings))
}

/// Paridade `toggle_theme` (`:505-509`): alterna e devolve o literal do repo.
///
/// Na fase Tauri: `#[tauri::command] fn theme_toggle()`.
pub fn theme_toggle(
    runner: &dyn CommandRunner,
    theme_fs: &dyn theme::ThemeFs,
    home: &str,
) -> String {
    theme::toggle(runner, theme_fs, &theme::SystemApplier, home)
}

// ---------------------------------------------------------------
// Página 3: Bateria & Energia
// ---------------------------------------------------------------

/// Estado da página 3: conservação + perfil ativo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerPage {
    pub supported: bool,
    pub conservation_on: bool,
    pub control_file: Option<String>,
    pub active_profile: String,
    pub profile_label: String,
}

/// Paridade `create_power_page`: estado da bateria + perfil (`:528-580`).
///
/// Na fase Tauri: `#[tauri::command] fn power_page()`.
pub fn power_page(runner: &dyn CommandRunner, battery_fs: &dyn battery::BatteryFs) -> PowerPage {
    let state = battery::get_state(runner, battery_fs);
    let profile = power::get_state(runner);
    PowerPage {
        supported: state.is_supported,
        conservation_on: state.is_conservation_on,
        control_file: state.control_file,
        profile_label: power::friendly_name(&profile.active_profile).to_string(),
        active_profile: profile.active_profile,
    }
}

/// Resultado do toggle de conservação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryToggle {
    pub success: bool,
    pub message: String,
}

/// Paridade `battery_use_case.execute()` (`:550`).
///
/// Na fase Tauri: `#[tauri::command] fn battery_toggle()`.
pub fn battery_toggle(runner: &dyn CommandRunner, battery_fs: &dyn battery::BatteryFs) -> BatteryToggle {
    let (success, message) = battery::toggle(runner, battery_fs);
    BatteryToggle { success, message }
}

/// Paridade `power_use_case.execute(flag)` (`:564-568`): `-p`/`-b`/`-s`/rotação.
/// `None`/desconhecido rotaciona (paridade `TogglePowerProfileUseCase.execute`).
///
/// Na fase Tauri: `#[tauri::command] fn power_apply(flag: Option<String>)`.
pub fn power_apply(runner: &dyn CommandRunner, flag: Option<String>) -> String {
    power::execute(runner, flag.as_deref())
}

// ---------------------------------------------------------------
// Página 4: Suspensão & Idle
// ---------------------------------------------------------------

/// Estado da página 4: inibidor + timeout documentado do repo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdlePage {
    pub running: bool,
    pub timeout_sleep: u32,
}

/// Paridade `create_idle_page` (`:585-616`).
///
/// Na fase Tauri: `#[tauri::command] fn idle_page()`.
pub fn idle_page(runner: &dyn CommandRunner) -> IdlePage {
    IdlePage { running: idle::get_state(runner).is_running, timeout_sleep: idle::TIMEOUT_SLEEP }
}

/// Paridade `idle_use_case.execute()` (`:606`).
///
/// Na fase Tauri: `#[tauri::command] fn idle_toggle(flag: Option<String>)`.
pub fn idle_toggle(runner: &dyn CommandRunner, home: &str, flag: Option<String>) -> String {
    idle::toggle(runner, home, flag.as_deref())
}

// ---------------------------------------------------------------
// Página 5: Login & LightDM
// ---------------------------------------------------------------

/// Estado da página 5: settings + usuários + sessões.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightdmPage {
    pub settings: BTreeMap<String, String>,
    pub users: Vec<String>,
    pub sessions: Vec<String>,
}

/// Paridade `create_lightdm_page`: `get_settings` + `get_available_users` +
/// `get_available_sessions` (`:635-640`).
///
/// Na fase Tauri: `#[tauri::command] fn lightdm_page()`.
pub fn lightdm_page(fs: &dyn lightdm::LightDmFs) -> LightdmPage {
    LightdmPage {
        settings: lightdm::get_settings(fs, lightdm::GREETER_CONFIG, lightdm::DAEMON_CONFIG).to_dict(),
        users: lightdm::get_available_users(fs),
        sessions: lightdm::get_available_sessions(fs),
    }
}

/// Paridade `salvar_config_lightdm` → `lightdm_use_case.execute` (`:1069-1111`):
/// eleva via `pkexec`, nunca toca em polkit/sysfs aqui.
///
/// Na fase Tauri: `#[tauri::command] fn lightdm_save(settings: ..., imageSource: ...)`.
#[allow(clippy::too_many_arguments)]
pub fn lightdm_save(
    runner: &dyn CommandRunner,
    fs: &dyn lightdm::LightDmFs,
    settings: BTreeMap<String, String>,
    image_source: Option<String>,
) -> bool {
    let parsed = lightdm::LightDmSettings::from_dict(&settings);
    lightdm::save_settings(
        runner,
        fs,
        &parsed,
        image_source.as_deref(),
        lightdm::GREETER_CONFIG,
        lightdm::DAEMON_CONFIG,
        lightdm::SYSTEM_WALLPAPER,
        lightdm::TMP_GREETER_PATH,
        lightdm::TMP_DAEMON_PATH,
    )
}
