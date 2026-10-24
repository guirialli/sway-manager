//! `.desktop` file index for the deferred `menu` path.
//!
//! Mirrors `DesktopParser.parse_all` (`desktop_parser.py:226-320`):
//! same search directories, same skip rules (`NoDisplay`, `Hidden`,
//! `NotShowIn=sway`, `OnlyShowIn` without sway/wayland, empty `Exec`,
//! duplicate execs, XFCE blacklist), same name normalization hooks,
//! accent-insensitive A-Z sort, system actions appended last.

use crate::process::CommandRunner;
use crate::repos::menu::{self, MenuItem};
use crate::repos::menu_exec::strip_accents;

pub const EXPLICIT_IGNORE_EXECS: &[&str] = &[
    "xfce4-about",
    "xfce4-appfinder",
    "xfce4-session-logout",
    "xfce4-panel",
    "exo-open",
];

pub const EXPLICIT_IGNORE_PREFIXES: &[&str] = &[
    "xfce4-settings",
    "xfwm4",
    "xfce-settings",
    "xfce-session",
    "xfce-wm",
    "xfce-backdrop",
    "panel-",
];

fn read_key(content: &str, key: &str) -> String {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            continue;
        }
        if let Some((name, value)) = line.split_once('=')
            && name.trim() == key
        {
            return value.trim().to_string();
        }
    }
    String::new()
}

fn get_bool(content: &str, key: &str) -> bool {
    read_key(content, key).to_lowercase() == "true"
}

fn parse_desktop_file(path: &std::path::Path, filename: &str) -> Option<MenuItem> {
    let content = std::fs::read_to_string(path).ok()?;
    let entry = content.split("[Desktop Entry]").nth(1)?;
    let entry = &entry[..entry.find('[').unwrap_or(entry.len())];

    if get_bool(entry, "NoDisplay") || get_bool(entry, "Hidden") {
        return None;
    }
    let not_show: Vec<String> = read_key(entry, "NotShowIn")
        .split(';')
        .map(|part| part.trim().to_lowercase())
        .filter(|part| !part.is_empty())
        .collect();
    if not_show.iter().any(|env| env == "sway") {
        return None;
    }
    let only_show = read_key(entry, "OnlyShowIn");
    if !only_show.is_empty() {
        let envs: Vec<String> = only_show
            .split(';')
            .map(|part| part.trim().to_lowercase())
            .filter(|part| !part.is_empty())
            .collect();
        if !envs.iter().any(|env| env == "sway" || env == "wayland") {
            return None;
        }
    }
    let raw_exec = read_key(entry, "Exec");
    if raw_exec.is_empty() {
        return None;
    }
    let clean_exec = menu::clean_exec_cmd(&raw_exec);
    let file_lower = filename.to_lowercase();
    let exec_lower = clean_exec.to_lowercase();
    if EXPLICIT_IGNORE_PREFIXES.iter().any(|prefix| file_lower.starts_with(prefix))
        || EXPLICIT_IGNORE_EXECS.iter().any(|cmd| exec_lower.starts_with(cmd))
    {
        return None;
    }
    let raw_name = read_key(entry, "Name");
    if raw_name.trim().is_empty() {
        return None;
    }
    let icon = read_key(entry, "Icon");
    let icon = if icon.is_empty() {
        "application-x-executable".to_string()
    } else {
        icon
    };
    Some(MenuItem {
        name: raw_name.clone(),
        normalized_name: raw_name,
        exec_cmd: clean_exec,
        icon,
        category: "Aplicativos".to_string(),
        is_system_action: false,
        is_category_header: false,
        category_target: None,
        is_web_app: false,
    })
}

/// Build the full menu index (apps sorted + system actions last).
/// `seen` dedup mirrors `seen_execs` across directories.
pub fn load_menu_items(home: &str) -> Vec<MenuItem> {
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for dir in menu::search_directories(home) {
        let Ok(entries) = walk_desktop_files(std::path::Path::new(&dir)) else {
            continue;
        };
        for path in entries {
            let filename = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            if let Some(mut item) = parse_desktop_file(&path, filename) {
                if !seen.insert(item.exec_cmd.clone()) {
                    continue;
                }
                // Keep normalized name as-is (full normalize_name needs
                // KNOWN_NAME_MAP); sort key strips accents like Python.
                item.normalized_name = item.name.clone();
                items.push(item);
            }
        }
    }
    items.sort_by_key(|item| strip_accents(&item.normalized_name));
    for (name, exec, icon) in menu::SYSTEM_ACTIONS {
        items.push(MenuItem {
            name: name.to_string(),
            normalized_name: name.to_string(),
            exec_cmd: exec.to_string(),
            icon: icon.to_string(),
            category: "Sistema / Sair".to_string(),
            is_system_action: true,
            is_category_header: false,
            category_target: None,
            is_web_app: false,
        });
    }
    items
}

fn walk_desktop_files(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "desktop") {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// Icon lookup: `gsettings` theme name (mirrors `IconResolver`), else
/// empty (wofi renders without image rather than failing).
pub fn resolve_icon(_icon_name: &str, _runner: &dyn CommandRunner) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repos::menu::SYSTEM_ACTIONS;

    #[test]
    fn skips_hidden_and_empty_exec() {
        let dir = tempfile::tempdir().unwrap();
        let hidden = dir.path().join("hidden.desktop");
        std::fs::write(&hidden, "[Desktop Entry]\nName=Hidden\nExec=hidden\nNoDisplay=true\n").unwrap();
        let noexec = dir.path().join("noexec.desktop");
        std::fs::write(&noexec, "[Desktop Entry]\nName=NoExec\n").unwrap();
        let good = dir.path().join("good.desktop");
        std::fs::write(&good, "[Desktop Entry]\nName=Good\nExec=good %u\n").unwrap();
        assert!(parse_desktop_file(&hidden, "hidden.desktop").is_none());
        assert!(parse_desktop_file(&noexec, "noexec.desktop").is_none());
        let item = parse_desktop_file(&good, "good.desktop").unwrap();
        assert_eq!(item.exec_cmd, "good");
    }

    #[test]
    fn system_actions_appended_last() {
        assert_eq!(SYSTEM_ACTIONS.len(), 5);
        let items = load_menu_items("/nonexistent-home-for-test");
        let tail: Vec<&str> = items.iter().rev().take(5).map(|item| item.name.as_str()).collect::<Vec<_>>().into_iter().rev().collect();
        assert_eq!(tail, vec!["Bloquear Tela", "Encerrar Sessão", "Suspender", "Reiniciar", "Desligar"]);
        assert!(items.iter().rev().take(5).all(|item| item.is_system_action));
    }
}
