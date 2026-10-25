//! `menu` deferred execution — .desktop index, filter, wofi round-trip.
//!
//! `handle_menu` is `INTERACTIVE_COMMANDS`: the socket answers immediately
//! and this runs deferred (Python: `QTimer.singleShot(0, ...)`). All
//! stdout literals (`Nenhum item...`, `Erro ao executar Wofi...`) match
//! `show_menu_use_case.py` and `wofi_launcher.py` byte for byte.

use crate::process::CommandRunner;
pub use crate::domains::menus::menu::MenuItem;

/// Lowercase accent-stripped key (mirrors `strip_accents`: NFKD + drop
/// combining marks + lowercase). Rust std has no NFKD; this folds the
/// Latin-1 range explicitly, which covers the desktop-file names in practice.
pub fn strip_accents(text: &str) -> String {
    text.chars()
        .flat_map(|char| {
            match char {
                'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => vec!['a'],
                'Á' | 'À' | 'Â' | 'Ã' | 'Ä' | 'Å' => vec!['a'],
                'é' | 'è' | 'ê' | 'ë' => vec!['e'],
                'É' | 'È' | 'Ê' | 'Ë' => vec!['e'],
                'í' | 'ì' | 'î' | 'ï' => vec!['i'],
                'Í' | 'Ì' | 'Î' | 'Ï' => vec!['i'],
                'ó' | 'ò' | 'ô' | 'õ' | 'ö' => vec!['o'],
                'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => vec!['o'],
                'ú' | 'ù' | 'û' | 'ü' => vec!['u'],
                'Ú' | 'Ù' | 'Û' | 'Ü' => vec!['u'],
                'ç' => vec!['c'],
                'Ç' => vec!['c'],
                'ñ' => vec!['n'],
                'Ñ' => vec!['n'],
                other => vec![other.to_lowercase().next().unwrap_or(other)],
            }
            .into_iter()
        })
        .collect()
}

/// Mirrors `DesktopParser.clean_exec_cmd` (field codes `%u %U %f %F...`).
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

/// Mirrors `WofiRepository._generate_line_key` (html-escaped name).
pub fn line_key(item: &MenuItem, icon_path: &str) -> String {
    let escaped = html_escape(&item.normalized_name);
    if item.is_system_action {
        format!(
            "img:{icon_path}:text:<b>{escaped}</b> <span alpha=\"1%\">/s /sessao /sistema /s {escaped} /s{escaped}</span>"
        )
    } else {
        format!(
            "img:{icon_path}:text:<b>{escaped}</b> <span alpha=\"1%\">/a /app /programa /a {escaped} /a{escaped}</span>"
        )
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Category filter semantics from `get_menu_items` (lines 45-90):
/// `/a` = apps only, `/s` = system only, else accent-insensitive search.
pub fn filter_items<'a>(items: &'a [MenuItem], filter: Option<&str>) -> Vec<&'a MenuItem> {
    let Some(filter) = filter.filter(|filter| !filter.is_empty()) else {
        return items.iter().collect();
    };
    let lower = filter.to_lowercase();
    if lower == "/a" || lower.starts_with("/a ") {
        let query = strip_accents(filter[2..].trim());
        return items
            .iter()
            .filter(|item| {
                !item.is_system_action
                    && (query.is_empty() || strip_accents(&item.normalized_name).contains(&query))
            })
            .collect();
    }
    if lower == "/s" || lower.starts_with("/s ") || lower.starts_with("/s") {
        let query = strip_accents(filter[2..].trim());
        return items
            .iter()
            .filter(|item| {
                item.is_system_action
                    && (query.is_empty() || strip_accents(&item.normalized_name).contains(&query))
            })
            .collect();
    }
    let query = strip_accents(&lower);
    items
        .iter()
        .filter(|item| strip_accents(&item.normalized_name).contains(&query))
        .collect()
}

pub fn menu_prompt(category_filter: Option<&str>) -> String {
    match category_filter {
        Some(filter) => format!("Buscar em [{filter}]..."),
        None => "Buscar aplicativo ou comando...".to_string(),
    }
}

/// Deferred menu execution. Returns `false` with the Python literal when
/// the list is empty; launches wofi synchronously otherwise.
pub fn execute_menu(
    runner: &dyn CommandRunner,
    items: &[MenuItem],
    category_filter: Option<&str>,
    icon_path: &dyn Fn(&str) -> String,
    wofi_home: &str,
    home_has_config: bool,
    home_has_style: bool,
) -> bool {
    let filtered = filter_items(items, category_filter);
    if filtered.is_empty() {
        println!("Nenhum item encontrado no menu.");
        return false;
    }
    let prompt = menu_prompt(category_filter);
    let lines: Vec<String> = filtered.iter().map(|item| line_key(item, &icon_path(&item.icon))).collect();
    let payload = lines.join("\n");
    let mut cmd = vec![
        "wofi".to_string(),
        "--dmenu".to_string(),
        "--allow-images".to_string(),
        "--allow-markup".to_string(),
        "--insensitive".to_string(),
        "--no-actions".to_string(),
        format!("--prompt={prompt}"),
    ];
    if home_has_config {
        cmd.push("--conf".to_string());
        cmd.push(format!("{wofi_home}/.config/wofi/config"));
    }
    if home_has_style {
        cmd.push("--style".to_string());
        cmd.push(format!("{wofi_home}/.config/wofi/style.css"));
    }
    let selected = match run_wofi(runner, &cmd, &payload) {
        Ok(selected) => selected,
        Err(err) => {
            println!("Erro ao executar Wofi: {err}");
            return false;
        }
    };
    let Some(selected) = selected else {
        return false;
    };
    let Some(item) = filtered.iter().find(|item| line_key(item, &icon_path(&item.icon)) == selected) else {
        return false;
    };
    if item.exec_cmd.is_empty() {
        return false;
    }
    runner.spawn_detached("sh", &["-c", &item.exec_cmd]);
    true
}

fn run_wofi(runner: &dyn CommandRunner, cmd: &[String], payload: &str) -> Result<Option<String>, String> {
    let (program, rest) = cmd.split_first().expect("wofi argv non-empty");
    let args: Vec<&str> = rest.iter().map(String::as_str).collect();
    // `Popen(communicate(input=payload))`: the fake matches on argv and
    // returns the canned selection; the system runner pipes stdin.
    let output = runner.run_with_input(program, &args, payload);
    if output.status != 0 || output.stdout.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(output.stdout.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    fn app(name: &str) -> MenuItem {
        MenuItem::app(name, name.to_lowercase().as_str(), "icon", "Utilitários")
    }

    #[test]
    fn filters_match_python_semantics() {
        let sys = MenuItem {
            is_system_action: true,
            ..app("Bloquear Tela")
        };
        let items = vec![app("Firefox"), sys];
        assert_eq!(filter_items(&items, None).len(), 2);
        assert_eq!(filter_items(&items, Some("/a")).len(), 1);
        assert_eq!(filter_items(&items, Some("/s")).len(), 1);
        assert_eq!(filter_items(&items, Some("fire")).len(), 1);
        assert_eq!(filter_items(&items, Some("BLOQUEAR")).len(), 1);
        assert!(filter_items(&items, Some("inexistente")).is_empty());
    }

    #[test]
    fn empty_menu_prints_literal() {
        let runner = FakeRunner::new();
        assert!(!execute_menu(&runner, &[], None, &|_| String::new(), "/home/u", false, false));
    }

    #[test]
    fn strip_accents_case_insensitive() {
        assert_eq!(strip_accents("BLOQUEAR Téla"), "bloquear tela");
    }

    #[test]
    fn line_key_formats() {
        let key = line_key(&app("Firefox"), "/i/firefox.png");
        assert!(key.starts_with("img:/i/firefox.png:text:<b>Firefox</b>"));
        assert!(key.contains("/a /app /programa"));
    }

    #[test]
    fn regression_wofi_payload_never_has_empty_icon() {
        // Guards the launcher-without-icons regression: every payload
        // line must carry `img:<existing-path>:text:`, never `img::text:`.
        let dir = tempfile::tempdir().unwrap();
        let icon = dir.path().join("firefox.png");
        std::fs::write(&icon, "png").unwrap();
        let icon_path = icon.to_string_lossy().into_owned();
        let icon_ref = icon_path.clone();
        let items = vec![app("Firefox")];
        let selected_line = line_key(&items[0], &icon_path);
        let runner = FakeRunner::new().with_success(
            "wofi",
            &["--dmenu", "--allow-images", "--allow-markup", "--insensitive", "--no-actions", "--prompt=Buscar aplicativo ou comando..."],
            &format!("{selected_line}\n"),
        );
        let icon_fn = move |_: &str| icon_ref.clone();
        assert!(execute_menu(&runner, &items, None, &icon_fn, "/home/u", false, false));
        let payloads = runner.piped_inputs(
            "wofi",
            &["--dmenu", "--allow-images", "--allow-markup", "--insensitive", "--no-actions", "--prompt=Buscar aplicativo ou comando..."],
        );
        assert_eq!(payloads.len(), 1, "wofi must receive exactly one payload");
        for line in payloads[0].lines() {
            let path = line.strip_prefix("img:").and_then(|rest| rest.split(":text:").next()).unwrap_or("");
            assert!(!path.is_empty(), "empty icon in line: {line}");
            assert!(std::path::Path::new(path).exists(), "missing icon file: {path}");
        }
    }
}
