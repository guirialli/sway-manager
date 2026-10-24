//! Wallpaper paths + `42-wallpaper` file contract.
//!
//! The picker window stays in Python. This port owns what the daemon
//! CLI touches: candidate folder order
//! (`get_wallpaper_folder`), the generated sway stanza
//! (`output "*" bg "<path>" fill`), and the read-back regex for the
//! current wallpaper.

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

pub fn wallpaper_folder(home: &str, is_dir: &dyn Fn(&str) -> bool) -> String {
    for candidate in FOLDER_CANDIDATES {
        let expanded = format!("{home}/{candidate}");
        if is_dir(&expanded) {
            return expanded;
        }
    }
    home.to_string()
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
}
