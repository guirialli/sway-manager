//! `clipboard` deferred execution — history, copy, wipe, pin manager.
//!
//! `clipboard`/`clip` is `INTERACTIVE_COMMANDS`: the socket answers
//! immediately and this runs deferred (Python `QTimer.singleShot`). All
//! stdout literals match `manage_clipboard_use_case.py` and
//! `cliphist_repository.py` byte for byte.

use crate::process::CommandRunner;

pub const EMPTY_HISTORY_MSG: &str = "Nenhum item encontrado no histórico do clipboard.";
pub const EMPTY_PIN_MSG: &str = "Nenhum item disponível para fixar/desfixar.";

#[derive(Debug, Clone, PartialEq)]
pub struct ClipItem {
    pub id: String,
    pub text: String,
    pub raw_preview: String,
    pub is_image: bool,
    pub image_path: Option<String>,
    pub is_favorite: bool,
    pub is_action: bool,
    pub action_type: Option<String>,
}

impl ClipItem {
    pub fn history(id: &str, summary: &str) -> Self {
        let is_img = summary.trim().starts_with("[[ binary data");
        Self {
            id: id.to_string(),
            text: if is_img {
                format!("[Imagem] {}", summary.trim())
            } else {
                summary.to_string()
            },
            raw_preview: summary.to_string(),
            is_image: is_img,
            image_path: None,
            is_favorite: false,
            is_action: false,
            action_type: None,
        }
    }

    pub fn action_clear() -> Self {
        Self {
            id: "action_clear".to_string(),
            text: "🧹 [Limpar Histórico do Clipboard]".to_string(),
            raw_preview: "🧹 Limpar Histórico do Clipboard".to_string(),
            is_image: false,
            image_path: None,
            is_favorite: false,
            is_action: true,
            action_type: Some("clear".to_string()),
        }
    }

    pub fn action_manage_favs() -> Self {
        Self {
            id: "action_manage_favs".to_string(),
            text: "📌 [Gerenciar / Fixar Favoritos]".to_string(),
            raw_preview: "📌 Gerenciar / Fixar Favoritos".to_string(),
            is_image: false,
            image_path: None,
            is_favorite: false,
            is_action: true,
            action_type: Some("manage_favorites".to_string()),
        }
    }
}

/// Mirrors `get_clipboard_items` list assembly (actions + cliphist list,
/// favorites excluded from history to avoid duplicates).
pub fn list_items(
    runner: &dyn CommandRunner,
    favorites: Vec<ClipItem>,
    decode_hint: &dyn Fn(&str) -> bool,
) -> Vec<ClipItem> {
    let mut items = vec![ClipItem::action_clear(), ClipItem::action_manage_favs()];
    items.extend(favorites.clone());
    if runner.which("cliphist").is_none() {
        return items;
    }
    let output = runner.run("cliphist", &["list"]);
    if !output.succeeded() || output.stdout.trim().is_empty() {
        return items;
    }
    let fav_texts: Vec<&str> = favorites.iter().map(|fav| fav.text.as_str()).collect();
    for line in output.stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let id = parts.next().unwrap_or("").trim().to_string();
        let summary = parts.next().unwrap_or("").to_string();
        if fav_texts.iter().any(|text| **text == summary) {
            continue;
        }
        let mut item = ClipItem::history(&id, &summary);
        if item.is_image {
            item.image_path = decode_hint(&id).then(|| format!("/tmp/clip_{id}.png"));
        }
        items.push(item);
    }
    items
}

/// Mirrors `copy_to_clipboard` (wl-copy variants + notify + literals).
pub fn copy_item(runner: &dyn CommandRunner, item: &ClipItem, decode: &dyn Fn(&str) -> Option<Vec<u8>>) -> bool {
    if runner.which("wl-copy").is_none() {
        println!("Erro: 'wl-copy' não encontrado.");
        return false;
    }
    let payload: Option<Vec<u8>> = if item.is_favorite {
        Some(item.text.as_bytes().to_vec())
    } else {
        decode(&item.id).or_else(|| Some(item.raw_preview.as_bytes().to_vec()))
    };
    let Some(bytes) = payload else {
        println!("Erro ao copiar para a área de transferência: sem dados");
        return false;
    };
    let output = runner.run_with_input("wl-copy", &[], &String::from_utf8_lossy(&bytes));
    if !output.succeeded() {
        println!("Erro ao copiar para a área de transferência: {}", output.stderr.trim());
        return false;
    }
    runner.run("notify-send", &["-a", "SwayManager", "Clipboard", "Copiado para a área de transferência!"]);
    true
}

/// Mirrors `clear_history` (wipe + thumb cache purge + notify).
pub fn clear_history(runner: &dyn CommandRunner, cache_dir: &str) -> bool {
    if runner.which("cliphist").is_some() {
        let output = runner.run("cliphist", &["wipe"]);
        if !output.succeeded() {
            println!("Erro ao limpar histórico do clipboard: {}", output.stderr.trim());
            return false;
        }
    }
    if let Ok(entries) = std::fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    runner.run("notify-send", &["-a", "SwayManager", "Clipboard", "Histórico do clipboard limpo com sucesso!"]);
    true
}

/// Mirrors `ManageClipboardUseCase.execute` routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutedAction {
    Clear,
    PinManager,
    Menu,
}

pub fn route_action(action: Option<&str>) -> RoutedAction {
    match action {
        Some("clear") => RoutedAction::Clear,
        Some("pin") | Some("favorites") => RoutedAction::PinManager,
        _ => RoutedAction::Menu,
    }
}

/// Line format for one clipboard item (mirrors `launch_menu` lines 354-369).
pub fn clip_line(item: &ClipItem, icon: &str) -> String {
    let escaped = html_escape(&item.text);
    if item.is_action {
        format!("img:{icon}:text:<b>{escaped}</b>")
    } else if item.is_favorite {
        format!("img:{icon}:text:📌 <b>[Favorito]</b> {escaped}")
    } else {
        format!("img:{icon}:text:{escaped}")
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Present `items` in wofi and return the chosen item (mirrors
/// `CliphistRepository.launch_menu` lines 341-409).
pub fn launch_clipboard_menu(
    runner: &dyn CommandRunner,
    home: &str,
    icon: &dyn Fn(&str) -> String,
    items: &[ClipItem],
    prompt: &str,
) -> Option<ClipItem> {
    if items.is_empty() {
        return None;
    }
    let copy_icon = icon("edit-copy");
    let clear_icon = icon("edit-clear");
    let fav_icon = icon("emblem-favorite");
    let mut line_to_index = std::collections::HashMap::new();
    let mut lines = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let icon_path = if item.is_action {
            if item.action_type.as_deref() == Some("clear") {
                &clear_icon
            } else {
                &fav_icon
            }
        } else if item.is_favorite {
            item.image_path.as_deref().unwrap_or(&fav_icon)
        } else {
            item.image_path.as_deref().unwrap_or(&copy_icon)
        };
        let line = clip_line(item, icon_path);
        line_to_index.insert(line.clone(), index);
        lines.push(line);
    }
    let payload = lines.join("\n");
    let mut cmd = vec![
        "wofi".to_string(),
        "--dmenu".to_string(),
        "--allow-images".to_string(),
        "--allow-markup".to_string(),
        "--insensitive".to_string(),
        format!("--prompt={prompt}"),
    ];
    if std::path::Path::new(&format!("{home}/.config/wofi/config")).exists() {
        cmd.push("--conf".to_string());
        cmd.push(format!("{home}/.config/wofi/config"));
    }
    if std::path::Path::new(&format!("{home}/.config/wofi/style.css")).exists() {
        cmd.push("--style".to_string());
        cmd.push(format!("{home}/.config/wofi/style.css"));
    }
    let (program, rest) = cmd.split_first().expect("wofi argv non-empty");
    let args: Vec<&str> = rest.iter().map(String::as_str).collect();
    let output = runner.run_with_input(program, &args, &payload);
    if !output.succeeded() || output.stdout.trim().is_empty() {
        return None;
    }
    let selected = output.stdout.trim().to_string();
    line_to_index.get(&selected).map(|index| items[*index].clone())
}

/// Full interactive flow (menu + pin manager + copy).
/// `decode` reads raw bytes for an id (`cliphist decode`).
/// Returns `true` when an action completed.
pub fn execute(
    runner: &dyn CommandRunner,
    home: &str,
    icon: &dyn Fn(&str) -> String,
    list: &dyn Fn() -> Vec<ClipItem>,
    decode: &dyn Fn(&str) -> Option<Vec<u8>>,
    select: &dyn Fn(&[ClipItem], &str) -> Option<ClipItem>,
    action: Option<&str>,
) -> bool {
    super::clipboard::ensure_daemon_running(runner);
    match route_action(action) {
        RoutedAction::Clear => clear_history(runner, &format!("{home}/.cache/sway-manager/clipboard_thumbs")),
        RoutedAction::PinManager => pin_manager(runner, home, icon, list, select),
        RoutedAction::Menu => show_menu(runner, home, icon, list, decode, select),
    }
}

fn show_menu(
    runner: &dyn CommandRunner,
    home: &str,
    icon: &dyn Fn(&str) -> String,
    list: &dyn Fn() -> Vec<ClipItem>,
    decode: &dyn Fn(&str) -> Option<Vec<u8>>,
    select: &dyn Fn(&[ClipItem], &str) -> Option<ClipItem>,
) -> bool {
    let items = list();
    if items.iter().all(|item| item.is_action) && items.len() <= 2 {
        // Only the two injected actions: history empty (or cliphist
        // missing with no favorites) — matches "Nenhum item..." when
        // there is nothing to show.
        let has_content = items.iter().any(|item| !item.is_action);
        if !has_content {
            println!("{EMPTY_HISTORY_MSG}");
            return false;
        }
    }
    let Some(selected) = select(&items, "Clipboard...") else {
        return false;
    };
    if selected.is_action {
        return match selected.action_type.as_deref() {
            Some("clear") => clear_history(runner, &format!("{home}/.cache/sway-manager/clipboard_thumbs")),
            Some("manage_favorites") => pin_manager(runner, home, icon, list, select),
            _ => false,
        };
    }
    copy_item(runner, &selected, decode)
}

/// Mirrors `_handle_pin_manager` (pin/unpin items + literals).
pub fn pin_manager(
    runner: &dyn CommandRunner,
    home: &str,
    icon: &dyn Fn(&str) -> String,
    list: &dyn Fn() -> Vec<ClipItem>,
    select: &dyn Fn(&[ClipItem], &str) -> Option<ClipItem>,
) -> bool {
    let items = list();
    let mut pin_items = Vec::new();
    for item in &items {
        if item.is_action {
            continue;
        }
        if item.is_favorite {
            pin_items.push(ClipItem {
                text: format!("❌ [Remover dos Favoritos] {}", item.text),
                is_action: true,
                action_type: Some("unpin".to_string()),
                ..item.clone()
            });
        } else {
            pin_items.push(ClipItem {
                text: format!("📌 [Fixar nos Favoritos] {}", item.text),
                is_action: true,
                action_type: Some("pin".to_string()),
                is_favorite: false,
                ..item.clone()
            });
        }
    }
    if pin_items.is_empty() {
        println!("{EMPTY_PIN_MSG}");
        return false;
    }
    let Some(selected) = select(&pin_items, "Gerenciar Favoritos...") else {
        return false;
    };
    let icons = icons_for(runner, home, icon);
    match selected.action_type.as_deref() {
        Some("pin") => {
            let clean = selected.text.replace("📌 [Fixar nos Favoritos] ", "");
            add_favorite(runner, home, &selected.id, &clean, &selected.raw_preview, selected.is_image, icons.fav)
        }
        Some("unpin") => remove_favorite(home, &selected.id),
        _ => false,
    }
}

struct ClipIcons {
    fav: String,
}

fn icons_for(runner: &dyn CommandRunner, home: &str, icon: &dyn Fn(&str) -> String) -> ClipIcons {
    let _ = (runner, home);
    ClipIcons { fav: icon("emblem-favorite") }
}

fn favorites_path(home: &str) -> String {
    format!("{home}/.config/sway-manager/clipboard_favorites.json")
}

/// Load persisted favorites (`clipboard_favorites.json`).
pub fn load_favorites_pub(home: &str) -> Vec<ClipItem> {
    load_favorites(home)
}

fn load_favorites(home: &str) -> Vec<ClipItem> {
    let Ok(content) = std::fs::read_to_string(favorites_path(home)) else {
        return Vec::new();
    };
    let Ok(entries): Result<Vec<serde_json::Value>, _> = serde_json::from_str(&content) else {
        println!("Erro ao carregar favoritos: formato inválido");
        return Vec::new();
    };
    entries
        .into_iter()
        .map(|entry| ClipItem {
            id: entry["id"].as_str().unwrap_or("").to_string(),
            text: entry["text"].as_str().unwrap_or("").to_string(),
            raw_preview: entry["raw_preview"].as_str().unwrap_or("").to_string(),
            is_image: entry["is_image"].as_bool().unwrap_or(false),
            image_path: entry["image_path"].as_str().map(|path| path.to_string()),
            is_favorite: true,
            is_action: false,
            action_type: None,
        })
        .collect()
}

fn save_favorites(home: &str, favorites: &[ClipItem]) -> bool {
    let entries: Vec<serde_json::Value> = favorites
        .iter()
        .map(|fav| {
            serde_json::json!({
                "id": fav.id,
                "text": fav.text,
                "raw_preview": fav.raw_preview,
                "is_image": fav.is_image,
                "image_path": fav.image_path,
            })
        })
        .collect();
    let path = favorites_path(home);
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&entries) {
        Ok(json) => std::fs::write(&path, format!("{json}\n")).is_ok(),
        Err(err) => {
            println!("Erro ao salvar favoritos: {err}");
            false
        }
    }
}

fn add_favorite(
    runner: &dyn CommandRunner,
    home: &str,
    item_id: &str,
    text: &str,
    raw_preview: &str,
    is_image: bool,
    _fav_icon: String,
) -> bool {
    let favorites = load_favorites(home);
    if favorites.iter().any(|fav| fav.id == item_id || (!fav.is_image && fav.text == text)) {
        return true;
    }
    let fav_id = format!("fav_{}_{}", favorites.len() + 1, (raw_preview.len() * 31 + 7) & 0xffffff);
    let new_fav = ClipItem {
        id: fav_id,
        text: text.to_string(),
        raw_preview: raw_preview.to_string(),
        is_image,
        image_path: None,
        is_favorite: true,
        is_action: false,
        action_type: None,
    };
    let mut favorites = favorites;
    favorites.push(new_fav);
    if !save_favorites(home, &favorites) {
        return false;
    }
    runner.run("notify-send", &["-a", "SwayManager", "Clipboard", "Item fixado nos favoritos!"]);
    true
}

fn remove_favorite(home: &str, item_id: &str) -> bool {
    let favorites = load_favorites(home);
    let updated: Vec<ClipItem> = favorites
        .into_iter()
        .filter(|fav| fav.id != item_id && fav.text != item_id)
        .collect();
    if !save_favorites(home, &updated) {
        return false;
    }
    // notify-send best-effort (mirrors `_notify` swallowing errors).
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "SwayManager", "Clipboard", "Item removido dos favoritos."])
        .output();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    #[test]
    fn history_parsing_skips_favorites() {
        let runner = FakeRunner::new()
            .with_which("cliphist", Some("/usr/bin/cliphist"))
            .with_success("cliphist", &["list"], "12\thello\n13\t[[ binary data image/png ]\n");
        let favs = vec![ClipItem { id: "1".to_string(), text: "hello".to_string(), raw_preview: "hello".to_string(), is_image: false, image_path: None, is_favorite: true, is_action: false, action_type: None }];
        let items = list_items(&runner, favs, &|_| true);
        // 2 actions + 1 fav + 1 new (binary) item; "hello" dup skipped.
        assert_eq!(items.len(), 4);
        assert!(items.iter().any(|item| item.is_image));
    }

    #[test]
    fn route_action_mapping() {
        assert_eq!(route_action(Some("clear")), RoutedAction::Clear);
        assert_eq!(route_action(Some("pin")), RoutedAction::PinManager);
        assert_eq!(route_action(Some("favorites")), RoutedAction::PinManager);
        assert_eq!(route_action(None), RoutedAction::Menu);
    }

    #[test]
    fn action_item_literals() {
        assert_eq!(ClipItem::action_clear().id, "action_clear");
        assert_eq!(ClipItem::action_manage_favs().id, "action_manage_favs");
    }

    #[test]
    fn copy_without_wlcopy_warns() {
        let runner = FakeRunner::new();
        let item = ClipItem::history("1", "hi");
        assert!(!copy_item(&runner, &item, &|_| None));
    }

    #[test]
    fn regression_clipboard_wofi_payload_never_has_empty_icon() {
        // Same launcher-without-icons guard for the clipboard menu:
        // actions, favorites and history lines all carry a real icon.
        let dir = tempfile::tempdir().unwrap();
        let icon = dir.path().join("edit-copy.png");
        std::fs::write(&icon, "png").unwrap();
        let icon_path = icon.to_string_lossy().into_owned();
        let argv = ["wofi", "--dmenu", "--allow-images", "--allow-markup", "--insensitive", "--prompt=Clipboard..."];
        let items = vec![
            ClipItem::action_clear(),
            ClipItem {
                text: "hello".to_string(),
                raw_preview: "hello".to_string(),
                is_favorite: true,
                ..ClipItem::history("7", "hello")
            },
            ClipItem::history("8", "world"),
        ];
        let first_line = clip_line(&items[0], &icon_path);
        let runner = FakeRunner::new().with_success("wofi", &argv[1..], &format!("{first_line}\n"));
        let icon_ref = icon_path.clone();
        let icon_fn = move |_: &str| icon_ref.clone();
        let selected = launch_clipboard_menu(&runner, "/home/u", &icon_fn, &items, "Clipboard...");
        assert_eq!(selected.map(|item| item.id), Some("action_clear".to_string()));
        let payloads = runner.piped_inputs("wofi", &argv[1..]);
        assert_eq!(payloads.len(), 1, "wofi must receive exactly one payload");
        assert_eq!(payloads[0].lines().count(), 3);
        for line in payloads[0].lines() {
            let path = line.strip_prefix("img:").and_then(|rest| rest.split(":text:").next()).unwrap_or("");
            assert!(!path.is_empty(), "empty icon in line: {line}");
            assert!(std::path::Path::new(path).exists(), "missing icon file: {path}");
        }
    }

    #[test]
    fn execute_menu_empty_history_prints_literal() {
        let runner = FakeRunner::new()
            .with_which("cliphist", None)
            .with_which("wl-paste", None);
        let icon = |_: &str| String::new();
        let list = || vec![ClipItem::action_clear(), ClipItem::action_manage_favs()];
        let decode = |_: &str| None;
        let select = |_: &[ClipItem], _: &str| None;
        assert!(!execute(&runner, "/home/u", &icon, &list, &decode, &select, None));
    }

    #[test]
    fn execute_menu_copy_and_clear_paths() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().to_str().unwrap().to_string();
        let item = ClipItem::history("5", "copied text");
        let runner = FakeRunner::new()
            .with_which("cliphist", Some("/usr/bin/cliphist"))
            .with_which("wl-paste", Some("/usr/bin/wl-paste"))
            .with_which("wl-copy", Some("/usr/bin/wl-copy"))
            .with_success("wl-copy", &[], "")
            .with_success("cliphist", &["wipe"], "")
            .with_success("notify-send", &[], "");
        let icon = |_: &str| String::new();
        let list = || vec![ClipItem::action_clear(), item.clone()];
        let decode = |_: &str| Some(b"raw".to_vec());
        let select_copy = |_: &[ClipItem], _: &str| Some(item.clone());
        assert!(execute(&runner, &home, &icon, &list, &decode, &select_copy, None));
        let select_clear = |_: &[ClipItem], _: &str| Some(ClipItem::action_clear());
        assert!(execute(&runner, &home, &icon, &list, &decode, &select_clear, None));
    }

    #[test]
    fn pin_manager_empty_and_cancel() {
        let runner = FakeRunner::new();
        let icon = |_: &str| String::new();
        let empty: Vec<ClipItem> = vec![ClipItem::action_clear()];
        let list_empty = || empty.clone();
        let select_none = |_: &[ClipItem], _: &str| None;
        assert!(!pin_manager(&runner, "/home/u", &icon, &list_empty, &select_none));
        // Cancel on non-empty list returns false without side effects.
        let items = vec![ClipItem::history("1", "a")];
        let list = || items.clone();
        assert!(!pin_manager(&runner, "/home/u", &icon, &list, &select_none));
    }

    #[test]
    fn pin_and_unpin_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().to_str().unwrap().to_string();
        let runner = FakeRunner::new().with_success("notify-send", &[], "");
        let icon = |_: &str| String::new();
        let items = vec![ClipItem::history("9", "pin me")];
        let list = || items.clone();
        let select_pin = |pin_items: &[ClipItem], _: &str| {
            pin_items.iter().find(|item| item.action_type.as_deref() == Some("pin")).cloned()
        };
        assert!(pin_manager(&runner, &home, &icon, &list, &select_pin));
        let favs = load_favorites_pub(&home);
        assert_eq!(favs.len(), 1);
        // Duplicate pin is a no-op success.
        assert!(pin_manager(&runner, &home, &icon, &list, &select_pin));
        assert_eq!(load_favorites_pub(&home).len(), 1);
        let select_unpin = |pin_items: &[ClipItem], _: &str| {
            pin_items.iter().find(|item| item.action_type.as_deref() == Some("unpin")).cloned()
        };
        // After pinning, the item is a favorite → unpin entry appears.
        let list_fav = || load_favorites_pub(&home);
        assert!(pin_manager(&runner, &home, &icon, &list_fav, &select_unpin));
        assert!(load_favorites_pub(&home).is_empty());
    }

    #[test]
    fn clip_line_formats_per_kind() {
        let fav = ClipItem { is_favorite: true, ..ClipItem::history("1", "a & b") };
        assert_eq!(clip_line(&fav, "/i.png"), "img:/i.png:text:📌 <b>[Favorito]</b> a &amp; b");
        assert_eq!(clip_line(&ClipItem::action_clear(), "/c.png"), "img:/c.png:text:<b>🧹 [Limpar Histórico do Clipboard]</b>");
        let plain = ClipItem::history("2", "x");
        assert_eq!(clip_line(&plain, "/p.png"), "img:/p.png:text:x");
    }
}
