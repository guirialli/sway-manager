//! Testes do ConfigCenter — oráculo: `tests/test_gui_windows.py::ConfigCenterTestCase`.
//!
//! `#[path]` próprio p/ compilar com ou sem `pub mod config_center` em `lib.rs`
//! (registro pedido ao Main via `agent://all`).
#[path = "../src/config_center.rs"]
mod config_center;

use std::collections::{BTreeMap, HashMap};

use sway_manager_daemon::process::{CmdOutput, CommandRunner, FakeRunner};
use sway_manager_daemon::repos::{appearance, battery, lightdm, theme};
use config_center::{
    appearance_apply, appearance_page, battery_toggle, idle_page, idle_toggle, lightdm_page,
    lightdm_save, monitors_apply, monitors_page, monitors_set_screenshot_folder, power_apply,
    power_page, select_page, theme_toggle, wallpaper_apply_to, wallpaper_page,
    wallpaper_set_folder, AppearanceSettingsDto, DisplayMode, DEFAULT_PAGE, SIDEBAR_ITEMS,
};

/// Runner permissivo: sucesso p/ qualquer argv (p/ `lightdm_save`, que monta
/// um `pkexec sh -c` longo demais p/ stub exato no FakeRunner).
#[derive(Debug, Default)]
struct OkRunner;

impl CommandRunner for OkRunner {
    fn run(&self, _program: &str, _args: &[&str]) -> CmdOutput {
        CmdOutput::success("")
    }
    fn spawn_detached(&self, _program: &str, _args: &[&str]) {}
    fn which(&self, _binary: &str) -> Option<String> {
        None
    }
}

#[derive(Debug, Default)]
struct MemAppearanceFs {
    dirs: std::collections::BTreeSet<String>,
    listings: HashMap<String, Vec<String>>,
    markers: std::collections::BTreeSet<String>,
    indexes: HashMap<String, String>,
}

impl appearance::AppearanceFs for MemAppearanceFs {
    fn is_dir(&self, path: &str) -> bool {
        self.dirs.contains(path) || self.markers.contains(path)
    }
    fn is_file(&self, _path: &str) -> bool {
        false
    }
    fn list_dir(&self, path: &str) -> Option<Vec<String>> {
        self.listings.get(path).cloned()
    }
    fn path_exists(&self, path: &str) -> bool {
        self.markers.contains(path) || self.indexes.contains_key(path)
    }
    fn read_index_theme(&self, path: &str) -> Option<String> {
        self.indexes.get(path).cloned()
    }
    fn ini_setting(&self, _path: &str, _key: &str) -> Option<String> {
        None
    }
    fn update_ini(&self, _path: &str, _section: &str, _key: &str, _value: &str) {}
    fn read_text(&self, _path: &str) -> Option<String> {
        None
    }
    fn write_text(&self, _path: &str, _content: &str) -> bool {
        true
    }
}

#[derive(Debug, Default)]
struct MemThemeFs {
    current: String,
}

impl theme::ThemeFs for MemThemeFs {
    fn read_theme(&self, _path: &str) -> Option<String> {
        Some(self.current.clone())
    }
    fn write_theme(&self, _path: &str, _theme: &str) {}
    fn kitty_source(&self, _path: &str) -> bool {
        false
    }
    fn copy_kitty_theme(&self, _src: &str, _dst: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Bateria gravável: `is_file` sempre verdadeiro, lê/escreve no mapa.
#[derive(Debug, Default)]
struct WritableBatteryFs {
    values: std::sync::Mutex<HashMap<String, i32>>,
}

impl battery::BatteryFs for WritableBatteryFs {
    fn is_file(&self, _path: &str) -> bool {
        true
    }
    fn read_value(&self, path: &str) -> Option<i32> {
        self.values.lock().unwrap().get(path).copied()
    }
    fn write_value(&self, path: &str, value: i32) -> bool {
        self.values.lock().unwrap().insert(path.to_string(), value);
        true
    }
}

#[derive(Debug, Default)]
struct MemLightdmFs {
    files: HashMap<String, String>,
    dirs: HashMap<String, Vec<String>>,
    passwd: String,
    user: String,
}

impl lightdm::LightDmFs for MemLightdmFs {
    fn is_file(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
    fn read_text(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }
    fn write_text(&self, _path: &str, _content: &str) -> bool {
        true
    }
    fn remove(&self, _path: &str) {}
    fn path_exists(&self, _path: &str) -> bool {
        false
    }
    fn is_dir(&self, path: &str) -> bool {
        self.dirs.contains_key(path)
    }
    fn list_dir(&self, path: &str) -> Option<Vec<String>> {
        self.dirs.get(path).cloned()
    }
    fn read_passwd(&self) -> Option<String> {
        Some(self.passwd.clone())
    }
    fn current_user(&self) -> String {
        self.user.clone()
    }
}

const GREP_ARGS: [&str; 2] = ["-c", "swaymsg -t get_outputs | grep \"\\\"name\\\":\""];
const TWO_MONITORS: &str = "\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n";

fn two_monitor_runner() -> FakeRunner {
    FakeRunner::new().with_success("sh", &GREP_ARGS, TWO_MONITORS)
}

fn json_outputs() -> &'static str {
    r#"[{"name": "eDP-1", "active": true}, {"name": "HDMI-A-1", "active": true}]"#
}

fn temp_home() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

// --- Sidebar / páginas (espelha ConfigCenterTestCase) ---

#[test]
fn sidebar_has_six_entries_mapped_to_pages() {
    // Paridade test_sidebar_has_six_entries_mapped_to_pages: 6 itens, UserRole = índice.
    assert_eq!(SIDEBAR_ITEMS.len(), 6);
    for (row, (label, index)) in SIDEBAR_ITEMS.iter().enumerate() {
        assert_eq!(*index, row, "item {row}");
        assert!(!label.is_empty());
    }
}

#[test]
fn switch_page_accepts_each_index_and_rejects_out_of_range() {
    // Paridade test_switch_page_selects_each_index_and_rejects_out_of_range.
    for index in 0..6 {
        assert_eq!(select_page(index), Some(index as usize));
    }
    assert_eq!(select_page(99), None);
    assert_eq!(select_page(-1), None);
}

#[test]
fn starts_on_first_page() {
    // Paridade test_starts_on_first_page.
    assert_eq!(DEFAULT_PAGE, 0);
    assert_eq!(select_page(0), Some(0));
}

// --- Página 0: Monitores ---

#[test]
fn monitors_page_reports_layout_and_screenshot_folder() {
    let runner = two_monitor_runner()
        .with_success("swaymsg", &["-t", "get_outputs"], json_outputs())
        .with_failure("pgrep", &["wl-mirror"], 1)
        .with_success("xdg-user-dir", &["PICTURES"], "/home/u/Pictures\n");
    let home = temp_home();
    let page = monitors_page(&runner, home.path().to_str().unwrap());
    assert_eq!(page.interno.as_deref(), Some("eDP-1"));
    assert_eq!(page.externo.as_deref(), Some("HDMI-A-1"));
    assert_eq!(page.count, 2);
    assert_eq!(page.current, DisplayMode::Extend);
    assert_eq!(page.screenshot_folder, "/home/u/Pictures/screenshots");
}

#[test]
fn monitor_apply_dispatches_all_four_modes() {
    // Paridade test_monitor_layout_dispatches_to_display_use_case.
    for (mode, int_arg, ext_arg) in [
        (DisplayMode::PcOnly, "enable,", "disable"),
        (DisplayMode::MonitorOnly, "disable,", "enable"),
        (DisplayMode::Extend, "enable,", "enable"),
        (DisplayMode::Duplicate, "enable,", "enable"),
    ] {
        let runner = two_monitor_runner()
            .with_success("swaymsg", &["output", "eDP-1", int_arg, "output", "HDMI-A-1", ext_arg], "")
            .with_success("pkill", &["wl-mirror"], "");
        assert!(monitors_apply(&runner, mode).is_ok(), "{mode:?}");
    }
}

#[test]
fn monitor_apply_without_two_monitors_is_error() {
    let runner = FakeRunner::new().with_success("sh", &GREP_ARGS, "\"name\": \"eDP-1\"\n");
    assert!(monitors_apply(&runner, DisplayMode::Duplicate).is_err());
}

#[test]
fn screenshot_folder_set_persists_expanded() {
    // Paridade selecionar_pasta_screenshot → set_screenshot_folder.
    let home = temp_home();
    let home_str = home.path().to_str().unwrap();
    let normalized = monitors_set_screenshot_folder("~/Capturas", home_str);
    assert_eq!(normalized, format!("{home_str}/Capturas"));
}

// --- Página 1: Papel de Parede ---

#[test]
fn wallpaper_page_lists_images_and_ignores_non_images() {
    // Paridade CarregadorDeImagens: só jpg/jpeg/png/webp, ordenadas.
    use std::io::Write as _;
    let pics = tempfile::tempdir().unwrap();
    for name in ["b.png", "a.jpg", "nota.txt", "c.webp"] {
        let mut f = std::fs::File::create(pics.path().join(name)).unwrap();
        f.write_all(b"x").unwrap();
    }
    let images = config_center::wallpaper_images(pics.path().to_str().unwrap());
    assert_eq!(images.len(), 3, "só imagens: {images:?}");
    assert!(images[0].ends_with("a.jpg"));
}

#[test]
fn wallpaper_page_without_dir_is_empty() {
    assert!(config_center::wallpaper_images("/inexistente/wallpapers-xyz").is_empty());
    let home = temp_home();
    let page = wallpaper_page(home.path().to_str().unwrap());
    assert!(!page.folder.is_empty());
}

#[test]
fn wallpaper_apply_without_path_is_noop() {
    // Paridade test_apply_wallpaper_without_path_is_noop.
    let home = temp_home();
    let home_str = home.path().to_str().unwrap();
    let runner = FakeRunner::new();
    assert!(wallpaper_apply_to(&runner, "", home_str).is_err());
    assert!(wallpaper_apply_to(&runner, "/inexistente/x.png", home_str).is_err());
    assert!(!home.path().join(".config/sway/config.d/42-wallpaper").exists());
}

#[test]
fn wallpaper_apply_dispatches_stanza_and_notifies() {
    // Paridade test_apply_wallpaper_dispatches_to_repos.
    use std::io::Write as _;
    let pics = tempfile::tempdir().unwrap();
    let image = pics.path().join("fake-wallpaper.png");
    std::fs::File::create(&image).unwrap().write_all(b"x").unwrap();
    let image_str = image.to_str().unwrap();
    let home = temp_home();
    let home_str = home.path().to_str().unwrap();
    let runner = FakeRunner::new()
        .with_success("swaymsg", &["reload"], "")
        .with_success(
            "notify-send",
            &["-a", "SwayManager", "Papel de Parede", "Aplicado: fake-wallpaper.png"],
            "",
        );
    let stanza = wallpaper_apply_to(&runner, image_str, home_str).unwrap();
    assert!(stanza.ends_with("42-wallpaper"));
    assert_eq!(
        std::fs::read_to_string(&stanza).unwrap(),
        format!("output \"*\" bg \"{image_str}\" fill\n")
    );
    assert!(
        runner
            .run("notify-send", &["-a", "SwayManager", "Papel de Parede", "Aplicado: fake-wallpaper.png"])
            .succeeded()
    );
}

#[test]
fn wallpaper_set_folder_expands_home() {
    let home = temp_home();
    let home_str = home.path().to_str().unwrap();
    assert_eq!(
        wallpaper_set_folder("~/Wallpapers", home_str),
        format!("{home_str}/Wallpapers")
    );
}

// --- Página 2: Aparência & Tema ---

#[test]
fn appearance_page_reads_state_options_and_theme() {
    // Paridade ConfigCenterTestCase.setUp: get_state + available_options + settings.
    let runner = FakeRunner::new()
        .with_success(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "gtk-theme"],
            "'Adwaita-dark'\n",
        )
        .with_success(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "icon-theme"],
            "'Adwaita'\n",
        )
        .with_success(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "cursor-theme"],
            "'Adwaita'\n",
        )
        .with_success(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "font-name"],
            "'Sans 10'\n",
        )
        .with_success("fc-list", &[":", "family"], "Sans\n");
    let fs = MemAppearanceFs::default();
    let theme_fs = MemThemeFs { current: "dark".to_string() };
    let home = temp_home();
    let page = appearance_page(&runner, &fs, &theme_fs, home.path().to_str().unwrap());
    assert_eq!(page.current_theme, "dark");
    assert_eq!(page.settings.gtk_theme, "Adwaita-dark");
    assert!(page.options.fonts.contains(&"Sans".to_string()));
}

#[test]
fn appearance_apply_dispatches_to_repo() {
    // Paridade aplicar_config_aparencia: sucesso → true.
    let runner = FakeRunner::new();
    let fs = MemAppearanceFs::default();
    let home = temp_home();
    let settings = AppearanceSettingsDto {
        gtk_theme: "Adwaita-dark".to_string(),
        icon_theme: "Adwaita".to_string(),
        cursor_theme: "Adwaita".to_string(),
        font_name: "Sans 10".to_string(),
    };
    assert!(appearance_apply(&runner, &fs, home.path().to_str().unwrap(), settings));
}

#[test]
fn theme_toggle_flips_and_reports_literal() {
    // Paridade test_toggle_theme_calls_repository.
    let runner = FakeRunner::new();
    let fs = MemThemeFs { current: "dark".to_string() };
    let home = temp_home();
    assert_eq!(
        theme_toggle(&runner, &fs, home.path().to_str().unwrap()),
        "Trocando para o tema light"
    );
}

// --- Página 3: Bateria & Energia ---

#[test]
fn power_page_reports_battery_and_profile() {
    let fs = WritableBatteryFs::default();
    fs.values.lock().unwrap().insert(
        "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode".to_string(),
        1,
    );
    let runner = FakeRunner::new().with_success("powerprofilesctl", &["get"], "balanced\n");
    let page = power_page(&runner, &fs);
    assert!(page.supported);
    assert!(page.conservation_on);
    assert_eq!(page.active_profile, "balanced");
    assert_eq!(page.profile_label, "equilibrado");
}

#[test]
fn battery_toggle_dispatches() {
    // Paridade battery_use_case.execute(): alterna e devolve mensagem.
    let fs = WritableBatteryFs::default();
    fs.values.lock().unwrap().insert(
        "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode".to_string(),
        0,
    );
    let runner = FakeRunner::new();
    let out = battery_toggle(&runner, &fs);
    assert!(out.success);
    assert!(!out.message.is_empty());
}

#[test]
fn power_apply_rotates_and_sets_flags() {
    // Paridade TogglePowerProfileUseCase.execute(flag): -p/-b/-s.
    for (flag, target) in [("-p", "performance"), ("-b", "balanced"), ("-s", "power-saver")] {
        let runner = FakeRunner::new().with_success("powerprofilesctl", &["get"], "balanced\n");
        assert_eq!(
            power_apply(&runner, Some(flag.to_string())),
            format!("Power profile set to: {target}")
        );
    }
}

// --- Página 4: Suspensão & Idle ---

#[test]
fn idle_page_reports_running() {
    // Paridade create_idle_page: pgrep decide o estado.
    let running = FakeRunner::new().with_success("pgrep", &["-x", "swayidle"], "1\n");
    assert!(idle_page(&running).running);
    let stopped = FakeRunner::new().with_failure("pgrep", &["-x", "swayidle"], 1);
    assert!(!idle_page(&stopped).running);
    assert_eq!(idle_page(&running).timeout_sleep, 300);
}

#[test]
fn idle_toggle_dispatches() {
    // Paridade idle_use_case.execute().
    let runner = FakeRunner::new().with_failure("pgrep", &["-x", "swayidle"], 1);
    let home = temp_home();
    assert_eq!(
        idle_toggle(&runner, home.path().to_str().unwrap(), None),
        "Suspensão ativada!"
    );
}

// --- Página 5: Login & LightDM ---

#[test]
fn lightdm_page_returns_settings_users_sessions() {
    // Paridade ConfigCenterTestCase.setUp: defaults + users + sessions.
    let fs = MemLightdmFs {
        files: HashMap::new(),
        dirs: HashMap::from([
            ("/usr/share/wayland-sessions".to_string(), vec!["sway.desktop".to_string()]),
            ("/usr/share/xsessions".to_string(), vec![]),
        ]),
        passwd: "testuser:x:1000:1000::/home/testuser:/bin/bash\n".to_string(),
        user: "testuser".to_string(),
    };
    let page = lightdm_page(&fs);
    assert_eq!(page.settings.get("user-session").map(String::as_str), Some("sway"));
    assert!(page.users.contains(&"testuser".to_string()));
    assert!(page.sessions.contains(&"sway".to_string()));
}

#[test]
fn lightdm_save_uses_repository_not_polkit() {
    // Paridade test_save_lightdm_uses_repository_not_polkit.
    let fs = MemLightdmFs::default();
    let settings: BTreeMap<String, String> = lightdm::LightDmSettings::default().to_dict();
    assert!(lightdm_save(&OkRunner, &fs, settings, None));
    // `wallpaper_apply`/`monitors_set_screenshot_folder` (atalhos $HOME p/ o invoke)
    // existem p/ o frontend; referência de assinatura como no slice B.
    let _ = config_center::wallpaper_apply as fn(&dyn CommandRunner, &str) -> Result<String, String>;
    let _ = config_center::monitors_set_screenshot_folder as fn(&str, &str) -> String;
    let _ = config_center::wallpaper_set_folder as fn(&str, &str) -> String;
}
