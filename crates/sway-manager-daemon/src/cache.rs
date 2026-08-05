use freedesktop_desktop_entry::{default_paths, DesktopEntry};
use sway_manager_core::MenuItem;
use std::sync::{Arc, RwLock};
use tracing::info;

#[derive(Clone, Default)]
pub struct ApplicationCache {
    items: Arc<RwLock<Vec<MenuItem>>>,
}

impl ApplicationCache {
    pub fn new() -> Self {
        let cache = Self {
            items: Arc::new(RwLock::new(Vec::new())),
        };
        cache.reload();
        cache
    }

    pub fn reload(&self) {
        let mut loaded_items = Vec::new();

        for path in default_paths() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        if let Ok(bytes) = std::fs::read_to_string(&entry_path) {
                            if let Ok(desktop_entry) = DesktopEntry::decode(&entry_path, &bytes) {
                                if desktop_entry.no_display() {
                                    continue;
                                }
                                let name = desktop_entry.name(None).unwrap_or_default().to_string();
                                let exec = desktop_entry.exec().unwrap_or_default().to_string();
                                let icon = desktop_entry.icon().map(|s| s.to_string());
                                let comment = desktop_entry.comment(None).map(|s| s.to_string());

                                if !name.is_empty() && !exec.is_empty() {
                                    loaded_items.push(MenuItem {
                                        name,
                                        exec,
                                        icon,
                                        comment,
                                        desktop_file_path: Some(entry_path),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Carregados {} aplicativos no cache do Wofi", loaded_items.len());
        if let Ok(mut items) = self.items.write() {
            *items = loaded_items;
        }
    }

    pub fn get_items(&self) -> Vec<MenuItem> {
        self.items.read().map(|guard| guard.clone()).unwrap_or_default()
    }
}
