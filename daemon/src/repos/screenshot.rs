//! `screenshot` mode parsing + compositor capture argv.
//!
//! The pixel work (Qt freeze overlay, `QImage` crop) stays in Python.
//! This port owns the headless contract: `ScreenshotMode` mapping from
//! `handle_screenshot` (`handlers.py:137-148`), the `slurp` geometry
//! parser (`_parse_geometry`), and the `grim`/`wl-copy` argv the daemon
//! path shells out to.

use crate::process::CommandRunner;

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
}
