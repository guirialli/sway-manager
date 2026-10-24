//! Binary resolution for `SwayManager` / `SwayManagerGUI`.
//!
//! Mirrors `get_binary_path()` in
//! `src/infrastructure/daemon/daemon_utils.py`: prefer a sibling of the
//! current executable (compiled/Nuitka layout, trying the name as-is,
//! lower- and upper-case), otherwise fall back to the bare binary name
//! (resolved via `PATH` by the spawner).

use std::path::PathBuf;

/// Resolve the executable command for `binary_name`.
///
/// Always returns a non-empty command: `[path]` when a sibling binary
/// exists next to the current executable, else `[binary_name]`.
pub fn binary_command(binary_name: &str) -> Vec<String> {
    if let Ok(current) = std::env::current_exe()
        && let Some(dir) = current.parent()
    {
        for candidate in [
            binary_name.to_string(),
            binary_name.to_lowercase(),
            binary_name.to_uppercase(),
        ] {
            let path: PathBuf = dir.join(&candidate);
            if path.is_file() {
                return vec![path.to_string_lossy().into_owned()];
            }
        }
    }
    vec![binary_name.to_string()]
}
