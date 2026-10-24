//! Daemon runtime: environment, startup sequence, memory upkeep.
//!
//! Mirrors `server/config/gui_config.py` (`setup_qt_environment`,
//! `setup_qt_cache`), `daemon_server.start` steps and `perform_gc`:
//! Qt/Wayland reconnect env (kept for the GUI children the daemon
//! spawns), pixmap-cache budget constant, stale-socket cleanup, and the
//! periodic + post-command upkeep hook.

use std::path::Path;

pub const PIXMAP_CACHE_LIMIT_KB: u32 = 2048;
/// GC interval mirroring the `QTimer` at `daemon_server.py:90-93`.
pub const GC_INTERVAL_SECS: u64 = 30;

/// `os.environ.setdefault` equivalents for the GUI children.
pub fn setup_environment() {
    // SAFETY: single-threaded startup before the accept loop spawns tasks.
    unsafe {
        if std::env::var_os("QT_WAYLAND_RECONNECT").is_none() {
            std::env::set_var("QT_WAYLAND_RECONNECT", "1");
        }
        if std::env::var_os("QT_QPA_PLATFORM").is_none() {
            std::env::set_var("QT_QPA_PLATFORM", "wayland;xcb");
        }
    }
}

/// Remove a stale socket file left by an unclean shutdown.
///
/// Mirrors `daemon_server.start` (`os.remove` + `QLocalServer.removeServer`
/// before `listen`); the Tokio listener binds the path afterwards.
pub fn cleanup_stale_socket(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

/// Memory upkeep after each command and every [`GC_INTERVAL_SECS`].
///
/// Python calls `QPixmapCache.clear()` + `gc.collect()` +
/// `libc malloc_trim(0)`; the Rust port has no Qt heap in-process (GUI is
/// spawned out), so this compacts the glibc arena the same way and is a
/// no-op on failure — it must never disturb the server loop.
pub fn perform_gc() {
    // SAFETY: `malloc_trim` with pad 0 is always safe to call; the return
    // value only reports whether memory was released.
    unsafe {
        libc::malloc_trim(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_defaults_are_set() {
        setup_environment();
        assert_eq!(std::env::var("QT_WAYLAND_RECONNECT").as_deref(), Ok("1"));
        assert_eq!(std::env::var("QT_QPA_PLATFORM").as_deref(), Ok("wayland;xcb"));
    }

    #[test]
    fn stale_socket_cleanup_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.sock");
        std::fs::write(&sock, "stale").unwrap();
        cleanup_stale_socket(&sock);
        assert!(!sock.exists());
        // Missing file is a no-op, never panics.
        cleanup_stale_socket(&sock);
    }

    #[test]
    fn gc_never_panics() {
        perform_gc();
        assert_eq!(GC_INTERVAL_SECS, 30);
        assert_eq!(PIXMAP_CACHE_LIMIT_KB, 2048);
    }
}
