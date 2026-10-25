//! `theme status|toggle` — GTK/Qt/Kitty theme state.
//!
//! Mirrors the Waybar-facing half of `GtkQtThemeRepository`
//! (`src/infrastructure/theme/gtk_qt_theme_repository.py:19-90`) and
//! `ThemeState.to_waybar_json`
//! (`src/domain/theme/entities.py:21-34`).
//!
//! Full theme application (gsettings, kitty copy, Qt ini edits,
//! kvantum/plasma) lives behind [`ThemeApplier`] so headless parity
//! tests assert argv without touching the session.

use crate::process::CommandRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeState {
    pub current_theme: String,
}

impl ThemeState {
    pub fn to_waybar_json(&self) -> serde_json::Value {
        if self.current_theme == "light" {
            serde_json::json!({
                "text": "☀️",
                "tooltip": "Tema Light",
                "class": "theme-light",
            })
        } else {
            serde_json::json!({
                "text": "🌕",
                "tooltip": "Tema Dark",
                "class": "theme-dark",
            })
        }
    }

    /// Python `json.dumps` bytes: `{"text", "tooltip", "class"}` order.
    pub fn to_waybar_string(&self) -> String {
        let (icon, tooltip, class) = if self.current_theme == "light" {
            ("☀️", "Tema Light", "theme-light")
        } else {
            ("🌕", "Tema Dark", "theme-dark")
        };
        crate::waybar::WaybarJson::new(vec![("text", icon.to_string()), ("tooltip", tooltip.to_string()), ("class", class.to_string())])
            .dumps_ascii()
    }
}

/// Filesystem seam for `~/.config/theme_toggle/theme`.
pub trait ThemeFs: Send + Sync {
    fn read_theme(&self, path: &str) -> Option<String>;
    fn write_theme(&self, path: &str, theme: &str);
    fn kitty_source(&self, path: &str) -> bool;
    fn copy_kitty_theme(&self, src: &str, dst: &str) -> Result<(), String>;
}

pub fn theme_file(home: &str) -> String {
    format!("{home}/.config/theme_toggle/theme")
}

pub fn get_state(fs: &dyn ThemeFs, home: &str) -> ThemeState {
    let theme = fs
        .read_theme(&theme_file(home))
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "light".to_string());
    ThemeState { current_theme: theme }
}

/// Applies the new theme to GTK/Qt/Kitty (argv asserted in tests).
pub trait ThemeApplier: Send + Sync {
    fn apply(&self, runner: &dyn CommandRunner, fs: &dyn ThemeFs, home: &str, new_theme: &str);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemApplier;

impl ThemeApplier for SystemApplier {
    fn apply(&self, runner: &dyn CommandRunner, fs: &dyn ThemeFs, home: &str, new_theme: &str) {
        let (color_scheme, gtk_theme, kitty_theme) = if new_theme == "dark" {
            ("prefer-dark", "Adwaita-dark", "black.conf")
        } else {
            ("prefer-light", "Adwaita", "white.conf")
        };
        runner.run(
            "gsettings",
            &["set", "org.gnome.desktop.interface", "color-scheme", color_scheme],
        );
        runner.run(
            "gsettings",
            &["set", "org.gnome.desktop.interface", "gtk-theme", gtk_theme],
        );
        let src = format!("{home}/.config/kitty/themes/{kitty_theme}");
        let dst = format!("{home}/.config/kitty/theme.conf");
        if fs.kitty_source(&src)
            && let Err(err) = fs.copy_kitty_theme(&src, &dst)
        {
            eprintln!("Error copying kitty theme: {err}");
        }
    }
}

/// Mirrors `GtkQtThemeRepository.toggle` incl. notify + literal output.
pub fn toggle(
    runner: &dyn CommandRunner,
    fs: &dyn ThemeFs,
    applier: &dyn ThemeApplier,
    home: &str,
) -> String {
    let current = get_state(fs, home).current_theme;
    let new_theme = if current == "light" { "dark" } else { "light" };
    applier.apply(runner, fs, home, new_theme);
    if let Some(parent) = std::path::Path::new(&theme_file(home)).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    fs.write_theme(&theme_file(home), new_theme);
    let title = format!("Trocando para o tema {new_theme}");
    runner.run("notify-send", &["-a", "SwayManager", &title]);
    title
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct MemFs {
        files: Arc<Mutex<HashMap<String, String>>>,
    }

    impl ThemeFs for MemFs {
        fn read_theme(&self, path: &str) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
        fn write_theme(&self, path: &str, theme: &str) {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_string(), theme.to_string());
        }
        fn kitty_source(&self, path: &str) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }
        fn copy_kitty_theme(&self, src: &str, dst: &str) -> Result<(), String> {
            let text = self
                .files
                .lock()
                .unwrap()
                .get(src)
                .cloned()
                .ok_or_else(|| "missing".to_string())?;
            self.files.lock().unwrap().insert(dst.to_string(), text);
            Ok(())
        }
    }

    #[test]
    fn waybar_literals() {
        assert_eq!(
            ThemeState { current_theme: "light".to_string() }.to_waybar_json()["class"],
            serde_json::json!("theme-light")
        );
        assert_eq!(
            ThemeState { current_theme: "dark".to_string() }.to_waybar_json()["class"],
            serde_json::json!("theme-dark")
        );
    }

    #[test]
    fn toggle_flips_and_applies_gtk_argv() {
        let runner = FakeRunner::new()
            .with_success("gsettings", &["set", "org.gnome.desktop.interface", "color-scheme", "prefer-dark"], "")
            .with_success("gsettings", &["set", "org.gnome.desktop.interface", "gtk-theme", "Adwaita-dark"], "")
            .with_success("notify-send", &[], "");
        let fs = MemFs::default();
        let msg = toggle(&runner, &fs, &SystemApplier, "/home/u");
        assert_eq!(msg, "Trocando para o tema dark");
        assert_eq!(
            fs.files.lock().unwrap().get("/home/u/.config/theme_toggle/theme").cloned(),
            Some("dark".to_string())
        );
    }
}
