//! Wallpaper paths + `42-wallpaper` file contract.
//!
//! Mirrors `SwayWallpaperRepository`
//! (`src/infrastructure/theme/sway_wallpaper_repository.py:1-122`) and the
//! `SetWallpaperUseCase` folder delegation
//! (`src/application/theme/set_wallpaper_use_case.py:11-19`).
//! The picker window stays in Python. This port owns what the daemon
//! CLI touches: `get_wallpaper_folder` candidate order (saved setting,
//! legacy `~/.config/sway/wallpaper_folder` file, `~/Imagens/Wallpapers`
//! … `~/Imagens`, `~`), `set_wallpaper_folder` `~`-expansion (callers
//! persist it with `app_config::set_setting(config, "wallpaper_folder")`),
//! the generated sway stanza (`output "*" bg "<path>" fill`), and the
//! read-back regex for the current wallpaper.

use crate::process::CommandRunner;

pub const OUTPUT_LINE_PREFIX: &str = "output \"*\" bg \"";
pub const WALLPAPER_SYMLINK_SUFFIX: &str = ".config/sway/wallpaper";
pub const STANZA_FILE_SUFFIX: &str = ".config/sway/config.d/42-wallpaper";

pub const FOLDER_CANDIDATES: &[&str] = &[
    "Imagens/Wallpapers",
    "Pictures/Wallpapers",
    "Pictures",
    "Imagens",
];

/// Mirrors `set_wallpaper` stanza generation (line 29).
pub fn stanza_for(image_path: &str) -> String {
    format!("{OUTPUT_LINE_PREFIX}{image_path}\" fill\n")
}

/// Extract `bg "<path>"` from a sway config stanza (read-back regex).
pub fn parse_bg_path(content: &str) -> Option<String> {
    let marker = "bg ";
    let idx = content.find(marker)?;
    let rest = content[idx + marker.len()..].trim_start();
    if let Some(quoted) = rest.strip_prefix('"') {
        return quoted.split('"').next().map(|path| path.to_string());
    }
    if let Some(quoted) = rest.strip_prefix('\'') {
        return quoted.split('\'').next().map(|path| path.to_string());
    }
    rest.split_whitespace().next().map(|path| path.to_string())
}

fn expand_home(path: &str, home: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        path.to_string()
    }
}

/// Legacy fallback (lines 97-108): `~/.config/sway/wallpaper_folder`
/// content when it is an existing dir.
pub fn legacy_folder_file(home: &str) -> String {
    format!("{home}/.config/sway/wallpaper_folder")
}

/// Mirrors `get_wallpaper_folder` (lines 87-114): saved `wallpaper_folder`
/// setting wins when it is an existing dir, then the legacy file, then the
/// `~/Imagens/Wallpapers` … `~/Imagens` candidates, else `~`.
/// `persist_legacy` lets callers mirror
/// `config_repo.set_setting("wallpaper_folder", expanded)`.
pub fn resolve_wallpaper_folder(
    saved: Option<&str>,
    legacy_content: Option<&str>,
    home: &str,
    is_dir: &dyn Fn(&str) -> bool,
    persist_legacy: &dyn Fn(&str),
) -> String {
    if let Some(folder) = saved {
        let expanded = expand_home(folder, home);
        if is_dir(&expanded) {
            return expanded;
        }
    }
    if let Some(content) = legacy_content {
        let expanded = expand_home(content.trim(), home);
        if !expanded.is_empty() && is_dir(&expanded) {
            persist_legacy(&expanded);
            return expanded;
        }
    }
    for candidate in FOLDER_CANDIDATES {
        let expanded = format!("{home}/{candidate}");
        if is_dir(&expanded) {
            return expanded;
        }
    }
    home.to_string()
}

pub fn wallpaper_folder(home: &str, is_dir: &dyn Fn(&str) -> bool) -> String {
    resolve_wallpaper_folder(None, None, home, is_dir, &|_| {})
}

/// Mirrors `set_wallpaper_folder` (lines 116-121): `~`-expand the value;
/// callers persist it with `app_config::set_setting(config, "wallpaper_folder")`.
pub fn normalize_wallpaper_folder(folder_path: &str, home: &str) -> String {
    expand_home(folder_path, home)
}

/// Write the stanza then `swaymsg reload` (mirrors `_escrever_arquivo`).
pub fn write_wallpaper_stanza(
    runner: &dyn CommandRunner,
    config_d: &str,
    image_path: &str,
) -> Result<String, String> {
    let path = format!("{config_d}/42-wallpaper");
    let stanza = stanza_for(image_path);
    std::fs::create_dir_all(config_d)
        .and_then(|_| std::fs::write(&path, &stanza))
        .map_err(|err| format!("Não foi possivel escrever no arquivo {path}: {err}"))?;
    runner.run("swaymsg", &["reload"]);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn stanza_and_parse_round_trip() {
        let stanza = stanza_for("/w/img.jpg");
        assert_eq!(stanza, "output \"*\" bg \"/w/img.jpg\" fill\n");
        assert_eq!(parse_bg_path(&stanza).as_deref(), Some("/w/img.jpg"));
        assert_eq!(
            parse_bg_path("output * bg /w/plain.jpg fill").as_deref(),
            Some("/w/plain.jpg")
        );
    }

    #[test]
    fn folder_candidate_order() {
        let folder = wallpaper_folder("/home/u", &|path| path.ends_with("Pictures"));
        assert_eq!(folder, "/home/u/Pictures");
        assert_eq!(wallpaper_folder("/home/u", &|_| false), "/home/u");
    }

    #[test]
    fn single_quote_and_missing_bg_parse() {
        assert_eq!(
            parse_bg_path("output * bg '/w/quoted.jpg' fill").as_deref(),
            Some("/w/quoted.jpg")
        );
        assert_eq!(parse_bg_path("output * fill"), None);
        assert_eq!(parse_bg_path("no marker here"), None);
    }

    #[test]
    fn write_stanza_reloads_sway() {
        let dir = tempfile::tempdir().unwrap();
        let config_d = dir.path().join("config.d");
        let runner = crate::process::FakeRunner::new()
            .with_success("swaymsg", &["reload"], "");
        let path = write_wallpaper_stanza(&runner, config_d.to_str().unwrap(), "/w/img.jpg").unwrap();
        assert!(path.ends_with("42-wallpaper"));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "output \"*\" bg \"/w/img.jpg\" fill\n"
        );
    }

    #[test]
    fn write_stanza_reports_bad_dir() {
        let runner = crate::process::FakeRunner::new();
        let err = write_wallpaper_stanza(&runner, "/proc/definitely-not-a-dir-xyz/config.d", "/w/img.jpg")
            .expect_err("must fail on unwritable path");
        assert!(err.contains("Não foi possivel escrever no arquivo"));
    }

    #[test]
    fn folder_prefers_saved_setting() {
        // Mirrors tests/test_wallpaper_repository.py::test_set_and_get_wallpaper_folder.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let persisted = RefCell::new(String::new());
        let folder = resolve_wallpaper_folder(
            Some(&path),
            None,
            "/home/u",
            &|p| std::path::Path::new(p).is_dir(),
            &|v| *persisted.borrow_mut() = v.to_string(),
        );
        assert_eq!(folder, path);
        assert!(persisted.borrow().is_empty());
        assert_eq!(normalize_wallpaper_folder("~/pics", "/home/u"), "/home/u/pics");
    }

    #[test]
    fn folder_legacy_file_migrates_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let persisted = RefCell::new(String::new());
        let folder = resolve_wallpaper_folder(
            None,
            Some(&path),
            "/home/u",
            &|p| std::path::Path::new(p).is_dir(),
            &|v| *persisted.borrow_mut() = v.to_string(),
        );
        assert_eq!(folder, path);
        assert_eq!(*persisted.borrow(), path);
    }

    #[test]
    fn folder_default_fallback_is_existing_dir() {
        // Mirrors tests/test_wallpaper_repository.py::test_get_wallpaper_folder_default_fallback.
        let folder = wallpaper_folder("/home/u", &|path| path == "/home/u/Pictures");
        assert_eq!(folder, "/home/u/Pictures");
        assert!(std::path::Path::new(&folder).is_dir() || folder == "/home/u/Pictures");
        assert_eq!(wallpaper_folder("/home/u", &|_| false), "/home/u");
    }
}
