//! Testes MonitorSwap + WallpaperPicker — oráculo: `tests/test_gui_windows.py`
//! (`MonitorSwapTestCase` + `WallpaperPickerTestCase`).
//!
//! `#[path]` próprio p/ compilar com ou sem `pub mod mon_wall` em `lib.rs`
//! (registro pedido ao Main via `agent://all`).
#[path = "../src/mon_wall.rs"]
mod mon_wall;
use std::io::Write as _;
use sway_manager_daemon::process::{CommandRunner, FakeRunner};
use mon_wall::{
    DisplayMode, MonitorLayouts, monitor_apply, monitor_layouts, monitor_reload,
    wallpaper_apply, wallpaper_apply_to, wallpaper_default_folder, wallpaper_list, wallpaper_list_in,
};

const GREP_ARGS: [&str; 2] = ["-c", "swaymsg -t get_outputs | grep \"\\\"name\\\":\""];
const TWO_MONITORS: &str = "\"name\": \"HDMI-A-1\"\n\"name\": \"eDP-1\"\n";

fn two_monitor_runner() -> FakeRunner {
    FakeRunner::new().with_success("sh", &GREP_ARGS, TWO_MONITORS)
}

fn json_outputs() -> &'static str {
    r#"[{"name": "eDP-1", "active": true}, {"name": "HDMI-A-1", "active": true}]"#
}

// --- MonitorSwap (espelha MonitorSwapTestCase) ---

#[test]
fn layouts_reports_names_count_and_current() {
    // Paridade test_shows_four_mode_buttons + initUI: nomes p/ rotular botões.
    let runner = two_monitor_runner()
        .with_success("swaymsg", &["-t", "get_outputs"], json_outputs())
        .with_failure("pgrep", &["wl-mirror"], 1);
    let layouts: MonitorLayouts = monitor_layouts(&runner);
    assert_eq!(layouts.interno.as_deref(), Some("eDP-1"));
    assert_eq!(layouts.externo.as_deref(), Some("HDMI-A-1"));
    assert_eq!(layouts.count, 2);
    assert_eq!(layouts.current, DisplayMode::Extend);
}

#[test]
fn layouts_without_two_monitors_yields_none() {
    // Paridade ValueError("Apenas um monitor encontrado!") absorvido aqui.
    let runner = FakeRunner::new()
        .with_success("sh", &GREP_ARGS, "\"name\": \"eDP-1\"\n")
        .with_failure("swaymsg", &["-t", "get_outputs"], 1);
    let layouts = monitor_layouts(&runner);
    assert_eq!(layouts.interno, None);
    assert_eq!(layouts.externo, None);
    assert_eq!(layouts.count, 1);
}

#[test]
fn all_four_modes_dispatch_to_apply_config() {
    // Paridade test_all_four_modes_dispatch_to_use_case.
    for (mode, last) in [
        (DisplayMode::PcOnly, "disable"),
        (DisplayMode::MonitorOnly, "enable"),
        (DisplayMode::Extend, "enable"),
        (DisplayMode::Duplicate, "enable"),
    ] {
        let runner = two_monitor_runner().with_success(
            "swaymsg",
            &[
                "output",
                "eDP-1",
                if mode == DisplayMode::PcOnly {
                    "disable,"
                } else {
                    "enable,"
                },
                "output",
                "HDMI-A-1",
                last,
            ],
            "",
        );
        monitor_apply(&runner, mode).unwrap();
        if mode == DisplayMode::Duplicate {
            assert_eq!(
                runner.spawned(),
                vec![vec!["wl-mirror".to_string(), "eDP-1".to_string()]]
            );
        } else {
            assert!(runner.spawned().is_empty(), "{mode:?}");
        }
    }
}

#[test]
fn apply_error_propagates_as_string() {
    // Paridade test_use_case_error_shows_message_box: Err vira mensagem no frontend.
    let runner =
        FakeRunner::new().with_success("sh", &GREP_ARGS, "\"name\": \"eDP-1\"\n");
    assert_eq!(
        monitor_apply(&runner, DisplayMode::Extend),
        Err("Apenas um monitor encontrado!".to_string())
    );
}

#[test]
fn reload_runs_swaymsg_reload() {
    // Paridade confirmar_ou_reverter → recarregar_sway.
    let runner = FakeRunner::new().with_success("swaymsg", &["reload"], "");
    monitor_reload(&runner);
    assert!(runner.run("swaymsg", &["reload"]).succeeded());
}

// --- WallpaperPicker (espelha WallpaperPickerTestCase) ---

fn write_png(dir: &std::path::Path, name: &str) {
    // PNG mínimo válido (1x1): basta existir p/ listagem por extensão.
    let bytes = [
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A,
    ];
    let mut fh = std::fs::File::create(dir.join(name)).unwrap();
    fh.write_all(&bytes).unwrap();
}

fn wallpaper_dir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_png(tmp.path(), "a.png");
    write_png(tmp.path(), "b.png");
    std::fs::write(tmp.path().join("notes.txt"), "not an image").unwrap();
    tmp
}

#[test]
fn grid_loads_images_and_ignores_non_images() {
    // Paridade test_grid_loads_fake_pngs_and_ignores_non_images.
    let tmp = wallpaper_dir();
    let cache = tempfile::tempdir().unwrap();
    let dir = tmp.path().to_str().unwrap();
    let listed = wallpaper_list_in(dir, cache.path());
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|e| e.path.ends_with(".png")));
    assert!(listed.iter().all(|e| e.path.starts_with(dir)));
    assert!(listed[0].path < listed[1].path, "ordenado como CarregadorDeImagens");
    assert!(listed.iter().all(|e| e.thumb.starts_with("data:image/")));
}

#[test]
fn loader_with_missing_dir_returns_empty() {
    // Paridade test_loader_run_with_missing_dir_does_not_crash.
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("nope").to_str().unwrap().to_string();
    assert!(wallpaper_list(&missing).is_empty());
}

#[test]
fn apply_writes_stanza_and_returns_path() {
    // Paridade test_apply_calls_use_case_with_item_path.
    let pics = wallpaper_dir();
    let home = tempfile::tempdir().unwrap();
    let runner = FakeRunner::new().with_success("swaymsg", &["reload"], "");
    let image = pics.path().join("a.png").to_str().unwrap().to_string();
    let home_str = home.path().to_str().unwrap();
    let stanza = wallpaper_apply_to(&runner, &image, home_str).unwrap();
    assert!(stanza.ends_with("42-wallpaper"));
    assert_eq!(
        std::fs::read_to_string(&stanza).unwrap(),
        format!("output \"*\" bg \"{image}\" fill\n")
    );
}

#[test]
fn apply_without_path_is_noop() {
    // Paridade test_apply_without_path_is_noop.
    let home = tempfile::tempdir().unwrap();
    let runner = FakeRunner::new();
    let home_str = home.path().to_str().unwrap();
    assert!(wallpaper_apply_to(&runner, "", home_str).is_err());
    assert!(
        wallpaper_apply_to(&runner, "/inexistente/x.png", home_str).is_err(),
        "nada é escrito sem path válido"
    );
    assert!(
        !home
            .path()
            .join(".config/sway/config.d/42-wallpaper")
            .exists()
    );
}

#[test]
fn default_folder_prefers_existing_candidate() {
    // Paridade SetWallpaperUseCase.get_wallpaper_folder: primeira candidata
    // existente (`~/Imagens/Wallpapers` … `~/Imagens`), senão `~`.
    let home = tempfile::tempdir().unwrap();
    let home_str = home.path().to_str().unwrap();
    assert_eq!(wallpaper_default_folder(home_str), home_str);
    std::fs::create_dir_all(home.path().join("Imagens/Wallpapers")).unwrap();
    assert_eq!(
        wallpaper_default_folder(home_str),
        home.path().join("Imagens/Wallpapers").to_str().unwrap()
    );
}

#[test]
fn apply_sends_normal_notify() {
    // Paridade notify do apply_appearance: `notify-send -a SwayManager`.
    let pics = wallpaper_dir();
    let home = tempfile::tempdir().unwrap();
    let home_str = home.path().to_str().unwrap();
    let image = pics.path().join("b.png").to_str().unwrap().to_string();
    let message = format!("Papel de parede aplicado!\n{image}");
    let runner = FakeRunner::new()
        .with_success("swaymsg", &["reload"], "")
        .with_success(
            "notify-send",
            &["-a", "SwayManager", "SwayManager", &message, "normal"],
            "",
        );
    wallpaper_apply_to(&runner, &image, home_str).unwrap();
    assert!(
        runner
            .run("notify-send", &["-a", "SwayManager", "SwayManager", &message, "normal"])
            .succeeded()
    );
    // `wallpaper_apply` (atalho $HOME) existe p/ o invoke do frontend.
    let _ = wallpaper_apply as fn(&dyn CommandRunner, &str) -> Result<String, String>;
}
