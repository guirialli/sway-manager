//! `menu` / `reload-cache` — .desktop index + wofi contract.
//!
//! The interactive menu runs in-process in Python (`handle_menu` is an
//! `INTERACTIVE_COMMANDS` deferred dispatch). This port owns the
//! headless contract: search directories, field-code cleaning, category
//! filter semantics (`/a` apps vs `/s` system), wofi argv, and the
//! use-case literals/empty-menu message.

//! (no process import: pure contract)

pub const SEARCH_DIRS: &[&str] = &[
    ".local/share/applications",
    "/usr/local/share/applications",
    "/usr/share/applications",
    "/var/lib/snap/desktop/applications",
    ".local/share/flatpak/exports/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];

pub const SYSTEM_ACTIONS: &[(&str, &str, &str)] = &[
    ("Bloquear Tela", "swaylock -f -c 000000", "system-lock-screen"),
    ("Encerrar Sessão", "swaymsg exit", "system-log-out"),
    ("Suspender", "systemctl suspend", "system-suspend"),
    ("Reiniciar", "systemctl reboot", "system-reboot"),
    ("Desligar", "systemctl poweroff", "system-shutdown"),
];

pub const EMPTY_MENU_MSG: &str = "Nenhum item encontrado no menu.";
pub const RELOAD_OK_PREFIX: &str = "Cache do Wofi recarregado com sucesso";

/// Mirrors `domain/menu/entities.py::MenuItem`.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    pub name: String,
    pub normalized_name: String,
    pub exec_cmd: String,
    pub icon: String,
    pub category: String,
    pub is_system_action: bool,
    pub is_category_header: bool,
    pub category_target: Option<String>,
    pub is_web_app: bool,
}

impl MenuItem {
    pub fn app(name: &str, exec_cmd: &str, icon: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            normalized_name: name.to_string(),
            exec_cmd: exec_cmd.to_string(),
            icon: icon.to_string(),
            category: category.to_string(),
            is_system_action: false,
            is_category_header: false,
            category_target: None,
            is_web_app: false,
        }
    }
}

pub fn search_directories(home: &str) -> Vec<String> {
    SEARCH_DIRS
        .iter()
        .map(|dir| {
            if dir.starts_with('/') {
                dir.to_string()
            } else {
                format!("{home}/{dir}")
            }
        })
        .collect()
}

/// Mirrors `DesktopParser.clean_exec_cmd` (`%u %U %f %F ...` stripping).
pub fn clean_exec_cmd(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars();
    while let Some(char) = chars.next() {
        if char == '%' {
            chars.next();
        } else {
            out.push(char);
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn menu_prompt(category_filter: Option<&str>) -> String {
    match category_filter {
        Some(filter) => format!("Buscar em [{filter}]..."),
        None => "Buscar aplicativo ou comando...".to_string(),
    }
}

/// Base wofi argv; `--conf`/`--style` appended when present (lines 118-121).
pub fn wofi_command(prompt: &str, home: &str, fs_has_config: bool, fs_has_style: bool) -> Vec<String> {
    let mut cmd = vec![
        "wofi".to_string(),
        "--dmenu".to_string(),
        "--allow-images".to_string(),
        "--allow-markup".to_string(),
        "--insensitive".to_string(),
        "--no-actions".to_string(),
        format!("--prompt={prompt}"),
    ];
    if fs_has_config {
        cmd.extend(["--conf".to_string(), format!("{home}/.config/wofi/config")]);
    }
    if fs_has_style {
        cmd.extend(["--style".to_string(), format!("{home}/.config/wofi/style.css")]);
    }
    cmd
}

/// Mirrors `handle_cli_command` reload-cache branch (server/cli.py:53-58).
pub fn reload_cache_message(item_count: usize) -> String {
    format!("{RELOAD_OK_PREFIX} ({item_count} itens).")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wofi_argv_base() {
        let cmd = wofi_command("Buscar...", "/home/u", false, false);
        assert_eq!(cmd[0], "wofi");
        assert!(cmd.contains(&"--dmenu".to_string()));
        assert!(cmd.iter().any(|arg| arg.starts_with("--prompt=")));
    }

    #[test]
    fn prompts_and_search_dirs() {
        assert_eq!(menu_prompt(None), "Buscar aplicativo ou comando...");
        assert_eq!(menu_prompt(Some("/a")), "Buscar em [/a]...");
        let dirs = search_directories("/home/u");
        assert!(dirs.contains(&"/usr/share/applications".to_string()));
        assert!(dirs.contains(&"/home/u/.local/share/applications".to_string()));
        assert_eq!(SYSTEM_ACTIONS.len(), 5);
    }

}
