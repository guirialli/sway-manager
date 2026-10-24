//! `lock` — swaylock launcher with theme-matched colors.
//!
//! Mirrors `SwayLockRepository.lock_screen`
//! (`src/infrastructure/power/swaylock_repository.py:26-111`).
//! Wallpaper/theme providers are injected seams; process spawn matches
//! Python (`Popen` for the 60s swayidle helper, blocking `run` for
//! swaylock, `swaymsg output * dpms on` re-arm in `finally`).

use crate::process::{CommandRunner, shell};

pub const IDLE_DPMS_ON: &str = "swaymsg output * dpms on";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

pub trait LockContext: Send + Sync {
    fn swaylock_running(&self, runner: &dyn CommandRunner) -> bool;
    fn swaylock_bin(&self, runner: &dyn CommandRunner) -> String;
    fn swayidle_available(&self, runner: &dyn CommandRunner) -> bool;
    fn wallpaper(&self) -> Option<String>;
    fn theme(&self) -> Theme;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockOutcome {
    AlreadyRunning,
    Locked,
}

fn dark_colors() -> Vec<String> {
    [
        "-c", "1a0a28", "--ring-color", "6c238eff", "--inside-color", "1a0a28aa",
        "--line-color", "00000000", "--text-color", "c3e88dff", "--key-hl-color",
        "00bcd4ff", "--bs-hl-color", "ff3333ff", "--ring-ver-color", "6c238eff",
        "--inside-ver-color", "1a0a28aa", "--ring-wrong-color", "ff3333ff",
        "--inside-wrong-color", "1a0a28aa", "--text-wrong-color", "ff3333ff",
        "--ring-clear-color", "00bcd4ff", "--inside-clear-color", "1a0a28aa",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect()
}

fn light_colors() -> Vec<String> {
    [
        "-c", "f5f5f7", "--ring-color", "1e66f5ff", "--inside-color", "ffffffaa",
        "--line-color", "00000000", "--text-color", "4c4f69ff", "--key-hl-color",
        "40a02bff", "--bs-hl-color", "d20f39ff", "--ring-ver-color", "1e66f5ff",
        "--inside-ver-color", "ffffffaa", "--ring-wrong-color", "d20f39ff",
        "--inside-wrong-color", "ffffffaa", "--text-wrong-color", "d20f39ff",
        "--ring-clear-color", "40a02bff", "--inside-clear-color", "ffffffaa",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect()
}

/// Build the exact swaylock argv Python assembles at lines 32-86.
pub fn swaylock_command(
    ctx: &dyn LockContext,
    runner: &dyn CommandRunner,
) -> Vec<String> {
    let mut cmd = vec![ctx.swaylock_bin(runner)];
    if let Some(wallpaper) = ctx.wallpaper() {
        cmd.extend(["-i".to_string(), wallpaper, "-s".to_string(), "fill".to_string()]);
    }
    match ctx.theme() {
        Theme::Dark => cmd.extend(dark_colors()),
        Theme::Light => cmd.extend(light_colors()),
    }
    cmd.extend(
        [
            "--indicator-idle-visible",
            "--indicator-caps-lock",
            "-F",
            "--font",
            "MesloLGS NF",
        ]
        .iter()
        .map(|arg| arg.to_string()),
    );
    cmd
}

/// Mirrors `lock_screen`, returning what happened (prints stay inside).
pub fn lock_screen(ctx: &dyn LockContext, runner: &dyn CommandRunner) -> LockOutcome {
    if ctx.swaylock_running(runner) {
        return LockOutcome::AlreadyRunning;
    }
    let cmd = swaylock_command(ctx, runner);
    if ctx.swayidle_available(runner) {
        runner.spawn_detached(
            "swayidle",
            &[
                "-w",
                "timeout",
                "60",
                "swaymsg \"output * dpms off\"",
                "resume",
                "swaymsg \"output * dpms on\"",
            ],
        );
    }
    let (program, rest) = cmd.split_first().expect("swaylock command non-empty");
    let args: Vec<&str> = rest.iter().map(String::as_str).collect();
    runner.run(program, &args);
    shell(runner, IDLE_DPMS_ON);
    LockOutcome::Locked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::FakeRunner;

    struct Ctx {
        theme: Theme,
        wallpaper: Option<String>,
    }

    impl LockContext for Ctx {
        fn swaylock_running(&self, _runner: &dyn CommandRunner) -> bool {
            false
        }
        fn swaylock_bin(&self, runner: &dyn CommandRunner) -> String {
            runner.which("swaylock").unwrap_or_else(|| "swaylock".to_string())
        }
        fn swayidle_available(&self, runner: &dyn CommandRunner) -> bool {
            runner.which("swayidle").is_some()
        }
        fn wallpaper(&self) -> Option<String> {
            self.wallpaper.clone()
        }
        fn theme(&self) -> Theme {
            self.theme
        }
    }

    #[test]
    fn dark_lock_command_literals() {
        let runner = FakeRunner::new()
            .with_which("swaylock", Some("/usr/bin/swaylock"))
            .with_which("swayidle", None);
        let ctx = Ctx {
            theme: Theme::Dark,
            wallpaper: Some("/w/wall.jpg".to_string()),
        };
        let cmd = swaylock_command(&ctx, &runner);
        assert_eq!(cmd[0], "/usr/bin/swaylock");
        assert!(cmd.windows(2).any(|pair| pair == ["-i", "/w/wall.jpg"]));
        assert!(cmd.contains(&"1a0a28".to_string()));
        assert!(cmd.contains(&"MesloLGS NF".to_string()));
    }

    #[test]
    fn lock_rearms_dpms_and_spawns_idle_helper() {
        let runner = FakeRunner::new()
            .with_which("swaylock", Some("/usr/bin/swaylock"))
            .with_which("swayidle", Some("/usr/bin/swayidle"))
            .with_success("/usr/bin/swaylock", &[], "")
            .with_success("swaymsg", &["output", "*", "dpms", "on"], "");
        let ctx = Ctx { theme: Theme::Light, wallpaper: None };
        assert_eq!(lock_screen(&ctx, &runner), LockOutcome::Locked);
        let spawned = runner.spawned();
        assert_eq!(spawned[0][0], "swayidle");
        assert!(spawned[0].contains(&"60".to_string()));
    }
}
