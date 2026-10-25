//! `screenshot` capture paths + compositor argv.
//!
//! Mirrors `GrimSlurpScreenshotRepository`
//! (`src/infrastructure/media/grim_slurp_screenshot_repository.py`).
//! The pixel work (Qt freeze overlay, `QImage` crop) stays in Python.
//! This port owns the headless contract: `ScreenshotMode` mapping from
//! `handle_screenshot` (`handlers.py:137-148`), the `slurp` geometry
//! parser (`_parse_geometry`), the focused-window lookup
//! (`_find_focused_rect` over `swaymsg -t get_tree`), the `grim`
//! full/per-output/region argv (`take_screenshot`), the config + `xdg-user-dir
//! PICTURES` folder resolution (`get_screenshot_folder`), and the `wl-copy`
//! png + `notify-send` publish step.

use crate::process::{CommandRunner, notify};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenshotMode {
    Full,
    Area,
    Window,
}

/// Mirrors `handle_screenshot` raw-mode mapping.
pub fn parse_mode(raw: Option<&str>) -> ScreenshotMode {
    match raw.map(str::to_lowercase).as_deref() {
        Some("area") | Some("-g") => ScreenshotMode::Area,
        Some("window") | Some("-w") => ScreenshotMode::Window,
        _ => ScreenshotMode::Full,
    }
}

pub fn action_desc(mode: ScreenshotMode) -> &'static str {
    match mode {
        ScreenshotMode::Full => "da tela",
        ScreenshotMode::Area => "da área selecionada",
        ScreenshotMode::Window => "da janela",
    }
}

/// Mirrors `_parse_geometry("x,y WxH")`.
pub fn parse_geometry(geom: &str) -> Option<(i32, i32, i32, i32)> {
    let mut parts = geom.split_whitespace();
    let (pos, size) = (parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let mut pos = pos.split(',');
    let mut size = size.split('x');
    let (x, y, w, h) = (
        pos.next()?.parse().ok()?,
        pos.next()?.parse().ok()?,
        size.next()?.parse().ok()?,
        size.next()?.parse().ok()?,
    );
    if pos.next().is_some() || size.next().is_some() {
        return None;
    }
    Some((x, y, w, h))
}

/// Mirrors `_find_focused_rect`: the focused node must carry an
/// `{"x","y","width","height"}` rect; otherwise recurse into
/// `nodes` + `floating_nodes` depth-first, first hit wins.
pub fn focused_rect(node: &serde_json::Value) -> Option<(i32, i32, i32, i32)> {
    if node.get("focused").and_then(|f| f.as_bool()).unwrap_or(false) {
        let rect = node.get("rect")?;
        let (x, y, w, h) = (
            rect.get("x")?.as_i64()?,
            rect.get("y")?.as_i64()?,
            rect.get("width")?.as_i64()?,
            rect.get("height")?.as_i64()?,
        );
        return Some((x as i32, y as i32, w as i32, h as i32));
    }
    for key in ["nodes", "floating_nodes"] {
        if let Some(children) = node.get(key).and_then(|c| c.as_array()) {
            for child in children {
                if let Some(hit) = focused_rect(child) {
                    return Some(hit);
                }
            }
        }
    }
    None
}

/// Mirrors `_get_focused_window_rect`: `swaymsg -t get_tree` then
/// `_find_focused_rect`; `None` on any failure.
pub fn focused_window_rect(runner: &dyn CommandRunner) -> Option<(i32, i32, i32, i32)> {
    let out = runner.run("swaymsg", &["-t", "get_tree"]);
    if !out.succeeded() || out.stdout.trim().is_empty() {
        return None;
    }
    let tree: serde_json::Value = serde_json::from_str(&out.stdout).ok()?;
    focused_rect(&tree)
}

pub fn grim_region_args(geometry: (i32, i32, i32, i32), dst: &str) -> Vec<String> {
    let (x, y, w, h) = geometry;
    vec![
        "grim".to_string(),
        "-g".to_string(),
        format!("{x},{y} {w}x{h}"),
        dst.to_string(),
    ]
}

/// Run `slurp` then `grim -g` (the `_take_slurp_selection` fallback).
/// Returns `true` on success; prints the Python literal otherwise.
pub fn take_slurp_selection(runner: &dyn CommandRunner, dst: &str) -> bool {
    let slurp = runner.run("slurp", &[]);
    if !slurp.succeeded() || slurp.stdout.trim().is_empty() {
        println!("Screenshot cancelled or failed.");
        return false;
    }
    let Some(geometry) = parse_geometry(&slurp.stdout) else {
        println!("Screenshot cancelled or failed.");
        return false;
    };
    let (x, y, w, h) = geometry;
    let grim = runner.run("grim", &["-g", &format!("{x},{y} {w}x{h}"), dst]);
    if grim.succeeded() {
        true
    } else {
        println!("Screenshot cancelled or failed.");
        false
    }
}

pub fn screenshot_filename(folder: &str, timestamp: &str) -> String {
    format!("{folder}/screenshot_{timestamp}.png")
}

fn expand_home(path: &str, home: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        path.to_string()
    }
}

/// Mirrors `get_screenshot_folder` (lines 103-123): a saved
/// `screenshot_folder` setting wins when it is an existing dir; otherwise
/// `xdg-user-dir PICTURES` (blank/failure falls back to `~/Pictures`) plus
/// `/screenshots`.
pub fn resolve_screenshot_folder(
    runner: &dyn CommandRunner,
    saved: Option<&str>,
    home: &str,
    is_dir: &dyn Fn(&str) -> bool,
) -> String {
    if let Some(folder) = saved {
        let expanded = expand_home(folder, home);
        if is_dir(&expanded) {
            return expanded;
        }
    }
    let mut pictures = String::new();
    let probe = runner.run("xdg-user-dir", &["PICTURES"]);
    if probe.succeeded() {
        pictures = probe.stdout.trim().to_string();
    }
    if pictures.is_empty() {
        pictures = format!("{home}/Pictures");
    }
    format!("{pictures}/screenshots")
}

/// Mirrors `set_screenshot_folder` (lines 125-130): `~`-expand the value;
/// callers persist it with `app_config::set_setting(config, "screenshot_folder")`.
pub fn normalize_screenshot_folder(folder: &str, home: &str) -> String {
    expand_home(folder, home)
}

/// Mirrors the full-desktop capture (`take_screenshot` line 145):
/// `grim <dst>`; `true` only when the capture succeeds and the file exists.
pub fn capture_full(runner: &dyn CommandRunner, dst: &str, is_file: &dyn Fn(&str) -> bool) -> bool {
    runner.run("grim", &[dst]).succeeded() && is_file(dst)
}

/// Mirrors the per-output capture (lines 156-159):
/// `grim -o <output> <dst>`; `true` only on success with the file present.
pub fn capture_output(
    runner: &dyn CommandRunner,
    output: &str,
    dst: &str,
    is_file: &dyn Fn(&str) -> bool,
) -> bool {
    runner.run("grim", &["-o", output, dst]).succeeded() && is_file(dst)
}

/// Mirrors the window capture (lines 185-192): focused rect from
/// `swaymsg -t get_tree`, then `grim -g "<x>,<y> <w>x<h>" <dst>`;
/// `false` when no window is focused (the Python fallback saves the full
/// freeze image, which is a Qt pixel op the daemon does not own).
pub fn capture_window(
    runner: &dyn CommandRunner,
    dst: &str,
    is_file: &dyn Fn(&str) -> bool,
) -> bool {
    let Some((x, y, w, h)) = focused_window_rect(runner) else {
        return false;
    };
    let geometry = format!("{x},{y} {w}x{h}");
    runner.run("grim", &["-g", &geometry, dst]).succeeded() && is_file(dst)
}

/// Mirrors the area capture selection step (lines 170-179): `slurp`
/// geometry then `grim -g`; prints the Python literal when cancelled.
pub fn capture_area(runner: &dyn CommandRunner, dst: &str) -> bool {
    take_slurp_selection(runner, dst)
}

/// Mirrors the publish step (lines 199-212): `wl-copy` the png via
/// `sh -c "wl-copy --type image/png < '<file>'"` (Python `shell=True`),
/// `notify-send` the `Captura {desc} salva em {file}` title, and print
/// `Screenshot saved to {file}`; prints the cancelled literal otherwise.
///
/// `wl-copy` fork-daemoniza o dono do clipboard herdando os FDs: com `run`
/// (pipes + wait) o `.output()` nunca vê EOF e trava. Por isso o copy vai
/// via `spawn_detached` (stdio null, sem wait) — o Python herdava o stdio
/// do terminal e nunca travava.
pub fn publish_screenshot(
    runner: &dyn CommandRunner,
    mode: ScreenshotMode,
    filename: &str,
    is_file: &dyn Fn(&str) -> bool,
) -> bool {
    if !is_file(filename) {
        println!("Screenshot cancelled or failed.");
        return false;
    }
    let script = format!("wl-copy --type image/png < '{filename}'");
    runner.spawn_detached("sh", &["-c", &script]);
    let title = format!("Captura {} salva em {filename}", action_desc(mode));
    notify(runner, &title, "", "normal");
    println!("Screenshot saved to {filename}");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    #[test]
    fn mode_mapping_matches_handler() {
        assert_eq!(parse_mode(Some("area")), ScreenshotMode::Area);
        assert_eq!(parse_mode(Some("-g")), ScreenshotMode::Area);
        assert_eq!(parse_mode(Some("window")), ScreenshotMode::Window);
        assert_eq!(parse_mode(Some("-w")), ScreenshotMode::Window);
        assert_eq!(parse_mode(None), ScreenshotMode::Full);
        assert_eq!(parse_mode(Some("full")), ScreenshotMode::Full);
        assert_eq!(action_desc(ScreenshotMode::Area), "da área selecionada");
    }

    #[test]
    fn geometry_parsing() {
        assert_eq!(parse_geometry("10,20 300x400"), Some((10, 20, 300, 400)));
        assert_eq!(parse_geometry("nope"), None);
        assert_eq!(parse_geometry("1,2 3x4 extra"), None);
    }

    #[test]
    fn slurp_fallback_runs_grim_with_geometry() {
        let runner = FakeRunner::new()
            .with_success("slurp", &[], "10,20 300x400\n")
            .with_success("grim", &["-g", "10,20 300x400", "/tmp/out.png"], "");
        assert!(take_slurp_selection(&runner, "/tmp/out.png"));
    }

    #[test]
    fn slurp_cancel_prints_literal() {
        let runner = FakeRunner::new().with_failure("slurp", &[], 1);
        assert!(!take_slurp_selection(&runner, "/tmp/out.png"));
    }

    #[test]
    fn parse_geometry_valid_with_trailing_newline() {
        // Mirrors tests/test_screenshot_repository.py::test_parse_geometry_valid.
        assert_eq!(parse_geometry("100,200 800x600\n"), Some((100, 200, 800, 600)));
    }

    #[test]
    fn parse_geometry_invalid_inputs() {
        // Mirrors tests/test_screenshot_repository.py::test_parse_geometry_invalid.
        assert_eq!(parse_geometry("invalid geometry"), None);
        assert_eq!(parse_geometry(""), None);
    }

    #[test]
    fn find_focused_rect_in_nested_tree() {
        // Mirrors tests/test_screenshot_repository.py::test_find_focused_rect.
        let tree: serde_json::Value = serde_json::from_str(
            r#"{"nodes": [{"focused": false, "nodes": [
                {"focused": true, "rect": {"x": 50, "y": 60, "width": 500, "height": 400}}]}]}"#,
        )
        .unwrap();
        assert_eq!(focused_rect(&tree), Some((50, 60, 500, 400)));
    }

    #[test]
    fn find_focused_rect_not_found() {
        // Mirrors tests/test_screenshot_repository.py::test_find_focused_rect_not_found.
        let tree: serde_json::Value =
            serde_json::from_str(r#"{"nodes": [{"focused": false}]}"#).unwrap();
        assert_eq!(focused_rect(&tree), None);
        let missing_rect: serde_json::Value =
            serde_json::from_str(r#"{"focused": true}"#).unwrap();
        assert_eq!(focused_rect(&missing_rect), None);
    }

    #[test]
    fn window_capture_queries_tree_then_grim() {
        let runner = FakeRunner::new()
            .with_success(
                "swaymsg",
                &["-t", "get_tree"],
                r#"{"nodes": [{"focused": true,
                    "rect": {"x": 1, "y": 2, "width": 30, "height": 40}}]}"#,
            )
            .with_success("grim", &["-g", "1,2 30x40", "/tmp/win.png"], "");
        assert!(capture_window(&runner, "/tmp/win.png", &|_| true));
        let no_focus = FakeRunner::new().with_success(
            "swaymsg",
            &["-t", "get_tree"],
            r#"{"nodes": []}"#,
        );
        assert!(!capture_window(&no_focus, "/tmp/win.png", &|_| true));
    }

    #[test]
    fn slurp_fallback_uses_grim_geometry_capture() {
        // Mirrors tests/test_screenshot_repository.py::test_slurp_fallback_uses_grim_geometry_capture.
        let runner = FakeRunner::new()
            .with_success("slurp", &[], "10,20 30x40\n")
            .with_success("grim", &["-g", "10,20 30x40", "selection.png"], "");
        assert!(capture_area(&runner, "selection.png"));
    }

    #[test]
    fn full_and_output_capture_argv() {
        let runner = FakeRunner::new()
            .with_success("grim", &["/tmp/full.png"], "")
            .with_success("grim", &["-o", "eDP-1", "/tmp/eDP-1.png"], "");
        assert!(capture_full(&runner, "/tmp/full.png", &|_| true));
        assert!(capture_output(&runner, "eDP-1", "/tmp/eDP-1.png", &|_| true));
        assert!(!capture_full(&runner, "/tmp/missing.png", &|_| false));
    }

    #[test]
    fn screenshot_folder_prefers_saved_setting() {
        // Mirrors tests/test_screenshot_repository.py::test_set_and_get_screenshot_folder.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let runner = FakeRunner::new();
        let folder = resolve_screenshot_folder(&runner, Some(&path), "/home/u", &|p| {
            std::path::Path::new(p).is_dir()
        });
        assert_eq!(folder, path);
        assert_eq!(normalize_screenshot_folder("~/shots", "/home/u"), "/home/u/shots");
    }

    #[test]
    fn screenshot_folder_falls_back_to_xdg_pictures() {
        let runner = FakeRunner::new()
            .with_success("xdg-user-dir", &["PICTURES"], "/home/u/Pictures\n");
        let folder = resolve_screenshot_folder(&runner, None, "/home/u", &|_| false);
        assert_eq!(folder, "/home/u/Pictures/screenshots");
        let empty = FakeRunner::new()
            .with_success("xdg-user-dir", &["PICTURES"], "\n");
        let folder = resolve_screenshot_folder(&runner, None, "/home/u", &|p| {
            let _ = empty.run("xdg-user-dir", &["PICTURES"]);
            let _ = p;
            false
        });
        assert_eq!(folder, "/home/u/Pictures/screenshots");
    }

    #[test]
    fn publish_copies_png_notifies_and_prints() {
        let runner = FakeRunner::new()
            .with_success(
                "sh",
                &["-c", "wl-copy --type image/png < '/tmp/s.png'"],
                "",
            )
            .with_success(
                "notify-send",
                &["-a", "SwayManager", "Captura da tela salva em /tmp/s.png"],
                "",
            );
        assert!(publish_screenshot(
            &runner,
            ScreenshotMode::Full,
            "/tmp/s.png",
            &|_| true
        ));
        assert!(!publish_screenshot(
            &runner,
            ScreenshotMode::Full,
            "/tmp/s.png",
            &|_| false
        ));
    }
}
