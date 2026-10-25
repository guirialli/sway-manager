//! LightDM greeter/daemon settings, users and sessions.
//!
//! Mirrors `LightDMRepository`
//! (`src/infrastructure/theme/lightdm_repository.py:11-218`) with
//! `LightDMSettings` defaults and `to_dict`/`from_dict` key mapping from
//! `src/domain/theme/entities.py:41-135`.

use std::collections::BTreeMap;

use crate::process::CommandRunner;

pub const GREETER_CONFIG: &str = "/etc/lightdm/lightdm-gtk-greeter.conf";
pub const DAEMON_CONFIG: &str = "/etc/lightdm/lightdm.conf";
pub const SYSTEM_WALLPAPER: &str = "/etc/lightdm/background.jpg";
pub const TMP_GREETER_PATH: &str = "/tmp/lightdm-gtk-greeter.conf.tmp";
pub const TMP_DAEMON_PATH: &str = "/tmp/lightdm.conf.tmp";

pub const GREETER_KEYS: &[&str] = &[
    "background",
    "theme-name",
    "icon-theme-name",
    "font-name",
    "cursor-theme-name",
    "cursor-theme-size",
    "xft-antialias",
    "xft-dpi",
    "xft-hintstyle",
    "xft-rgba",
    "clock-format",
    "position",
    "active-monitor",
    "screensaver-timeout",
    "indicators",
    "draw-user-backgrounds",
    "draw-grid",
    "hide-user-image",
    "default-user-image",
];

pub const DAEMON_KEYS: &[&str] = &[
    "autologin-user",
    "autologin-user-timeout",
    "autologin-session",
    "user-session",
    "greeter-show-manual-login",
    "greeter-hide-users",
    "allow-guest",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightDmSettings {
    pub background: String,
    pub theme_name: String,
    pub icon_theme_name: String,
    pub font_name: String,
    pub cursor_theme_name: String,
    pub cursor_theme_size: String,
    pub xft_antialias: String,
    pub xft_dpi: String,
    pub xft_hintstyle: String,
    pub xft_rgba: String,
    pub clock_format: String,
    pub position: String,
    pub active_monitor: String,
    pub screensaver_timeout: String,
    pub indicators: String,
    pub draw_user_backgrounds: String,
    pub draw_grid: String,
    pub hide_user_image: String,
    pub default_user_image: String,
    pub autologin_user: String,
    pub autologin_user_timeout: String,
    pub autologin_session: String,
    pub user_session: String,
    pub greeter_show_manual_login: String,
    pub greeter_hide_users: String,
    pub allow_guest: String,
}

impl Default for LightDmSettings {
    fn default() -> Self {
        Self {
            background: "/usr/share/backgrounds/lightdm-wallpaper.jpg".to_string(),
            theme_name: "Adwaita-dark".to_string(),
            icon_theme_name: "Adwaita".to_string(),
            font_name: "Sans 11".to_string(),
            cursor_theme_name: "Bibata-Modern-Ice".to_string(),
            cursor_theme_size: "24".to_string(),
            xft_antialias: "true".to_string(),
            xft_dpi: String::new(),
            xft_hintstyle: "hintslight".to_string(),
            xft_rgba: "rgb".to_string(),
            clock_format: "%a, %d %b %H:%M".to_string(),
            position: "50%,center".to_string(),
            active_monitor: "#cursor".to_string(),
            screensaver_timeout: "60".to_string(),
            indicators: "~host;~spacer;~clock;~spacer;~session;~language;~a11y;~power".to_string(),
            draw_user_backgrounds: "false".to_string(),
            draw_grid: "false".to_string(),
            hide_user_image: "false".to_string(),
            default_user_image: String::new(),
            autologin_user: String::new(),
            autologin_user_timeout: "0".to_string(),
            autologin_session: "sway".to_string(),
            user_session: "sway".to_string(),
            greeter_show_manual_login: "false".to_string(),
            greeter_hide_users: "false".to_string(),
            allow_guest: "false".to_string(),
        }
    }
}

impl LightDmSettings {
    pub fn to_dict(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("background".to_string(), self.background.clone()),
            ("theme-name".to_string(), self.theme_name.clone()),
            ("icon-theme-name".to_string(), self.icon_theme_name.clone()),
            ("font-name".to_string(), self.font_name.clone()),
            ("cursor-theme-name".to_string(), self.cursor_theme_name.clone()),
            ("cursor-theme-size".to_string(), self.cursor_theme_size.clone()),
            ("xft-antialias".to_string(), self.xft_antialias.clone()),
            ("xft-dpi".to_string(), self.xft_dpi.clone()),
            ("xft-hintstyle".to_string(), self.xft_hintstyle.clone()),
            ("xft-rgba".to_string(), self.xft_rgba.clone()),
            ("clock-format".to_string(), self.clock_format.clone()),
            ("position".to_string(), self.position.clone()),
            ("active-monitor".to_string(), self.active_monitor.clone()),
            ("screensaver-timeout".to_string(), self.screensaver_timeout.clone()),
            ("indicators".to_string(), self.indicators.clone()),
            ("draw-user-backgrounds".to_string(), self.draw_user_backgrounds.clone()),
            ("draw-grid".to_string(), self.draw_grid.clone()),
            ("hide-user-image".to_string(), self.hide_user_image.clone()),
            ("default-user-image".to_string(), self.default_user_image.clone()),
            ("autologin-user".to_string(), self.autologin_user.clone()),
            ("autologin-user-timeout".to_string(), self.autologin_user_timeout.clone()),
            ("autologin-session".to_string(), self.autologin_session.clone()),
            ("user-session".to_string(), self.user_session.clone()),
            ("greeter-show-manual-login".to_string(), self.greeter_show_manual_login.clone()),
            ("greeter-hide-users".to_string(), self.greeter_hide_users.clone()),
            ("allow-guest".to_string(), self.allow_guest.clone()),
        ])
    }

    pub fn from_dict(data: &BTreeMap<String, String>) -> Self {
        let d = Self::default();
        let get = |k: &str, cur: &String| data.get(k).cloned().unwrap_or_else(|| cur.clone());
        Self {
            background: get("background", &d.background),
            theme_name: get("theme-name", &d.theme_name),
            icon_theme_name: get("icon-theme-name", &d.icon_theme_name),
            font_name: get("font-name", &d.font_name),
            cursor_theme_name: get("cursor-theme-name", &d.cursor_theme_name),
            cursor_theme_size: get("cursor-theme-size", &d.cursor_theme_size),
            xft_antialias: get("xft-antialias", &d.xft_antialias),
            xft_dpi: get("xft-dpi", &d.xft_dpi),
            xft_hintstyle: get("xft-hintstyle", &d.xft_hintstyle),
            xft_rgba: get("xft-rgba", &d.xft_rgba),
            clock_format: get("clock-format", &d.clock_format),
            position: get("position", &d.position),
            active_monitor: get("active-monitor", &d.active_monitor),
            screensaver_timeout: get("screensaver-timeout", &d.screensaver_timeout),
            indicators: get("indicators", &d.indicators),
            draw_user_backgrounds: get("draw-user-backgrounds", &d.draw_user_backgrounds),
            draw_grid: get("draw-grid", &d.draw_grid),
            hide_user_image: get("hide-user-image", &d.hide_user_image),
            default_user_image: get("default-user-image", &d.default_user_image),
            autologin_user: get("autologin-user", &d.autologin_user),
            autologin_user_timeout: get("autologin-user-timeout", &d.autologin_user_timeout),
            autologin_session: get("autologin-session", &d.autologin_session),
            user_session: get("user-session", &d.user_session),
            greeter_show_manual_login: get("greeter-show-manual-login", &d.greeter_show_manual_login),
            greeter_hide_users: get("greeter-hide-users", &d.greeter_hide_users),
            allow_guest: get("allow-guest", &d.allow_guest),
        }
    }
}

/// Filesystem seam (`os.path`, config reads/writes, `/etc/passwd`).
pub trait LightDmFs: Send + Sync {
    fn is_file(&self, path: &str) -> bool;
    fn read_text(&self, path: &str) -> Option<String>;
    fn write_text(&self, path: &str, content: &str) -> bool;
    fn remove(&self, path: &str);
    fn path_exists(&self, path: &str) -> bool;
    fn is_dir(&self, path: &str) -> bool;
    fn list_dir(&self, path: &str) -> Option<Vec<String>>;
    fn read_passwd(&self) -> Option<String>;
    fn current_user(&self) -> String;
}

/// Minimal `configparser`-compatible read: `[section]` + `key = value`.
pub fn parse_ini(content: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut section = String::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once(['=', ':']) {
            out.entry(section.clone()).or_default().insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    out
}

pub fn render_ini(sections: &BTreeMap<String, BTreeMap<String, String>>) -> String {
    let mut s = String::new();
    for (section, kv) in sections {
        s.push_str(&format!("[{section}]\n"));
        for (k, v) in kv {
            s.push_str(&format!("{k} = {v}\n"));
        }
        s.push('\n');
    }
    s
}

pub fn get_settings(
    fs: &dyn LightDmFs,
    greeter_config: &str,
    daemon_config: &str,
) -> LightDmSettings {
    let mut dict = LightDmSettings::default().to_dict();
    if fs.is_file(greeter_config)
        && let Some(content) = fs.read_text(greeter_config)
    {
        let ini = parse_ini(&content);
        if let Some(greeter) = ini.get("greeter") {
            for key in GREETER_KEYS {
                if let Some(v) = greeter.get(*key) {
                    dict.insert((*key).to_string(), v.clone());
                }
            }
        }
    }
    if fs.is_file(daemon_config)
        && let Some(content) = fs.read_text(daemon_config)
    {
        let ini = parse_ini(&content);
        let mut seat = None;
        for sec in ["Seat:*", "SeatDefaults", "LightDM"] {
            if let Some(s) = ini.get(sec) {
                seat = Some(s);
                break;
            }
        }
        if let Some(seat) = seat {
            for key in DAEMON_KEYS {
                if let Some(v) = seat.get(*key) {
                    dict.insert((*key).to_string(), v.clone());
                }
            }
        }
    }
    LightDmSettings::from_dict(&dict)
}

/// POSIX-shell single-quote escaping (`shlex.quote` equivalent).
pub fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    let safe = s.chars().all(|c| c.is_ascii_alphanumeric() || "_-+=:,./".contains(c));
    if safe {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}
/// `greeter_config`/`daemon_config`/tmp paths are parameters (not
/// constants) so tests can point at temp files — Python parity via
/// instance attributes.
#[allow(clippy::too_many_arguments)]
pub fn save_settings(
    runner: &dyn CommandRunner,
    fs: &dyn LightDmFs,
    settings: &LightDmSettings,
    image_source_path: Option<&str>,
    greeter_config: &str,
    daemon_config: &str,
    system_wallpaper: &str,
    tmp_greeter: &str,
    tmp_daemon: &str,
) -> bool {
    let mut cmd_parts = vec!["mkdir -p /etc/lightdm".to_string()];
    let mut new_settings = settings.to_dict();
    if let Some(src) = image_source_path
        && fs.is_file(src)
    {
        cmd_parts.push(format!("cp {} {}", shell_quote(src), shell_quote(system_wallpaper)));
        cmd_parts.push(format!("chmod 644 {}", shell_quote(system_wallpaper)));
        new_settings.insert("background".to_string(), system_wallpaper.to_string());
    }

    let mut greeter: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    if fs.is_file(greeter_config)
        && let Some(content) = fs.read_text(greeter_config)
    {
    greeter = parse_ini(&content);
    }
    let g = greeter.entry("greeter".to_string()).or_default();
    for key in GREETER_KEYS {
        if let Some(val) = new_settings.get(*key) {
            if *key == "xft-dpi" && val.is_empty() {
                g.remove(*key);
            } else {
                g.insert((*key).to_string(), val.clone());
            }
        }
    }
    if !fs.write_text(tmp_greeter, &render_ini(&greeter)) {
        return false;
    }
    cmd_parts.push(format!("cp {} {}", shell_quote(tmp_greeter), shell_quote(greeter_config)));
    cmd_parts.push(format!("chmod 644 {}", shell_quote(greeter_config)));

    let mut daemon: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    if fs.is_file(daemon_config)
        && let Some(content) = fs.read_text(daemon_config)
    {
    daemon = parse_ini(&content);
    }
    let seat = daemon.entry("Seat:*".to_string()).or_default();
    for key in DAEMON_KEYS {
        if let Some(val) = new_settings.get(*key) {
            if val.is_empty() && (*key == "autologin-user" || *key == "autologin-session") {
                seat.remove(*key);
            } else {
                seat.insert((*key).to_string(), val.clone());
            }
        }
    }
    if !fs.write_text(tmp_daemon, &render_ini(&daemon)) {
        fs.remove(tmp_greeter);
        return false;
    }
    cmd_parts.push(format!("cp {} {}", shell_quote(tmp_daemon), shell_quote(daemon_config)));
    cmd_parts.push(format!("chmod 644 {}", shell_quote(daemon_config)));

    let full_cmd = cmd_parts.join(" && ");
    let out = runner.run("pkexec", &["sh", "-c", full_cmd.as_str()]);
    fs.remove(tmp_greeter);
    fs.remove(tmp_daemon);
    out.succeeded()
}

pub fn available_users(passwd_content: &str, current_user: &str) -> Vec<String> {
    let mut users = std::collections::BTreeSet::new();
    for line in passwd_content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            continue;
        }
        let username = parts[0];
        let uid: i64 = parts[2].parse().unwrap_or(-1);
        let shell = parts[6];
        if (1000..65534).contains(&uid)
            && username != "nobody"
            && ![
                "/bin/false",
                "/usr/bin/nologin",
                "/sbin/nologin",
                "/bin/sync",
            ]
            .contains(&shell)
        {
            users.insert(username.to_string());
        }
    }
    if !current_user.is_empty() && current_user != "nobody" {
        users.insert(current_user.to_string());
    }
    users.into_iter().collect()
}

pub fn get_available_users(fs: &dyn LightDmFs) -> Vec<String> {
    let content = fs.read_passwd().unwrap_or_default();
    available_users(&content, &fs.current_user())
}

pub fn get_available_sessions(fs: &dyn LightDmFs) -> Vec<String> {
    let mut sessions = std::collections::BTreeSet::new();
    for dir in ["/usr/share/wayland-sessions", "/usr/share/xsessions"] {
        if !fs.is_dir(dir) {
            continue;
        }
        if let Some(names) = fs.list_dir(dir) {
            for name in names {
                if let Some(stripped) = name.strip_suffix(".desktop") {
                    sessions.insert(stripped.to_string());
                }
            }
        }
    }
    let mut result: Vec<String> = sessions.into_iter().collect();
    result.sort();
    if let Some(pos) = result.iter().position(|s| s == "sway") {
        result.remove(pos);
        result.insert(0, "sway".to_string());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CmdOutput, FakeRunner};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// Mirrors `tests/test_lightdm_repository.py`.
    #[derive(Debug, Default)]
    struct MemFs {
        files: Arc<Mutex<HashMap<String, String>>>,
        dirs: HashMap<String, Vec<String>>,
        passwd: String,
        user: String,
    }

    impl MemFs {
        fn with_file(self, path: &str, content: &str) -> Self {
            self.files.lock().unwrap().insert(path.to_string(), content.to_string());
            self
        }
        fn with_dir(mut self, dir: &str, items: &[&str]) -> Self {
            self.dirs.insert(dir.to_string(), items.iter().map(|s| s.to_string()).collect());
            self
        }
    }

    #[test]
    fn save_settings_empty_keys_omitted_from_tmp_ini() {
        // are popped, never written.
        #[derive(Debug, Default)]
        struct CapFs {
            inner: MemFs,
            captured: std::sync::Arc<std::sync::Mutex<HashMap<String, String>>>,
        }
        impl LightDmFs for CapFs {
            fn is_file(&self, _p: &str) -> bool {
                false
            }
            fn read_text(&self, _p: &str) -> Option<String> {
                None
            }
            fn write_text(&self, path: &str, content: &str) -> bool {
                self.captured.lock().unwrap().insert(path.to_string(), content.to_string());
                self.inner.write_text(path, content)
            }
            fn remove(&self, _p: &str) {}
            fn path_exists(&self, _p: &str) -> bool {
                false
            }
            fn is_dir(&self, _p: &str) -> bool {
                false
            }
            fn list_dir(&self, _p: &str) -> Option<Vec<String>> {
                None
            }
            fn read_passwd(&self) -> Option<String> {
                None
            }
            fn current_user(&self) -> String {
                String::new()
            }
        }
        let fs = CapFs::default();
        let runner = RecRunner { inner: FakeRunner::new(), ..Default::default() };
        let settings = LightDmSettings {
            xft_dpi: String::new(),
            autologin_user: String::new(),
            autologin_session: String::new(),
            ..Default::default()
        };
        assert!(save_settings(
            &runner,
            &fs,
            &settings,
            None,
            "/g/greeter.conf",
            "/g/lightdm.conf",
            SYSTEM_WALLPAPER,
            "/g/greeter.tmp",
            "/g/daemon.tmp",
        ));
        let cap = fs.captured.lock().unwrap().clone();
        let greeter = cap.get("/g/greeter.tmp").cloned().unwrap_or_default();
        let daemon = cap.get("/g/daemon.tmp").cloned().unwrap_or_default();
        assert!(!greeter.contains("xft-dpi"), "greeter tmp:\n{greeter}");
        assert!(!daemon.contains("autologin-user ="), "daemon tmp:\n{daemon}");
        assert!(!daemon.contains("autologin-session ="), "daemon tmp:\n{daemon}");
        assert!(daemon.contains("user-session = sway"), "daemon tmp:\n{daemon}");
    }

    impl LightDmFs for MemFs {
        fn is_file(&self, path: &str) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }
        fn read_text(&self, path: &str) -> Option<String> {
            if path == "/etc/passwd" {
                return Some(self.passwd.clone());
            }
            self.files.lock().unwrap().get(path).cloned()
        }
        fn write_text(&self, path: &str, content: &str) -> bool {
            self.files.lock().unwrap().insert(path.to_string(), content.to_string());
            true
        }
        fn remove(&self, path: &str) {
            self.files.lock().unwrap().remove(path);
        }
        fn path_exists(&self, path: &str) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }
        fn is_dir(&self, path: &str) -> bool {
            self.dirs.contains_key(path)
        }
        fn list_dir(&self, path: &str) -> Option<Vec<String>> {
            self.dirs.get(path).cloned()
        }
        fn read_passwd(&self) -> Option<String> {
            Some(self.passwd.clone())
        }
        fn current_user(&self) -> String {
            self.user.clone()
        }
    }

    #[derive(Debug, Default)]
    struct RecRunner {
        inner: FakeRunner,
        calls: Arc<Mutex<Vec<Vec<String>>>>,
    }

    impl CommandRunner for RecRunner {
        fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
            let argv: Vec<String> =
                std::iter::once(program.to_string()).chain(args.iter().map(|s| s.to_string())).collect();
            self.calls.lock().unwrap().push(argv);
            CmdOutput::success("")
        }
        fn spawn_detached(&self, _program: &str, _args: &[&str]) {}
        fn which(&self, binary: &str) -> Option<String> {
            self.inner.which(binary)
        }
    }

    #[test]
    fn users_filter_nobody_root_daemon() {
        // Mirrors test_get_available_users_filters_nobody.
        let fs = MemFs {
            passwd: "root:x:0:0:root:/root:/bin/bash\n\
                daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin\n\
                guilherme:x:1000:1000:Guilherme,,,:/home/guilherme:/bin/bash\n\
                nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin\n"
                .to_string(),
            user: "guilherme".to_string(),
            ..Default::default()
        };
        let users = get_available_users(&fs);
        assert!(users.contains(&"guilherme".to_string()));
        assert!(!users.contains(&"nobody".to_string()));
        assert!(!users.contains(&"root".to_string()));
        assert!(!users.contains(&"daemon".to_string()));
    }

    #[test]
    fn sessions_sway_first() {
        // Mirrors test_get_available_sessions.
        let fs = MemFs::default()
            .with_dir("/usr/share/wayland-sessions", &["sway.desktop", "xfce.desktop"])
            .with_dir("/usr/share/xsessions", &["lxqt.desktop"]);
        let sessions = get_available_sessions(&fs);
        assert!(sessions.contains(&"sway".to_string()));
        assert!(sessions.contains(&"xfce".to_string()));
        assert!(sessions.contains(&"lxqt".to_string()));
        assert_eq!(sessions[0], "sway");
    }

    #[test]
    fn settings_from_conf_files() {
        // Mirrors test_get_settings_from_temp_files.
        let fs = MemFs::default()
            .with_file("/g/lightdm-gtk-greeter.conf", "[greeter]\ntheme-name = TestTheme\nclock-format = %H:%M\n")
            .with_file("/g/lightdm.conf", "[Seat:*]\nautologin-user = testuser\nuser-session = sway\n");
        let s = get_settings(&fs, "/g/lightdm-gtk-greeter.conf", "/g/lightdm.conf");
        assert_eq!(s.theme_name, "TestTheme");
        assert_eq!(s.clock_format, "%H:%M");
        assert_eq!(s.autologin_user, "testuser");
        assert_eq!(s.user_session, "sway");
    }

    #[test]
    fn save_settings_pkexec_argv_and_tmp_cleanup() {
        let fs = MemFs::default();
        let runner = RecRunner { inner: FakeRunner::new(), ..Default::default() };
        let settings = LightDmSettings { theme_name: "TestTheme".to_string(), xft_dpi: String::new(), ..Default::default() };
        assert!(save_settings(
            &runner,
            &fs,
            &settings,
            None,
            "/g/greeter.conf",
            "/g/lightdm.conf",
            SYSTEM_WALLPAPER,
            "/g/greeter.tmp",
            "/g/daemon.tmp",
        ));
        let calls = runner.calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0][0], "pkexec");
        assert_eq!(calls[0][1], "sh");
        assert_eq!(calls[0][2], "-c");
        let script = &calls[0][3];
        assert!(script.contains("mkdir -p /etc/lightdm"), "script: {script}");
        assert!(script.contains("cp /g/greeter.tmp /g/greeter.conf"), "script: {script}");
        assert!(script.contains("chmod 644 /g/greeter.conf"), "script: {script}");
        assert!(script.contains("cp /g/daemon.tmp /g/lightdm.conf"), "script: {script}");
        // Temps removed (Python finally-block parity).
        assert!(!fs.files.lock().unwrap().contains_key("/g/greeter.tmp"));
        assert!(!fs.files.lock().unwrap().contains_key("/g/daemon.tmp"));
    }
}
