//! Appearance get/apply — GTK/icon/cursor/font options.
//!
//! Mirrors `GtkQtThemeRepository.get_appearance_settings`,
//! `get_available_options`, `apply_appearance` and the `_get_gsetting`,
//! `_get_ini_setting`, `_scan_*`, `_update_gtk2_rc`,
//! `_update_cursor_index_theme`, `_update_ini_setting` helpers
//! (`src/infrastructure/theme/gtk_qt_theme_repository.py:83-344`).
//! Entities mirror `AppearanceSettings` / `AvailableAppearanceOptions`
//! (`src/domain/theme/entities.py:5-18`).
//!
//! `apply_appearance` here covers exactly what the Python method does
//! (gsettings sets incl. color-scheme, gtk-3.0/gtk-4.0 `settings.ini`,
//! `~/.gtkrc-2.0`, cursor `index.theme`, notify
//! `"Aparência aplicada!\nTema: {gtk}"`).
//! The kitty/kvantum/plasma path (`_apply_qt_theme`, used by `toggle`)
//! already lives in [`crate::repos::theme`].

use std::collections::{BTreeMap, BTreeSet};

use crate::process::{CommandRunner, notify};

pub const GSETTINGS_SCHEMA: &str = "org.gnome.desktop.interface";
pub const FALLBACK_FONTS: &[&str] = &[
    "DejaVu Sans",
    "Inter",
    "Monospace",
    "Roboto",
    "Sans",
    "Serif",
    "Ubuntu",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppearanceSettings {
    pub gtk_theme: String,
    pub icon_theme: String,
    pub cursor_theme: String,
    pub font_name: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            gtk_theme: "Adwaita".to_string(),
            icon_theme: "Adwaita".to_string(),
            cursor_theme: "Adwaita".to_string(),
            font_name: "Sans 10".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AvailableAppearanceOptions {
    pub gtk_themes: Vec<String>,
    pub icon_themes: Vec<String>,
    pub cursor_themes: Vec<String>,
    pub fonts: Vec<String>,
}

/// Filesystem seam (`os.path` + ini/file reads/writes).
pub trait AppearanceFs: Send + Sync {
    fn is_dir(&self, path: &str) -> bool;
    fn is_file(&self, path: &str) -> bool;
    fn list_dir(&self, path: &str) -> Option<Vec<String>>;
    fn path_exists(&self, path: &str) -> bool;
    fn read_index_theme(&self, path: &str) -> Option<String>;
    fn ini_setting(&self, path: &str, key: &str) -> Option<String>;
    fn update_ini(&self, path: &str, section: &str, key: &str, value: &str);
    fn read_text(&self, path: &str) -> Option<String>;
    fn write_text(&self, path: &str, content: &str) -> bool;
}

pub fn settings_ini_path(home: &str, config_dir: &str) -> String {
    format!("{home}/.config/{config_dir}/settings.ini")
}

pub fn get_gsetting(runner: &dyn CommandRunner, key: &str) -> String {
    let out = runner.run("gsettings", &["get", GSETTINGS_SCHEMA, key]);
    if !out.succeeded() {
        return String::new();
    }
    out.stdout.trim().trim_matches(|c| c == '\'' || c == '"').to_string()
}

pub fn get_appearance_settings(
    runner: &dyn CommandRunner,
    fs: &dyn AppearanceFs,
    home: &str,
) -> AppearanceSettings {
    let pick = |key: &str, ini_key: &str, fallback: &str| {
        let v = get_gsetting(runner, key);
        if !v.is_empty() {
            return v;
        }
        let path = settings_ini_path(home, "gtk-3.0");
        if let Some(ini) = fs.ini_setting(&path, ini_key)
            && !ini.is_empty()
        {
            return ini;
        }
        fallback.to_string()
    };
    AppearanceSettings {
        gtk_theme: pick("gtk-theme", "gtk-theme-name", "Adwaita"),
        icon_theme: pick("icon-theme", "gtk-icon-theme-name", "Adwaita"),
        cursor_theme: pick("cursor-theme", "gtk-cursor-theme-name", "Adwaita"),
        font_name: pick("font-name", "gtk-font-name", "Sans 10"),
    }
}

fn theme_search_paths(home: &str) -> Vec<String> {
    vec![
        "/usr/share/themes".to_string(),
        "/usr/local/share/themes".to_string(),
        format!("{home}/.themes"),
        format!("{home}/.local/share/themes"),
    ]
}

fn icon_search_paths(home: &str) -> Vec<String> {
    vec![
        "/usr/share/icons".to_string(),
        "/usr/local/share/icons".to_string(),
        format!("{home}/.icons"),
        format!("{home}/.local/share/icons"),
    ]
}

pub fn scan_gtk_themes(fs: &dyn AppearanceFs, home: &str) -> Vec<String> {
    let markers = ["gtk-2.0", "gtk-3.0", "gtk-4.0", "index.theme"];
    let mut themes = BTreeSet::new();
    for base in theme_search_paths(home) {
        let Some(items) = fs.list_dir(&base) else {
            continue;
        };
        for item in items {
            let full = format!("{base}/{item}");
            if !fs.is_dir(&full) {
                continue;
            }
            if markers.iter().any(|m| fs.path_exists(&format!("{full}/{m}"))) {
                themes.insert(item);
            }
        }
    }
    themes.into_iter().collect()
}

pub fn scan_icon_themes(fs: &dyn AppearanceFs, home: &str) -> Vec<String> {
    let ignore = ["default", "vendor", "distrobox", "zbar.ico"];
    let mut icons = BTreeSet::new();
    for base in icon_search_paths(home) {
        let Some(items) = fs.list_dir(&base) else {
            continue;
        };
        for item in items {
            if item.starts_with('.') || ignore.contains(&item.as_str()) {
                continue;
            }
            let full = format!("{base}/{item}");
            if !fs.is_dir(&full) {
                continue;
            }
            if let Some(content) = fs.read_index_theme(&format!("{full}/index.theme"))
                && content.contains("[Icon Theme]")
            {
                icons.insert(item);
            }
        }
    }
    icons.into_iter().collect()
}

pub fn scan_cursor_themes(fs: &dyn AppearanceFs, home: &str) -> Vec<String> {
    let mut cursors = BTreeSet::new();
    for base in icon_search_paths(home) {
        let Some(items) = fs.list_dir(&base) else {
            continue;
        };
        for item in items {
            if item.starts_with('.') {
                continue;
            }
            let full = format!("{base}/{item}");
            if !fs.is_dir(&full) {
                continue;
            }
            if fs.is_dir(&format!("{full}/cursors")) {
                cursors.insert(item);
            }
        }
    }
    cursors.into_iter().collect()
}

pub fn scan_system_fonts(runner: &dyn CommandRunner) -> Vec<String> {
    let out = runner.run("fc-list", &[":", "family"]);
    let mut fonts = BTreeSet::new();
    if out.succeeded() && !out.stdout.is_empty() {
        for line in out.stdout.lines() {
            for name in line.split(',') {
                let clean = name.trim().to_string();
                if !clean.is_empty() && !clean.starts_with('.') {
                    fonts.insert(clean);
                }
            }
        }
    }
    if fonts.is_empty() {
        for f in FALLBACK_FONTS {
            fonts.insert((*f).to_string());
        }
    }
    fonts.into_iter().collect()
}

pub fn get_available_options(
    runner: &dyn CommandRunner,
    fs: &dyn AppearanceFs,
    home: &str,
) -> AvailableAppearanceOptions {
    AvailableAppearanceOptions {
        gtk_themes: scan_gtk_themes(fs, home),
        icon_themes: scan_icon_themes(fs, home),
        cursor_themes: scan_cursor_themes(fs, home),
        fonts: scan_system_fonts(runner),
    }
}

fn update_gtk2_rc(fs: &dyn AppearanceFs, home: &str, settings: &AppearanceSettings) {
    let path = format!("{home}/.gtkrc-2.0");
    let existing = fs.read_text(&path).unwrap_or_default();
    let existing_lines: Vec<String> = if existing.is_empty() {
        Vec::new()
    } else {
        existing.lines().map(|l| format!("{l}\n")).collect()
    };
    let kv = [
        ("gtk-theme-name", settings.gtk_theme.as_str()),
        ("gtk-icon-theme-name", settings.icon_theme.as_str()),
        ("gtk-cursor-theme-name", settings.cursor_theme.as_str()),
        ("gtk-font-name", settings.font_name.as_str()),
    ];
    let mut found = BTreeSet::new();
    let mut new_lines: Vec<String> = Vec::new();
    for line in &existing_lines {
        let stripped = line.trim();
        let mut updated = false;
        for (k, v) in &kv {
            if stripped.starts_with(&format!("{k}=")) {
                new_lines.push(format!("{k}=\"{v}\"\n"));
                found.insert(*k);
                updated = true;
                break;
            }
        }
        if !updated {
            new_lines.push(line.clone());
        }
    }
    for (k, v) in &kv {
        if !found.contains(k) && !v.is_empty() {
            new_lines.push(format!("{k}=\"{v}\"\n"));
        }
    }
    fs.write_text(&path, &new_lines.concat());
}

/// Mirrors `GtkQtThemeRepository.apply_appearance` incl. notify literal.
pub fn apply_appearance(
    runner: &dyn CommandRunner,
    fs: &dyn AppearanceFs,
    home: &str,
    settings: &AppearanceSettings,
) -> bool {
    if !settings.gtk_theme.is_empty() {
        runner.run("gsettings", &["set", GSETTINGS_SCHEMA, "gtk-theme", &settings.gtk_theme]);
        let scheme = if settings.gtk_theme.to_lowercase().contains("dark") {
            "prefer-dark"
        } else {
            "default"
        };
        runner.run("gsettings", &["set", GSETTINGS_SCHEMA, "color-scheme", scheme]);
    }
    if !settings.icon_theme.is_empty() {
        runner.run("gsettings", &["set", GSETTINGS_SCHEMA, "icon-theme", &settings.icon_theme]);
    }
    if !settings.cursor_theme.is_empty() {
        runner.run("gsettings", &["set", GSETTINGS_SCHEMA, "cursor-theme", &settings.cursor_theme]);
    }
    if !settings.font_name.is_empty() {
        runner.run("gsettings", &["set", GSETTINGS_SCHEMA, "font-name", &settings.font_name]);
    }
    for ver in ["gtk-3.0", "gtk-4.0"] {
        let path = settings_ini_path(home, ver);
        if !settings.gtk_theme.is_empty() {
            fs.update_ini(&path, "Settings", "gtk-theme-name", &settings.gtk_theme);
        }
        if !settings.icon_theme.is_empty() {
            fs.update_ini(&path, "Settings", "gtk-icon-theme-name", &settings.icon_theme);
        }
        if !settings.cursor_theme.is_empty() {
            fs.update_ini(&path, "Settings", "gtk-cursor-theme-name", &settings.cursor_theme);
        }
        if !settings.font_name.is_empty() {
            fs.update_ini(&path, "Settings", "gtk-font-name", &settings.font_name);
        }
    }
    update_gtk2_rc(fs, home, settings);
    if !settings.cursor_theme.is_empty() {
        fs.write_text(
            &format!("{home}/.icons/default/index.theme"),
            &format!("[Icon Theme]\nInherits={}\n", settings.cursor_theme),
        );
    }
    notify(
        runner,
        "SwayManager",
        &format!("Aparência aplicada!\nTema: {}", settings.gtk_theme),
        "normal",
    );
    true
}

pub fn appearance_map(settings: &AppearanceSettings) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("gtk_theme".to_string(), settings.gtk_theme.clone()),
        ("icon_theme".to_string(), settings.icon_theme.clone()),
        ("cursor_theme".to_string(), settings.cursor_theme.clone()),
        ("font_name".to_string(), settings.font_name.clone()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CmdOutput, FakeRunner};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mirrors `tests/test_use_cases.py::test_manage_appearance_use_case`
    /// (get returns settings, apply round-trips them) plus gsettings
    /// argv parity from `gtk_qt_theme_repository.py:104-147`.
    #[derive(Debug, Default)]
    struct MemFs {
        dirs: BTreeSet<String>,
        exists: BTreeSet<String>,
        listings: HashMap<String, Vec<String>>,
        indexes: HashMap<String, String>,
        files: Arc<Mutex<HashMap<String, String>>>,
        inis: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MemFs {
        fn with_list(mut self, dir: &str, items: &[&str]) -> Self {
            self.dirs.insert(dir.to_string());
            self.listings.insert(
                dir.to_string(),
                items.iter().map(|s| s.to_string()).collect(),
            );
            for item in items {
                self.dirs.insert(format!("{dir}/{item}"));
            }
            self
        }
        fn with_marker(mut self, path: &str) -> Self {
            self.exists.insert(path.to_string());
            self
        }
        fn with_index(mut self, path: &str, content: &str) -> Self {
            self.indexes.insert(path.to_string(), content.to_string());
            self
        }
        fn writes(&self) -> HashMap<String, String> {
            self.files.lock().unwrap().clone()
        }
        fn ini_updates(&self) -> HashMap<String, String> {
            self.inis.lock().unwrap().clone()
        }
    }

    impl AppearanceFs for MemFs {
        fn is_dir(&self, path: &str) -> bool {
            self.dirs.contains(path) || self.exists.contains(path)
        }
        fn is_file(&self, _path: &str) -> bool {
            false
        }
        fn list_dir(&self, path: &str) -> Option<Vec<String>> {
            self.listings.get(path).cloned()
        }
        fn path_exists(&self, path: &str) -> bool {
            self.exists.contains(path) || self.indexes.contains_key(path)
        }
        fn read_index_theme(&self, path: &str) -> Option<String> {
            self.indexes.get(path).cloned()
        }
        fn ini_setting(&self, path: &str, key: &str) -> Option<String> {
            self.inis.lock().unwrap().get(&format!("{path}\u{1f}{key}")).cloned()
        }
        fn update_ini(&self, path: &str, _section: &str, key: &str, value: &str) {
            self.inis.lock().unwrap().insert(format!("{path}\u{1f}{key}"), value.to_string());
        }
        fn read_text(&self, path: &str) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
        fn write_text(&self, path: &str, content: &str) -> bool {
            self.files.lock().unwrap().insert(path.to_string(), content.to_string());
            true
        }
    }
    #[derive(Debug, Default)]
    struct RecRunner {
        inner: FakeRunner,
        calls: Arc<Mutex<Vec<Vec<String>>>>,
    }

    impl RecRunner {
        fn new(inner: FakeRunner) -> Self {
            Self { inner, calls: Arc::new(Mutex::new(Vec::new())) }
        }
        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
        fn argv(program: &str, args: &[&str]) -> Vec<String> {
            std::iter::once(program.to_string()).chain(args.iter().map(|s| s.to_string())).collect()
        }
    }

    impl CommandRunner for RecRunner {
        fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
            self.calls.lock().unwrap().push(Self::argv(program, args));
            self.inner.run(program, args)
        }
        fn spawn_detached(&self, program: &str, args: &[&str]) {
            self.calls.lock().unwrap().push(Self::argv(program, args));
        }
        fn which(&self, binary: &str) -> Option<String> {
            self.inner.which(binary)
        }
    }

    fn gsettings_runner(value: &str) -> FakeRunner {
        FakeRunner::new()
            .with_success("gsettings", &["get", GSETTINGS_SCHEMA, "gtk-theme"], &format!("'{value}'\n"))
            .with_success("gsettings", &["get", GSETTINGS_SCHEMA, "icon-theme"], "'Adwaita'\n")
            .with_success("gsettings", &["get", GSETTINGS_SCHEMA, "cursor-theme"], "'Adwaita'\n")
            .with_success("gsettings", &["get", GSETTINGS_SCHEMA, "font-name"], "'Sans 10'\n")
    }

    #[test]
    fn get_settings_prefers_gsettings_over_ini() {
        let runner = gsettings_runner("Orchis-Dark");
        let fs = MemFs::default();
        let s = get_appearance_settings(&runner, &fs, "/home/u");
        assert_eq!(s.gtk_theme, "Orchis-Dark");
        assert_eq!(s.font_name, "Sans 10");
    }

    #[test]
    fn get_settings_falls_back_to_ini_then_default() {
        let runner = FakeRunner::new();
        let fs = MemFs::default();
        fs.inis.lock().unwrap().insert(
            "/home/u/.config/gtk-3.0/settings.ini\u{1f}gtk-theme-name".to_string(),
            "Nordic".to_string(),
        );
        let s = get_appearance_settings(&runner, &fs, "/home/u");
        assert_eq!(s.gtk_theme, "Nordic");
        assert_eq!(s.icon_theme, "Adwaita");
    }

    #[test]
    fn scan_themes_icons_cursors_sorted() {
        let fs = MemFs::default()
            .with_list("/usr/share/themes", &["Zeta", "Alpha"])
            .with_marker("/usr/share/themes/Alpha/gtk-3.0")
            .with_marker("/usr/share/themes/Zeta/index.theme")
            .with_list("/usr/share/icons", &["Beta", "default", "Alfa"])
            .with_index("/usr/share/icons/Alfa/index.theme", "[Icon Theme]\nName=Alfa\n")
            .with_index("/usr/share/icons/Beta/index.theme", "[Icon Theme]\nName=Beta\n")
            .with_marker("/usr/share/icons/Beta/cursors");
        assert_eq!(scan_gtk_themes(&fs, "/home/u"), vec!["Alpha".to_string(), "Zeta".to_string()]);
        // "default" is ignored for icons but still counts as cursor dir only if has cursors/
        assert_eq!(scan_icon_themes(&fs, "/home/u"), vec!["Alfa".to_string(), "Beta".to_string()]);
        assert_eq!(scan_cursor_themes(&fs, "/home/u"), vec!["Beta".to_string()]);
    }

    #[test]
    fn scan_fonts_parses_fc_list_with_fallback() {
        let runner = FakeRunner::new().with_success(
            "fc-list",
            &[":", "family"],
            "DejaVu Sans,DejaVu Sans Condensed\nInter\n.Hidden\n",
        );
        let fonts = scan_system_fonts(&runner);
        assert!(fonts.contains(&"Inter".to_string()));
        assert!(fonts.contains(&"DejaVu Sans".to_string()));
        assert!(!fonts.iter().any(|f| f.starts_with('.')));

        let empty = FakeRunner::new()
            .with_output("fc-list", &[":", "family"], CmdOutput::success(""));
        let fb = scan_system_fonts(&empty);
        let mut want: Vec<String> = FALLBACK_FONTS.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(fb, want);
    }

    #[test]
    fn apply_appearance_dark_sets_color_scheme_and_notifies() {
        let runner = RecRunner::new(FakeRunner::new());
        let fs = MemFs::default();
        let settings = AppearanceSettings {
            gtk_theme: "Adwaita-dark".to_string(),
            icon_theme: "Adwaita".to_string(),
            cursor_theme: "Bibata".to_string(),
            font_name: "Sans 11".to_string(),
        };
        assert!(apply_appearance(&runner, &fs, "/home/u", &settings));
        let calls = runner.calls();
        for want in [
            vec!["gsettings", "set", GSETTINGS_SCHEMA, "gtk-theme", "Adwaita-dark"],
            vec!["gsettings", "set", GSETTINGS_SCHEMA, "color-scheme", "prefer-dark"],
            vec!["gsettings", "set", GSETTINGS_SCHEMA, "icon-theme", "Adwaita"],
            vec!["gsettings", "set", GSETTINGS_SCHEMA, "cursor-theme", "Bibata"],
            vec!["gsettings", "set", GSETTINGS_SCHEMA, "font-name", "Sans 11"],
        ] {
            let want: Vec<String> = want.into_iter().map(str::to_string).collect();
            assert!(calls.contains(&want), "missing {want:?} in {calls:?}");
        }
        assert!(
            calls.contains(&vec![
                "notify-send".to_string(),
                "-a".to_string(),
                "SwayManager".to_string(),
                "SwayManager".to_string(),
                "Aparência aplicada!\nTema: Adwaita-dark".to_string(),
            ]),
        );
        let writes = fs.writes();
        assert_eq!(
            writes.get("/home/u/.icons/default/index.theme").map(String::as_str),
            Some("[Icon Theme]\nInherits=Bibata\n")
        );
        assert!(writes.get("/home/u/.gtkrc-2.0").unwrap().contains("gtk-theme-name=\"Adwaita-dark\""));
        let inis = fs.ini_updates();
        assert_eq!(
            inis.get("/home/u/.config/gtk-4.0/settings.ini\u{1f}gtk-theme-name").map(String::as_str),
            Some("Adwaita-dark")
        );
    }
}
