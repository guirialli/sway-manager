//! Unix-socket path resolution for daemon IPC.
//!
//! Mirrors `get_socket_path()` in
//! `src/infrastructure/daemon/daemon_client.py`:
//! `$XDG_RUNTIME_DIR/sway-manager-$UID.sock`, falling back to
//! `/tmp/user-$UID/sway-manager-$UID.sock` when the runtime dir is
//! unset or missing.

use std::path::PathBuf;

/// Pure constructor used by [`socket_path`] and by tests.
pub fn socket_path_from(runtime_dir: Option<&str>, uid: u32) -> PathBuf {
    let dir = runtime_dir
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/tmp/user-{uid}")));
    dir.join(format!("sway-manager-{uid}.sock"))
}

fn runtime_dir_valid(dir: &str) -> bool {
    !dir.is_empty() && std::path::Path::new(dir).exists()
}
/// Runtime socket path for the current user (see module docs).
pub fn socket_path() -> PathBuf {
    let uid = users_uid();
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .filter(|dir| runtime_dir_valid(dir));
    let path = socket_path_from(runtime_dir.as_deref(), uid);
    if runtime_dir.is_none() && let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    path
}

fn users_uid() -> u32 {
    // SAFETY: `getuid` always succeeds and has no side effects.
    unsafe { libc::getuid() }
}
