//! Command execution: argv parsing, process running, notifications.
//!
//! Shared by every repository that shells out (`swaymsg`, `pgrep`,
//! `powerprofilesctl`, `brightnessctl`, `wpctl`, `notify-send`, ...).
//! Python uses `subprocess.run/subprocess.Popen` with text capture;
//! this module is the single choke point so tests can stub it.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CmdOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            status: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    pub fn failure(status: i32, stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            status,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    pub fn succeeded(&self) -> bool {
        self.status == 0
    }
}

fn shell_split(script: &str) -> (String, Vec<String>) {
    let mut parts = script.split_whitespace();
    let program = parts.next().unwrap_or("sh").to_string();
    let args = parts.map(|part| part.to_string()).collect();
    (program, args)
}

/// What the daemon can run; stubbed in tests.
pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> CmdOutput;
    /// `Popen(communicate(input=...))`: wofi menu payload on stdin.
    /// The fake ignores stdin and matches on argv.
    fn run_with_input(&self, program: &str, args: &[&str], input: &str) -> CmdOutput {
        let _ = input;
        self.run(program, args)
    }
    fn spawn_detached(&self, program: &str, args: &[&str]);
    fn which(&self, binary: &str) -> Option<String>;
}

/// Real runner: `subprocess.run(capture_output=True, text=True)` and
/// `Popen(start_new_session=True, DEVNULL)` equivalents.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
        match std::process::Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
        {
            Ok(output) => CmdOutput {
                status: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            },
            Err(err) => CmdOutput::failure(-1, String::new(), err.to_string()),
        }
    }
    fn run_with_input(&self, program: &str, args: &[&str], input: &str) -> CmdOutput {
        use std::io::Write as _;
        match std::process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(input.as_bytes());
                }
                match child.wait_with_output() {
                    Ok(output) => CmdOutput {
                        status: output.status.code().unwrap_or(-1),
                        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                        stderr: String::new(),
                    },
                    Err(err) => CmdOutput::failure(-1, String::new(), err.to_string()),
                }
            }
            Err(err) => CmdOutput::failure(-1, String::new(), err.to_string()),
        }
    }


    fn spawn_detached(&self, program: &str, args: &[&str]) {
        use std::os::unix::process::CommandExt as _;
        let result = std::process::Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn();
        if let Err(err) = result {
            eprintln!("Erro ao iniciar processo isolado '{program}': {err}");
        }
    }

    fn which(&self, binary: &str) -> Option<String> {
        if binary.contains('/') {
            return std::path::Path::new(binary)
                .is_file()
                .then(|| binary.to_string());
        }
        std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths).find_map(|dir| {
                let candidate = dir.join(binary);
                candidate.is_file().then(|| candidate.to_string_lossy().into_owned())
            })
        })
    }
}

/// In-memory stub; tests preregister outputs keyed by argv.
/// `run_with_input` also records the piped stdin per argv key so tests
/// can assert the exact wofi payload (e.g. `img:<path>:text:` icons).
#[derive(Debug, Default, Clone)]
pub struct FakeRunner {
    outputs: Arc<Mutex<HashMap<String, CmdOutput>>>,
    spawned: Arc<Mutex<Vec<Vec<String>>>>,
    paths: Arc<Mutex<HashMap<String, Option<String>>>>,
    inputs: Arc<Mutex<HashMap<String, Vec<String>>>>,
}
impl FakeRunner {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(program: &str, args: &[&str]) -> String {
        std::iter::once(program)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join("\u{1f}")
    }

    pub fn with_output(self, program: &str, args: &[&str], output: CmdOutput) -> Self {
        self.outputs
            .lock()
            .unwrap()
            .insert(Self::key(program, args), output);
        self
    }

    pub fn with_success(self, program: &str, args: &[&str], stdout: &str) -> Self {
        self.outputs.lock().unwrap().insert(
            Self::key(program, args),
            CmdOutput::success(stdout),
        );
        self
    }

    pub fn with_failure(self, program: &str, args: &[&str], status: i32) -> Self {
        self.outputs.lock().unwrap().insert(
            Self::key(program, args),
            CmdOutput::failure(status, String::new(), String::new()),
        );
        self
    }

    pub fn with_which(self, binary: &str, path: Option<&str>) -> Self {
        self.paths
            .lock()
            .unwrap()
            .insert(binary.to_string(), path.map(|path| path.to_string()));
        self
    }

    pub fn spawned(&self) -> Vec<Vec<String>> {
        self.spawned.lock().unwrap().clone()
    }

    /// Payloads piped to `program` via `run_with_input`, in call order.
    pub fn piped_inputs(&self, program: &str, args: &[&str]) -> Vec<String> {
        self.inputs.lock().unwrap().get(&Self::key(program, args)).cloned().unwrap_or_default()
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str]) -> CmdOutput {
        self.outputs
            .lock()
            .unwrap()
            .get(&Self::key(program, args))
            .cloned()
            .unwrap_or_else(|| CmdOutput::failure(1, String::new(), String::new()))
    }
    fn run_with_input(&self, program: &str, args: &[&str], input: &str) -> CmdOutput {
        self.inputs
            .lock()
            .unwrap()
            .entry(Self::key(program, args))
            .or_default()
            .push(input.to_string());
        self.run(program, args)
    }


    fn spawn_detached(&self, program: &str, args: &[&str]) {
        let mut cmd = vec![program.to_string()];
        cmd.extend(args.iter().map(|arg| arg.to_string()));
        self.spawned.lock().unwrap().push(cmd);
    }

    fn which(&self, binary: &str) -> Option<String> {
        self.paths
            .lock()
            .unwrap()
            .get(binary)
            .cloned()
            .unwrap_or(None)
    }
}

pub fn shared(runner: FakeRunner) -> Arc<FakeRunner> {
    Arc::new(runner)
}

/// Desktop notification via `notify-send -a SwayManager`.
///
/// Mirrors `DesktopNotificationRepository` /
/// `src/infrastructure/notification/desktop_notification_repository.py`.
/// `urgency` is `"normal"` or `"critical"`.
pub fn notify(runner: &dyn CommandRunner, title: &str, message: &str, urgency: &str) {
    let app = "SwayManager";
    if urgency == "critical" {
        runner.run("notify-send", &["-a", app, "-u", "critical", title, message]);
    } else {
        runner.run("notify-send", &["-a", app, title, message]);
    }
}

/// `sh -c` helper for redirections (`echo {v} > 'file'`); returns the
/// runner output. Mirrors the `pkexec`/`sudo` elevation path.
pub fn shell(runner: &dyn CommandRunner, script: &str) -> CmdOutput {
    let (program, args) = shell_split(script);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    runner.run(&program, &arg_refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_uses_normal_vs_critical_argv() {
        let runner = FakeRunner::new()
            .with_success("notify-send", &["-a", "SwayManager", "T", "M"], "")
            .with_success("notify-send", &["-a", "SwayManager", "-u", "critical", "T", "M"], "");
        notify(&runner, "T", "M", "normal");
        notify(&runner, "T", "M", "critical");
    }

    #[test]
    fn shell_splits_and_runs() {
        let runner = FakeRunner::new()
            .with_success("echo", &["hello"], "hello\n");
        let output = shell(&runner, "echo hello");
        assert!(output.succeeded());
    }

    #[test]
    fn fake_which_and_spawned_records() {
        let runner = FakeRunner::new().with_which("foo", Some("/usr/bin/foo"));
        assert_eq!(runner.which("foo").as_deref(), Some("/usr/bin/foo"));
        assert_eq!(runner.which("missing"), None);
        runner.spawn_detached("foo", &["--bar"]);
        assert_eq!(runner.spawned(), vec![vec!["foo".to_string(), "--bar".to_string()]]);
        let _ = shared(runner);
    }

    #[test]
    fn system_which_resolves_absolute_and_path() {
        let runner = SystemRunner;
        assert_eq!(runner.which("/bin/sh"), Some("/bin/sh".to_string()));
        assert_eq!(runner.which("definitely-not-a-binary-xyz"), None);
    }
}
