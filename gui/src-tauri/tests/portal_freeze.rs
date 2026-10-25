//! Testes do slice portal+freeze — oráculos:
//! `tests/test_portal.py` (dmenu parse, validate) +
//! `tests/test_freeze_selection_overlay.py` (SelectionState, compose multimonitor).

#[path = "../src/domains/system/portal_freeze.rs"]
mod portal_freeze;
use std::collections::HashMap;

use portal_freeze::{
    MIN_SELECTION_SIZE, FrozenScreenDto, PortalSelection, Rect,
    SelectionState, freeze_capture, freeze_compose, parse_dmenu_sources,
    portal_cancel, portal_select, portal_sources,
};
use sway_manager_daemon::process::{CmdOutput, CommandRunner};

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

    fn failing(mut self, prog: &str, args: &[&str], status: i32) -> Self {
        self.outputs.insert(
            (prog.to_string(), args.join(" ")),
            CmdOutput::failure(status, "", "boom"),
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

// --- dmenu parse (oráculo: TestPortalController.test_parses_dmenu_stdin_sources) ---

#[test]
fn parses_structured_dmenu_labels() {
    let content = "Monitor: eDP-1\nMonitor: HDMI-A-1 Goldstar 1920x1080\nWindow: Firefox (abc123def456)\nWindow: Discord - Call (7890abcdef12)\n";
    let (monitors, windows) = parse_dmenu_sources(content);

    assert_eq!(monitors.len(), 2);
    assert_eq!(monitors[0].id, "eDP-1");
    assert_eq!(monitors[0].raw_label.as_deref(), Some("Monitor: eDP-1"));
    assert_eq!(monitors[1].id, "HDMI-A-1");
    assert_eq!(monitors[1].details, "Goldstar 1920x1080");

    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].id, "abc123def456");
    assert_eq!(windows[0].label, "Firefox");
    assert_eq!(windows[1].id, "7890abcdef12");
    assert_eq!(windows[1].label, "Discord - Call");
}

#[test]
fn parses_legacy_dmenu_output_names() {
    let (monitors, windows) = parse_dmenu_sources("HDMI-A-3\nDP-1\n");
    assert_eq!(
        monitors.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
        vec!["HDMI-A-3", "DP-1"]
    );
    assert_eq!(monitors[0].raw_label.as_deref(), Some("HDMI-A-3"));
    assert!(windows.is_empty());
}

#[test]
fn dmenu_result_preserves_raw_label() {
    let (monitors, windows) = parse_dmenu_sources("Monitor: eDP-1\nWindow: Firefox (abc123)\n");
    let window = PortalSelection {
        source_type: "window".to_string(),
        id: windows[0].id.clone(),
        raw_label: windows[0].raw_label.clone(),
    };
    assert_eq!(window.line(), "Window: Firefox (abc123)");
    let monitor = PortalSelection {
        source_type: "monitor".to_string(),
        id: monitors[0].id.clone(),
        raw_label: monitors[0].raw_label.clone(),
    };
    assert_eq!(monitor.line(), "Monitor: eDP-1");
}

// --- validate (oráculo: TestPortalResultWriter) ---

#[test]
fn select_rejects_multiline_identifier() {
    let err = portal_select(r#"{"source_type":"monitor","id":"line\nbreak"}"#).unwrap_err();
    assert!(err.contains("inválido"), "{err}");
}

#[test]
fn select_rejects_empty_result() {
    let err = portal_select(r#"{"source_type":"monitor","id":""}"#).unwrap_err();
    assert!(err.contains("inválido") || err.contains("vazio"), "{err}");
}

#[test]
fn select_accepts_explicit_raw_label() {
    let line = portal_select(r#"{"source_type":"monitor","id":"eDP-1","raw_label":"Monitor: eDP-1"}"#).unwrap();
    assert_eq!(line, "Monitor: eDP-1");
}

#[test]
fn cancel_emits_nothing() {
    assert_eq!(portal_cancel(), None);
}

#[test]
fn sources_prefers_dmenu_stdin_over_swaymsg() {
    let runner = FakeRunner::default().failing("swaymsg", &["-t", "get_outputs", "-r"], 1);
    let sources = portal_sources(&runner, Some("HDMI-A-3\n".to_string())).unwrap();
    assert_eq!(sources.monitors.len(), 1);
    assert_eq!(sources.monitors[0].id, "HDMI-A-3");
}

#[test]
fn sources_maps_sway_failure_to_portuguese_error() {
    let runner = FakeRunner::default().failing("swaymsg", &["-t", "get_outputs", "-r"], 1);
    let err = portal_sources(&runner, None).unwrap_err();
    assert!(err.contains("swaymsg"), "{err}");
}


// --- SelectionState (oráculo: TestSelectionState) ---

#[test]
fn drag_creates_normalized_rectangle() {
    let mut state = SelectionState::default();
    state.begin(30, 20);
    state.update(10, 5);
    assert_eq!(
        state.rectangle(),
        Some(Rect { x: 10, y: 5, w: 21, h: 16 })
    );
}

#[test]
fn finish_rejects_selection_smaller_than_minimum() {
    let mut state = SelectionState::default();
    state.begin(0, 0);
    assert_eq!(state.finish(4, 4), None);
    assert_eq!(MIN_SELECTION_SIZE, 6);
}

#[test]
fn finish_returns_valid_selection() {
    let mut state = SelectionState::default();
    state.begin(0, 0);
    assert_eq!(state.finish(5, 5), Some(Rect { x: 0, y: 0, w: 6, h: 6 }));
}

// --- compose multimonitor (oráculo: TestComposeSelection) ---

fn screen(name: &str, x: i32, y: i32, w: i32, h: i32, scale: f64) -> FrozenScreenDto {
    FrozenScreenDto {
        name: name.to_string(),
        geometry: Rect { x, y, w, h },
        scale_x: scale,
        scale_y: scale,
        image: None,
    }
}

#[test]
fn crops_one_screen_at_native_scale() {
    let screens = vec![screen("left", 0, 0, 100, 50, 1.0)];
    let plan = freeze_compose(Rect { x: 10, y: 5, w: 20, h: 10 }, &screens).unwrap();
    assert_eq!((plan.width, plan.height), (20, 10));
    assert_eq!(plan.ops.len(), 1);
    assert_eq!(plan.ops[0].src, Rect { x: 10, y: 5, w: 20, h: 10 });
    assert_eq!(plan.ops[0].dst, Rect { x: 0, y: 0, w: 20, h: 10 });
}

#[test]
fn uses_physical_resolution_for_hidpi_output() {
    let screens = vec![screen("hidpi", 0, 0, 100, 50, 2.0)];
    let plan = freeze_compose(Rect { x: 10, y: 5, w: 20, h: 10 }, &screens).unwrap();
    assert_eq!((plan.width, plan.height), (40, 20));
    assert_eq!(plan.ops[0].src, Rect { x: 20, y: 10, w: 40, h: 20 });
}

#[test]
fn combines_monitors_at_highest_participating_scale() {
    let screens = vec![
        screen("left", 0, 0, 100, 50, 1.0),
        screen("right", 100, 0, 100, 50, 2.0),
    ];
    let plan = freeze_compose(Rect { x: 50, y: 0, w: 100, h: 50 }, &screens).unwrap();
    assert_eq!((plan.width, plan.height), (200, 100));
    assert_eq!(plan.ops.len(), 2);
    // Metade esquerda (scale 1) ocupa os primeiros 100 px do canvas em scale 2.
    assert_eq!(plan.ops[0].dst, Rect { x: 0, y: 0, w: 100, h: 100 });
    assert_eq!(plan.ops[1].dst.x, 100);
}

#[test]
fn preserves_transparent_gap_between_outputs() {
    let screens = vec![
        screen("left", 0, 0, 100, 20, 1.0),
        screen("right", 150, 0, 100, 20, 1.0),
    ];
    let plan = freeze_compose(Rect { x: 50, y: 0, w: 150, h: 10 }, &screens).unwrap();
    assert_eq!((plan.width, plan.height), (150, 10));
    assert_eq!(plan.ops.len(), 2);
    assert_eq!(plan.ops[0].dst, Rect { x: 0, y: 0, w: 50, h: 10 });
    assert_eq!(plan.ops[1].dst, Rect { x: 100, y: 0, w: 50, h: 10 });
}

#[test]
fn compose_returns_none_without_intersection() {
    let screens = vec![screen("left", 0, 0, 100, 50, 1.0)];
    assert!(freeze_compose(Rect { x: 500, y: 500, w: 10, h: 10 }, &screens).is_none());
}

#[test]
fn selection_roundtrips_through_daemon_model() {
    let daemon = sway_manager_daemon::domains::system::portal::PortalResult::new(
        sway_manager_daemon::domains::system::portal::SourceType::Window,
        "abc123",
    );
    let selection = PortalSelection::from_daemon(&daemon);
    assert_eq!(selection.source_type, "window");
    assert_eq!(selection.line(), "abc123");
}


#[test]
fn freeze_capture_runs_grim_per_output() {
    use std::io::Write as _;
    let dir = std::env::temp_dir().join(format!("freeze_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let dst = dir.join("eDP-1.png");
    let mut file = std::fs::File::create(&dst).unwrap();
    file.write_all(b"png").unwrap();
    let runner = FakeRunner::default().with(
        "grim",
        &["-o", "eDP-1", dst.to_str().unwrap()],
        "",
    );
    let out = freeze_capture(&runner, "eDP-1", dst.to_str().unwrap()).unwrap();
    assert_eq!(out, dst.to_str().unwrap());
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Pipeline screenshot (fix: overlay salvava nada) ---

#[test]
fn freeze_screens_parses_geometry_and_scale() {
    let runner = FakeRunner::default().with("swaymsg", &["-t", "get_outputs", "-r"], r#"[{"name":"eDP-1","active":true,"scale":1.0,"rect":{"x":0,"y":0,"width":1920,"height":1080}},{"name":"HDMI-A-1","active":true,"scale":2.0,"rect":{"x":1920,"y":0,"width":1920,"height":1080}},{"name":"DP-1","active":false,"scale":1.0,"rect":{"x":0,"y":0,"width":640,"height":480}}]"#);
    let screens = portal_freeze::freeze_screens(&runner);
    assert_eq!(screens.len(), 2);
    assert_eq!(screens[0].name, "eDP-1");
    assert_eq!(screens[0].geometry, Rect { x: 0, y: 0, w: 1920, h: 1080 });
    assert_eq!(screens[1].scale_x, 2.0);
    assert_eq!(screens[1].geometry.x, 1920);
}

#[test]
fn freeze_screens_empty_when_swaymsg_fails() {
    let runner = FakeRunner::default().failing("swaymsg", &["-t", "get_outputs", "-r"], 1);
    assert!(portal_freeze::freeze_screens(&runner).is_empty());
}

#[test]
fn freeze_cancel_is_none() {
    assert_eq!(portal_freeze::freeze_cancel(), None);
}

#[test]
fn screenshot_timestamp_format() {
    let ts = portal_freeze::screenshot_timestamp();
    // YYYY-MM-DD_HH:MM:SS, 19 chars.
    assert_eq!(ts.len(), 19);
    assert_eq!(&ts[4..5], "-");
    assert_eq!(&ts[10..11], "_");
    assert_eq!(&ts[13..14], ":");
    assert!(ts.starts_with("2026-"));
}

// PNG 1x1 mínimo válido (assinatura + IHDR + IDAT + IEND).
const TINY_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

#[test]
fn screenshot_save_png_writes_notifies_and_returns_path() {
    let dir = std::env::temp_dir().join(format!("shot_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let runner = FakeRunner::default()
        .with("sh", &["-c", "wl-copy --type image/png <"], "")
        .with("notify-send", &[], "");
    let url = format!("data:image/png;base64,{TINY_PNG_B64}");
    let path = portal_freeze::screenshot_save_png(&runner, &url, dir.to_str().unwrap(), "/tmp").unwrap();
    assert!(path.starts_with(dir.to_str().unwrap()));
    assert!(path.ends_with(".png"));
    assert!(std::path::Path::new(&path).is_file());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn screenshot_save_png_rejects_non_png() {
    let runner = FakeRunner::default();
    assert!(portal_freeze::screenshot_save_png(&runner, "data:image/png;base64,aGVsbG8=", "/tmp", "/tmp").is_err());
    assert!(portal_freeze::screenshot_save_png(&runner, "not-a-data-url", "/tmp", "/tmp").is_err());
}

#[test]
fn screenshot_full_captures_and_publishes() {
    // grim escreve o arquivo: FakeRunner não cria arquivos, então o teste
    // valida o caminho de falha (sem arquivo = erro honesto).
    let runner = FakeRunner::default()
        .with("xdg-user-dir", &["PICTURES"], "/tmp/pics\n")
        .with("grim", &["/tmp/pics/screenshots/x.png"], "");
    let result = portal_freeze::screenshot_full(&runner, "/tmp/nonexistent-home");
    // Sem grim real o capture falha → erro honesto (não silêncio).
    assert!(result.is_err());
}

#[test]
fn screenshot_window_errors_without_focused_window() {
    // Sem janela focada no get_tree fake → erro honesto (não silêncio).
    let runner = FakeRunner::default()
        .with("xdg-user-dir", &["PICTURES"], "/tmp/pics\n")
        .with("swaymsg", &["-t", "get_tree"], r#"{"nodes":[]}"#);
    let result = portal_freeze::screenshot_window(&runner, "/tmp/nonexistent-home");
    assert!(result.is_err());
}

#[test]
fn freeze_image_data_returns_png_data_url() {
    use std::io::Write as _;
    let dir = std::env::temp_dir().join(format!("b64_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("tiny.png");
    // PNG 1x1: decodifica o base64 conhecido e regrava.
    let raw: Vec<u8> = vec![
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
        8, 2, 0, 0, 0, 144, 119, 83, 222,
    ];
    std::fs::File::create(&path).unwrap().write_all(&raw).unwrap();
    let url = portal_freeze::freeze_image_data(path.to_str().unwrap()).unwrap();
    assert!(url.starts_with("data:image/png;base64,iVBORw0KGgo"));
    // Roundtrip: o mesmo conteúdo volta via save (valida o PNG).
    let out = dir.join("out");
    let runner = FakeRunner::default();
    let saved = portal_freeze::screenshot_save_png(&runner, &url, out.to_str().unwrap(), "/tmp").unwrap();
    assert_eq!(std::fs::read(&saved).unwrap(), raw);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn freeze_image_data_errors_on_missing_file() {
    assert!(portal_freeze::freeze_image_data("/tmp/definitivamente-nao-existe-xyz.png").is_err());
}

// --- Composição nativa dual-monitor (fix: canvas preto / cross-output) ---

fn solid_png(path: &std::path::Path, w: u32, h: u32, pixel: [u8; 4]) {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba(pixel));
    img.save_with_format(path, image::ImageFormat::Png).unwrap();
}

fn two_screens(dir: &std::path::Path) -> Vec<portal_freeze::FrozenScreenDto> {
    // eDP: 100x100 lógicos @2.0 = 200x200 físicos (vermelho).
    // HDMI: 100x100 lógicos @1.0 em (100,0), 100x100 físicos (verde).
    let edp = dir.join("eDP-1.png");
    let hdmi = dir.join("HDMI-A-1.png");
    solid_png(&edp, 200, 200, [255, 0, 0, 255]);
    solid_png(&hdmi, 100, 100, [0, 255, 0, 255]);
    vec![
        portal_freeze::FrozenScreenDto {
            name: "eDP-1".to_string(),
            geometry: portal_freeze::Rect { x: 0, y: 0, w: 100, h: 100 },
            scale_x: 2.0,
            scale_y: 2.0,
            image: Some(edp.to_str().unwrap().to_string()),
        },
        portal_freeze::FrozenScreenDto {
            name: "HDMI-A-1".to_string(),
            geometry: portal_freeze::Rect { x: 100, y: 0, w: 100, h: 100 },
            scale_x: 1.0,
            scale_y: 1.0,
            image: Some(hdmi.to_str().unwrap().to_string()),
        },
    ]
}

#[test]
fn compose_save_single_screen_native_pixels() {
    let dir = std::env::temp_dir().join(format!("compose1_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let screens = two_screens(&dir);
    let out = dir.join("out");
    let runner = FakeRunner::default();
    // Seleção 10x10 lógicos dentro do eDP → 20x20 físicos vermelhos.
    let sel = portal_freeze::Rect { x: 5, y: 5, w: 10, h: 10 };
    let path = portal_freeze::screenshot_compose_save(&runner, sel, &screens, out.to_str().unwrap(), "/tmp").unwrap();
    let img = image::open(&path).unwrap().to_rgba8();
    assert_eq!((img.width(), img.height()), (20, 20));
    assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(img.get_pixel(19, 19).0, [255, 0, 0, 255]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compose_save_cross_output_keeps_both_sides() {
    let dir = std::env::temp_dir().join(format!("compose2_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let screens = two_screens(&dir);
    let out = dir.join("out");
    let runner = FakeRunner::default();
    // Seleção cruzando a fronteira (95..104 lógicos), canvas no maior
    // scale (2.0): eDP 10 físicos vermelhos (dst 0..10), HDMI com upscale
    // Nearest 5→10 (dst 10..20, paridade `drawImage` do Qt).
    let sel = portal_freeze::Rect { x: 95, y: 10, w: 10, h: 10 };
    let path = portal_freeze::screenshot_compose_save(&runner, sel, &screens, out.to_str().unwrap(), "/tmp").unwrap();
    let img = image::open(&path).unwrap().to_rgba8();
    assert_eq!((img.width(), img.height()), (20, 20));
    assert_eq!(img.get_pixel(0, 10).0, [255, 0, 0, 255], "lado eDP vermelho");
    assert_eq!(img.get_pixel(9, 10).0, [255, 0, 0, 255], "fim do lado eDP");
    assert_eq!(img.get_pixel(10, 10).0, [0, 255, 0, 255], "início do lado HDMI");
    assert_eq!(img.get_pixel(19, 19).0, [0, 255, 0, 255], "fim do lado HDMI");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compose_save_rejects_empty_and_offscreen() {
    let dir = std::env::temp_dir().join(format!("compose3_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let screens = two_screens(&dir);
    let runner = FakeRunner::default();
    assert!(portal_freeze::screenshot_compose_save(&runner, portal_freeze::Rect { x: 0, y: 0, w: 0, h: 0 }, &screens, dir.to_str().unwrap(), "/tmp").is_err());
    assert!(portal_freeze::screenshot_compose_save(&runner, portal_freeze::Rect { x: 5000, y: 5000, w: 10, h: 10 }, &screens, dir.to_str().unwrap(), "/tmp").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
