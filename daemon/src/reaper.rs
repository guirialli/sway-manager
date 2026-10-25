//! Reaper for detached child processes.
//!
//! Every `spawn()` without `wait()` leaves a zombie (`Z`) until the parent
//! reaps it. Detached GUI processes outlive the request that spawned them,
//! so the daemon parks them here and a single background thread reaps each
//! one as soon as it exits.

use std::process::Child;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static PARKED: LazyLock<Mutex<Vec<Child>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static REAPER: LazyLock<()> = LazyLock::new(|| {
    std::thread::Builder::new()
        .name("sway-manager-reaper".to_string())
        .spawn(|| loop {
            std::thread::sleep(Duration::from_secs(2));
            let Ok(mut guard) = PARKED.lock() else {
                continue;
            };
            guard.retain_mut(|child| {
                // `try_wait` reaps on `Ok(Some(_))`; keep the rest.
                !matches!(child.try_wait(), Ok(Some(_)))
            });
        })
        .ok();
});

/// Park a detached child for background reaping.
///
/// Starts the reaper thread on first call; each parked child is
/// `try_wait()`-polled until it exits, then dropped (reaped).
/// Never blocks the caller.
pub fn park(child: Child) {
    if let Ok(mut guard) = PARKED.lock() {
        guard.push(child);
    }
    LazyLock::force(&REAPER);
}
