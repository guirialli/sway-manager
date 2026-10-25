//! JSON config load/save/get/set.
//!
//! Mirrors `JsonConfigRepository`
//! (`src/infrastructure/config/json_config_repository.py:22-63`).
//! Default path parity: explicit path else
//! `~/.config/sway-manager/config.json`. (The Python test-mode
//! `/tmp/sway_manager_tests/test_config.json` branch is intentionally
//! not ported: daemon callers always pass an explicit path.)

use serde_json::Value;
use std::collections::BTreeMap;

pub fn default_config_path(home: &str) -> String {
    format!("{home}/.config/sway-manager/config.json")
}

/// Mirrors `load_config`: missing file / bad JSON / non-dict → `{}`.
pub fn load_config(config_path: &str) -> BTreeMap<String, Value> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return BTreeMap::new(),
    };
    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(map)) => map.into_iter().collect(),
        _ => BTreeMap::new(),
    }
}

/// Mirrors `save_config`: makedirs + `json.dump(indent=2,
/// ensure_ascii=False)` + trailing newline. Byte parity: Python writes
/// `json.dump(..., indent=2, ensure_ascii=False)` then `"\n"`;
/// `serde_json::to_string_pretty` matches that layout for plain data.
pub fn save_config(config_path: &str, data: &BTreeMap<String, Value>) -> bool {
    if let Some(parent) = std::path::Path::new(config_path).parent()
        && !parent.as_os_str().is_empty()
        && std::fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    let mut text = match serde_json::to_string_pretty(data) {
        Ok(t) => t,
        Err(_) => return false,
    };
    text.push('\n');
    std::fs::write(config_path, text).is_ok()
}

pub fn get_setting(config_path: &str, key: &str) -> Option<Value> {
    load_config(config_path).get(key).cloned()
}

pub fn set_setting(config_path: &str, key: &str, value: Value) -> bool {
    let mut data = load_config(config_path);
    data.insert(key.to_string(), value);
    save_config(config_path, &data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Mirrors `tests/test_config_repository.py`.
    #[test]
    fn get_and_set_setting() {
        // Mirrors test_get_and_set_setting.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let p = path.to_str().unwrap();
        assert_eq!(get_setting(p, "non_existent_key"), None);
        assert_eq!(
            load_config(p).get("non_existent_key").cloned(),
            None
        );
        assert!(set_setting(p, "wallpaper_folder", json!("/tmp/wallpapers")));
        assert!(std::path::Path::new(p).is_file());
        assert_eq!(get_setting(p, "wallpaper_folder"), Some(json!("/tmp/wallpapers")));
        // File ends with newline (Python `f.write("\n")` parity).
        assert!(std::fs::read_to_string(p).unwrap().ends_with('\n'));
    }

    #[test]
    fn multiple_keys_preservation() {
        // Mirrors test_multiple_keys_preservation.
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        let p = p.to_str().unwrap();
        assert!(set_setting(p, "key1", json!("val1")));
        assert!(set_setting(p, "key2", json!(123)));
        assert_eq!(get_setting(p, "key1"), Some(json!("val1")));
        assert_eq!(get_setting(p, "key2"), Some(json!(123)));
    }

    #[test]
    fn corrupt_or_missing_config_loads_empty() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        let p = p.to_str().unwrap();
        assert!(load_config(p).is_empty());
        std::fs::write(p, "[1,2,3]").unwrap();
        assert!(load_config(p).is_empty());
        std::fs::write(p, "{bad json").unwrap();
        assert!(load_config(p).is_empty());
    }

    #[test]
    fn default_path_parity() {
        assert_eq!(
            default_config_path("/home/u"),
            "/home/u/.config/sway-manager/config.json"
        );
    }
}
