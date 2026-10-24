//! Icon lookup mirroring `IconResolver` (no GTK dependency).
//!
//! `resolve_icon_path(icon_name, size=32)` exact behavior
//! (`src/infrastructure/menu/icon_resolver.py:63-152`):
//! absolute-path passthrough, candidate expansion (favorites, category,
//! webapp, generic fallbacks), active theme directory scan (`index.theme`
//! `Inherits` aware), then the fixed fallback directory matrix. Without
//! GTK/GI bindings the theme lookup walks the theme directory directly
//! instead of `Gtk.IconTheme.lookup_icon` — same files, same order.

use crate::process::CommandRunner;

pub const FALLBACK_ICON: &str = "application-x-executable";

pub const STAR_CANDIDATES: &[&str] = &[
    "favorite",
    "favorites",
    "emblem-favorite",
    "rating-rated",
    "starred",
    "star",
    "bookmark-new",
    "folder-favorites",
    "folder-starred",
    "start-here",
];

pub const GENERIC_FALLBACKS: &[&str] = &[
    "application-x-executable",
    "system-run",
    "preferences-other",
    "exec",
    "utilities-terminal",
];

pub fn category_fallbacks(target: &str) -> &'static [&'static str] {
    match target {
        "folder-network" => &[
            "applications-internet",
            "network-workgroup",
            "preferences-system-network",
            "internet-web-browser",
        ],
        "folder-code" => &["applications-development", "preferences-desktop", "utilities-terminal"],
        "folder-games" => &["applications-games", "input-gaming"],
        "folder-music" => &["folder-sound", "applications-multimedia", "audio-x-generic"],
        "folder-pictures" => &["applications-graphics", "image-x-generic"],
        "folder-documents" => &["applications-office", "x-office-document"],
        "preferences-system" => &["preferences-desktop", "system-run"],
        "folder-utils" => &["applications-utilities", "utilities-terminal"],
        "system-shutdown" => &["system-log-out", "application-exit"],
        _ => &[],
    }
}

pub fn webapp_fallbacks(target: &str) -> &'static [&'static str] {
    if ["chrome-", "brave-", "msedge-", "ffpwa-", "webapp-"]
        .iter()
        .any(|prefix| target.starts_with(prefix))
    {
        &[
            "internet-web-browser",
            "google-chrome",
            "brave-browser",
            "browser",
            "text-html",
        ]
    } else {
        &[]
    }
}

/// Candidate list in Python order: target, star/category/webapp
/// expansions, then generic fallbacks (dedup).
pub fn candidates(target: &str) -> Vec<String> {
    let target = target.trim();
    let target = if target.is_empty() { FALLBACK_ICON } else { target };
    let mut out = vec![target.to_string()];
    let mut push = |name: &str| {
        if !out.iter().any(|item| item == name) {
            out.push(name.to_string());
        }
    };
    if ["emblem-favorite", "starred", "star", "rating-rated", "favorite"].contains(&target) {
        for name in STAR_CANDIDATES {
            push(name);
        }
    }
    for name in category_fallbacks(target) {
        push(name);
    }
    for name in webapp_fallbacks(target) {
        push(name);
    }
    for name in GENERIC_FALLBACKS {
        push(name);
    }
    out
}

/// Active icon theme: `gsettings` first, then `gtk-3.0/settings.ini`
/// (`get_system_icon_theme_name`, lines 12-40).
pub fn system_theme_name(runner: &dyn CommandRunner, home: &str) -> String {
    let output = runner.run("gsettings", &["get", "org.gnome.desktop.interface", "icon-theme"]);
    let val = output.stdout.trim().trim_matches(|char| char == '\'' || char == '"').to_string();
    if output.succeeded() && !val.is_empty() {
        return val;
    }
    let ini = format!("{home}/.config/gtk-3.0/settings.ini");
    if let Ok(content) = std::fs::read_to_string(&ini) {
        let mut in_settings = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_settings = line == "[Settings]";
                continue;
            }
            if in_settings
                && let Some((key, value)) = line.split_once('=')
                && key.trim() == "gtk-icon-theme-name"
            {
                let val = value.trim().trim_matches(|char| char == '\'' || char == '"');
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    String::new()
}

fn theme_dirs(theme: &str, home: &str) -> Vec<String> {
    if theme.is_empty() {
        return Vec::new();
    }
    let mut dirs = vec![
        format!("{home}/.local/share/icons/{theme}"),
        format!("{home}/.icons/{theme}"),
        format!("/usr/share/icons/{theme}"),
        format!("/usr/local/share/icons/{theme}"),
    ];
    // `Inherits` chain from index.theme (Adwaita fallback in practice).
    for base in [format!("/usr/share/icons/{theme}"), format!("{home}/.local/share/icons/{theme}")] {
        let index = format!("{base}/index.theme");
        if let Ok(content) = std::fs::read_to_string(&index) {
            let mut in_icon_theme = false;
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with('[') {
                    in_icon_theme = line == "[Icon Theme]";
                    continue;
                }
                if in_icon_theme
                    && let Some((key, value)) = line.split_once('=')
                    && key.trim() == "Inherits"
                {
                    for parent in value.split(',').map(str::trim).filter(|part| !part.is_empty()) {
                        for dir in [
                            format!("{home}/.local/share/icons/{parent}"),
                            format!("/usr/share/icons/{parent}"),
                        ] {
                            if !dirs.contains(&dir) {
                                dirs.push(dir);
                            }
                        }
                    }
                }
            }
            break;
        }
    }
    dirs
}

fn fallback_dirs(home: &str) -> Vec<String> {
    let base_dirs = [
        format!("{home}/.local/share/icons/hicolor"),
        format!("{home}/.local/share/icons"),
        format!("{home}/.icons"),
        "/usr/share/icons/Papirus-Dark".to_string(),
        "/usr/share/icons/Papirus".to_string(),
        "/usr/share/icons/Adwaita".to_string(),
        "/usr/share/icons/hicolor".to_string(),
        "/usr/share/pixmaps".to_string(),
    ];
    let sub_types = ["places", "actions", "categories", "emblems", "status", "preferences", "apps", "mimetypes", "devices"];
    let sizes = ["32x32", "48x48", "scalable", "24x24", "16x16", "256x256", "512x512", "symbolic"];
    let mut out = Vec::new();
    for base in base_dirs {
        out.push(base.clone());
        for size in sizes {
            for sub in sub_types {
                out.push(format!("{base}/{size}/{sub}"));
                out.push(format!("{base}/{sub}"));
            }
        }
    }
    out
}

fn find_in(name: &str, dirs: &[String]) -> Option<String> {
    for dir in dirs {
        for ext in ["", ".png", ".svg", ".xpm"] {
            let candidate = format!("{dir}/{name}{ext}");
            if std::path::Path::new(&candidate).is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Full port of `IconResolver.resolve_icon_path`.
pub fn resolve_icon(runner: &dyn CommandRunner, home: &str, icon_name: &str, _size: u32) -> String {
    let target = icon_name.trim();
    let target = if target.is_empty() { FALLBACK_ICON } else { target };
    if target.starts_with('/') && std::path::Path::new(target).exists() {
        return target.to_string();
    }
    let cands = candidates(target);
    // 1. Active theme (+Inherits) — replaces Gtk.IconTheme.lookup_icon.
    let themed: Vec<String> = theme_dirs(&system_theme_name(runner, home), home)
        .into_iter()
        .flat_map(|base| {
            // Theme root first (scalable/apps often live at top via
            // `index.theme` Directory entries), then the same matrix.
            let sub_types = ["places", "actions", "categories", "emblems", "status", "preferences", "apps", "mimetypes", "devices"];
            let mut dirs = vec![base.clone()];
            for size in ["32x32", "48x48", "scalable", "24x24", "16x16", "symbolic"] {
                for sub in sub_types {
                    dirs.push(format!("{base}/{size}/{sub}"));
                }
            }
            dirs
        })
        .collect();
    for cand in &cands {
        if let Some(found) = find_in(cand, &themed) {
            return found;
        }
    }
    // 2. Fixed fallback matrix (exact Python order).
    for cand in &cands {
        if let Some(found) = find_in(cand, &fallback_dirs(home)) {
            return found;
        }
    }
    target.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    #[test]
    fn resolves_real_system_icon() {
        if !std::path::Path::new("/usr/share/icons").exists() {
            return;
        }
        let runner = FakeRunner::new()
            .with_success("gsettings", &["get", "org.gnome.desktop.interface", "icon-theme"], "Papirus-Dark\n");
        let found = resolve_icon(&runner, "/root", "edit-copy", 32);
        assert!(found.ends_with("edit-copy.svg"), "got {found}");
    }

    #[test]
    fn candidates_order_matches_python() {
        let cands = candidates("emblem-favorite");
        assert_eq!(cands[0], "emblem-favorite");
        assert!(cands.contains(&"application-x-executable".to_string()));
        // generic fallback last
        assert_eq!(cands.last().unwrap(), "utilities-terminal");
    }

    #[test]
    fn absolute_path_passthrough() {
        let runner = FakeRunner::new();
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().into_owned();
        assert_eq!(resolve_icon(&runner, "/root", &path, 32), path);
    }

    #[test]
    fn unknown_falls_back_to_generic() {
        let runner = FakeRunner::new()
            .with_failure("gsettings", &["get", "org.gnome.desktop.interface", "icon-theme"], 1);
        let name = "definitely-not-an-icon-xyz-123";
        let found = resolve_icon(&runner, "/nonexistent-home", name, 32);
        // Python returns the first generic fallback hit, else the target.
        assert!(found.ends_with("application-x-executable.svg") || found == name, "got {found}");
    }
}
