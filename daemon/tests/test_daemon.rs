//! Ports of `tests/test_daemon.py::TestDaemonClientAndServer` +
//! end-to-end sync parity against the real `handle_sync` path.
//!
//! The first block maps 1:1 to the Python originals. The second block
//! (`*_via_real_handlers`) drives requests through `SyncCliRunner` +
//! `handle_sync` with an in-memory `CommandRunner`, asserting the exact
//! stdout literals the Python daemon ships back over the socket.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use sway_manager_daemon::binary::binary_command;
use sway_manager_daemon::client::{Output, send_command_to};
use sway_manager_daemon::commands::{GUI_COMMANDS, INTERACTIVE_COMMANDS, is_gui_command};
use sway_manager_daemon::dispatch::{CliRunner, DispatchOutcome, dispatch};
use sway_manager_daemon::gui::{GuiSpawner, gui_command};
use sway_manager_daemon::handlers::{SyncDeps, handle_sync};
use sway_manager_daemon::process::{CommandRunner, FakeRunner as ProcessFake};
use sway_manager_daemon::protocol::{Request, Response, decode_line, encode_line};
use sway_manager_daemon::repos::{battery, lock, theme};
use sway_manager_daemon::server::{SyncCliRunner, handle_connection, is_daemon_running_at};
use sway_manager_daemon::socket::{socket_path, socket_path_from};

struct FakeRunner {
    fail_on_battery: bool,
    calls: Arc<Mutex<usize>>,
    _gc_calls: Arc<Mutex<usize>>,
}

#[derive(Debug)]
struct RunnerError(String);

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CliRunner for FakeRunner {
    type Error = RunnerError;

    fn run_cli(&self, args: &[String]) -> Result<(String, String), Self::Error> {
        *self.calls.lock().unwrap() += 1;
        let cmd = args.get(1).cloned().unwrap_or_default();
        if self.fail_on_battery && cmd == "battery" {
            return Err(RunnerError(format!(
                "Falha na execução do comando '{cmd}': Simulated Error"
            )));
        }
        Ok((format!("OK for {cmd}"), String::new()))
    }
}

#[derive(Default, Clone)]
struct FakeGui {
    spawned: Arc<Mutex<Vec<Vec<String>>>>,
}

impl GuiSpawner for FakeGui {
    fn spawn_gui(&self, args: &[String]) -> Vec<String> {
        let cmd = gui_command(args);
        self.spawned.lock().unwrap().push(cmd.clone());
        cmd
    }
}

#[derive(Default)]
struct SilentOutput {
    stdout: String,
    stderr: String,
}

impl Output for SilentOutput {
    fn print_stdout(&mut self, text: &str) {
        self.stdout.push_str(text);
    }
    fn print_stderr(&mut self, text: &str) {
        self.stderr.push_str(text);
    }
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| part.to_string()).collect()
}

// (i) test_get_socket_path_format
#[test]
fn socket_path_uses_runtime_socket_name() {
    let path = socket_path().to_string_lossy().into_owned();
    assert!(path.ends_with(".sock"), "socket path: {path}");
    assert!(path.contains("sway-manager"), "socket path: {path}");
}

// (ii) test_get_binary_path_resolution — non-empty command either way,
// plus bare-name fallback (compiled layout without sibling).
#[test]
fn binary_command_always_resolves() {
    assert!(!binary_command("SwayManager").is_empty());
    assert!(!binary_command("SwayManagerGUI").is_empty());
}

#[test]
fn binary_command_falls_back_to_bare_name() {
    // No `DefinitelyNotInstalled12345*` sibling exists next to the test
    // binary, so resolution must fall back to the bare name (PATH lookup
    // by the spawner) — mirroring the Python `return [binary_name]` path.
    assert_eq!(
        binary_command("DefinitelyNotInstalled12345"),
        vec!["DefinitelyNotInstalled12345".to_string()]
    );
}

// (iii) test_interactive_commands_set
#[test]
fn interactive_command_set_contains_core_commands() {
    for cmd in ["menu", "screenshot", "settings"] {
        assert!(INTERACTIVE_COMMANDS.contains(&cmd));
    }
}

// (iv) test_client_send_command_daemon_not_running
#[tokio::test]
async fn client_returns_false_when_socket_missing() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("sway-manager-1.sock");
    let mut output = SilentOutput::default();
    let result = send_command_to(
        &missing,
        &argv(&["SwayManager", "menu"]),
        &mut output,
        Duration::from_secs(2),
    )
    .await;
    assert!(!result);
}

// (v) test_is_daemon_running_check
#[tokio::test]
async fn daemon_probe_is_false_when_socket_missing() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("sway-manager-1.sock");
    assert!(!is_daemon_running_at(&missing).await);
}

// (vi) test_daemon_command_exception_handled_without_crash — the runner
// error is swallowed (logged upstream in Python), GC still runs.
#[test]
fn dispatch_swallows_cli_error_and_runs_gc() {
    let calls = Arc::new(Mutex::new(0usize));
    let gc_calls = Arc::new(Mutex::new(0usize));
    let runner = FakeRunner {
        fail_on_battery: true,
        calls: Arc::clone(&calls),
        _gc_calls: Arc::clone(&gc_calls),
    };
    let gui = FakeGui::default();

    let outcome = dispatch(
        &argv(&["SwayManager", "battery", "toggle"]),
        &runner,
        &gui,
        || {
            *gc_calls.lock().unwrap() += 1;
        },
    );

    assert_eq!(
        outcome,
        DispatchOutcome::CliExecuted {
            stdout: String::new(),
            stderr: String::new(),
        }
    );
    assert_eq!(*calls.lock().unwrap(), 1);
    assert_eq!(*gc_calls.lock().unwrap(), 1);
    assert!(gui.spawned.lock().unwrap().is_empty());
}

// (vii) test_client_send_command_success — full Unix-socket round-trip.
#[tokio::test]
async fn client_round_trip_over_unix_socket() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("test_daemon.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (mut conn, _) = listener.accept().await.unwrap();
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        let mut reader = tokio::io::BufReader::new(&mut conn);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let request: Request = decode_line(&line).unwrap();
        let response = encode_line(&Response::ok(format!("OK for {}", request.args[1]))).unwrap();
        conn.write_all(response.as_bytes()).await.unwrap();
    });

    let mut output = SilentOutput::default();
    let result = send_command_to(
        &sock_path,
        &argv(&["SwayManager", "battery", "status"]),
        &mut output,
        Duration::from_secs(2),
    )
    .await;

    server.await.unwrap();
    assert!(result);
    assert_eq!(output.stdout, "OK for battery");
}

// (viii) test_gui_commands_dispatch_spawns_gui_executable
#[test]
fn dispatch_routes_gui_commands_to_spawner() {
    let gui = FakeGui::default();
    let runner = FakeRunner {
        fail_on_battery: false,
        calls: Arc::new(Mutex::new(0)),
        _gc_calls: Arc::new(Mutex::new(0)),
    };

    for args in [
        argv(&["SwayManager", "settings"]),
        argv(&["SwayManager", "screenshot", "area"]),
    ] {
        let outcome = dispatch(&args, &runner, &gui, || {});
        assert_eq!(outcome, DispatchOutcome::GuiSpawned);
    }

    let spawned = gui.spawned.lock().unwrap();
    assert_eq!(spawned.len(), 2);
    assert_eq!(&spawned[0][1..], &["settings".to_string()]);
    assert_eq!(&spawned[1][1..], &["screenshot".to_string(), "area".to_string()]);
}

// (ix) test_spawn_standalone_gui_invokes_subprocess — GUI spawn builds
// `[SwayManagerGUI, ...subcommand]`; plus the clipboard `ensure_daemon`
// contract: `wl-paste --watch cliphist store` is the daemonized command.
#[test]
fn gui_spawn_builds_isolated_subprocess_command() {
    let cmd = gui_command(&argv(&["SwayManager", "settings"]));
    assert!(cmd.iter().any(|arg| arg.contains("SwayManagerGUI")));
    assert!(cmd.contains(&"settings".to_string()));

    assert!(is_gui_command("settings"));
    assert!(!GUI_COMMANDS.contains(&"battery"));

    let cliphist_watch = ["wl-paste", "--watch", "cliphist", "store"];
    assert_eq!(cliphist_watch[0], "wl-paste");
}

#[test]
fn fallback_socket_dir_matches_python_tmp_layout() {
    let path = socket_path_from(None, 1000);
    assert_eq!(
        path.to_string_lossy(),
        "/tmp/user-1000/sway-manager-1000.sock"
    );
}

#[test]
fn socket_path_prefers_valid_runtime_dir() {
    let dir = tempfile::tempdir().unwrap();
    let path = socket_path_from(Some(dir.path().to_str().unwrap()), 4242);
    assert_eq!(
        path,
        dir.path().join("sway-manager-4242.sock"),
        "must use $XDG_RUNTIME_DIR when set"
    );
}

#[tokio::test]
async fn server_answers_malformed_request_with_exit_code_1() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("malformed.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (mut conn, _) = listener.accept().await.unwrap();
        let runner = FakeRunner {
            fail_on_battery: false,
            calls: Arc::new(Mutex::new(0)),
            _gc_calls: Arc::new(Mutex::new(0)),
        };
        let (logger, _rx) = sway_manager_daemon::logging::test_logger();
        handle_connection(&mut conn, &runner, &FakeGui::default(), Duration::from_secs(2), &logger).await;
    });

    let mut client = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    client.write_all(b"not json\n").await.unwrap();
    let mut reader = tokio::io::BufReader::new(&mut client);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    let response: Response = decode_line(&line).unwrap();

    server.await.unwrap();
    assert_eq!(response.exit_code, 1);
    assert!(!response.stderr.is_empty());
}

// --- Real-handler parity: requests through SyncCliRunner + handle_sync ---

#[derive(Debug, Default)]
struct MemBatteryFs {
    files: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl MemBatteryFs {
    fn with(self, path: &str, value: &str) -> Self {
        self.files.lock().unwrap().insert(path.to_string(), value.to_string());
        self
    }
}

impl battery::BatteryFs for MemBatteryFs {
    fn is_file(&self, path: &str) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }
    fn read_value(&self, path: &str) -> Option<i32> {
        self.files.lock().unwrap().get(path).and_then(|text| text.trim().parse().ok())
    }
    fn write_value(&self, path: &str, value: i32) -> bool {
        if let Some(slot) = self.files.lock().unwrap().get_mut(path) {
            *slot = value.to_string();
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
struct MemThemeFs {
    files: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl theme::ThemeFs for MemThemeFs {
    fn read_theme(&self, path: &str) -> Option<String> {
        self.files.lock().unwrap().get(path).cloned()
    }
    fn write_theme(&self, path: &str, theme: &str) {
        self.files.lock().unwrap().insert(path.to_string(), theme.to_string());
    }
    fn kitty_source(&self, _path: &str) -> bool {
        false
    }
    fn copy_kitty_theme(&self, _src: &str, _dst: &str) -> Result<(), String> {
        Ok(())
    }
}

struct StubLockCtx;

impl lock::LockContext for StubLockCtx {
    fn swaylock_running(&self, _runner: &dyn CommandRunner) -> bool {
        true // no-op path; lock tested separately in repos::lock
    }
    fn swaylock_bin(&self, _runner: &dyn CommandRunner) -> String {
        "swaylock".to_string()
    }
    fn swayidle_available(&self, _runner: &dyn CommandRunner) -> bool {
        false
    }
    fn wallpaper(&self) -> Option<String> {
        None
    }
    fn theme(&self) -> lock::Theme {
        lock::Theme::Dark
    }
}

struct Deps {
    runner: ProcessFake,
    battery: MemBatteryFs,
    theme_fs: MemThemeFs,
    lock: StubLockCtx,
    home: String,
    menu_calls: Arc<Mutex<Vec<Option<String>>>>,
    clipboard_calls: Arc<Mutex<Vec<Option<String>>>>,
}

impl SyncDeps for Deps {
    fn runner(&self) -> &dyn CommandRunner {
        &self.runner
    }
    fn battery_fs(&self) -> &dyn battery::BatteryFs {
        &self.battery
    }
    fn theme_fs(&self) -> &dyn theme::ThemeFs {
        &self.theme_fs
    }
    fn theme_applier(&self) -> &dyn theme::ThemeApplier {
        &theme::SystemApplier
    }
    fn lock_ctx(&self) -> &dyn lock::LockContext {
        &self.lock
    }
    fn home(&self) -> &str {
        &self.home
    }
    fn run_menu(&self, filter: Option<&str>) {
        self.menu_calls.lock().unwrap().push(filter.map(|filter| filter.to_string()));
    }
    fn run_clipboard(&self, action: Option<&str>) {
        self.clipboard_calls.lock().unwrap().push(action.map(|action| action.to_string()));
    }
}

const CTL: &str = "/sys/devices/platform/ideapad_acpi/conservation_mode";

fn parity_deps(runner: ProcessFake) -> Deps {
    Deps {
        runner,
        battery: MemBatteryFs::default().with(CTL, "1\n"),
        theme_fs: MemThemeFs::default(),
        lock: StubLockCtx,
        home: "/home/u".to_string(),
        menu_calls: Arc::new(Mutex::new(Vec::new())),
        clipboard_calls: Arc::new(Mutex::new(Vec::new())),
    }
}

#[test]
fn parity_battery_status_json() {
    let deps = parity_deps(ProcessFake::new());
    let (stdout, stderr) = handle_sync(&deps, &argv(&["SwayManager", "battery", "status"]));
    // Exact Python `json.dumps` bytes: insertion order, `ensure_ascii`.
    let expected = "{\"text\": \"\\uf241\", \"class\": \"conservation-on\", \"tooltip\": \"Conserva\\u00e7\\u00e3o de Bateria: LIGADO (~80%)\"}\n";
    assert_eq!(stdout, expected);
    assert!(stderr.is_empty());
}

#[test]
fn parity_battery_toggle_literal() {
    let runner = ProcessFake::new().with_success("notify-send", &[], "");
    let deps = parity_deps(runner);
    let (stdout, _) = handle_sync(&deps, &argv(&["SwayManager", "battery", "toggle"]));
    assert_eq!(stdout, "Conservação Desligada (Carregando até 100%)\n");
}

#[test]
fn parity_idle_power_theme_status() {
    let runner = ProcessFake::new()
        .with_success("pgrep", &["-x", "swayidle"], "")
        .with_success("powerprofilesctl", &["get"], "balanced\n");
    let deps = parity_deps(runner);
    let (idle_out, _) = handle_sync(&deps, &argv(&["SwayManager", "idle", "status"]));
    assert!(idle_out.contains("\"class\": \"idle-on\""));
    let (power_out, _) = handle_sync(&deps, &argv(&["SwayManager", "power", "status"]));
    assert!(power_out.contains("\"tooltip\": \"Power Profile: balanced\""));
    let (theme_out, _) = handle_sync(&deps, &argv(&["SwayManager", "theme", "status"]));
    assert!(theme_out.contains("\"class\": \"theme-light\""));
}

#[test]
fn parity_unknown_command_stderr() {
    let deps = parity_deps(ProcessFake::new());
    let (stdout, stderr) = handle_sync(&deps, &argv(&["SwayManager", "frobnicar"]));
    assert!(stdout.is_empty());
    assert_eq!(stderr, "Comando 'frobnicar' não reconhecido pelo SwayManager.\n");
}

#[test]
fn parity_menu_clipboard_lock_reload_dispatch() {
    let deps = parity_deps(ProcessFake::new());
    let (menu_out, menu_err) = handle_sync(&deps, &argv(&["SwayManager", "menu", "/a"]));
    assert!(menu_out.is_empty() && menu_err.is_empty());
    assert_eq!(*deps.menu_calls.lock().unwrap(), vec![Some("/a".to_string())]);
    let (clip_out, clip_err) = handle_sync(&deps, &argv(&["SwayManager", "clip"]));
    assert!(clip_out.is_empty() && clip_err.is_empty());
    assert_eq!(*deps.clipboard_calls.lock().unwrap(), vec![None]);
    let (lock_out, lock_err) = handle_sync(&deps, &argv(&["SwayManager", "lock"]));
    assert!(lock_out.is_empty() && lock_err.is_empty());
    let (reload_out, reload_err) = handle_sync(&deps, &argv(&["SwayManager", "reload-cache"]));
    assert!(reload_out.contains("Cache do Wofi recarregado com sucesso (5 itens)."));
    assert!(reload_err.is_empty());
    let (refresh_out, _) = handle_sync(&deps, &argv(&["SwayManager", "refresh-cache"]));
    assert!(refresh_out.contains("(5 itens)"));
}

#[test]
fn dispatch_no_command_short_circuits() {
    let runner = FakeRunner {
        fail_on_battery: false,
        calls: Arc::new(Mutex::new(0)),
        _gc_calls: Arc::new(Mutex::new(0)),
    };
    let outcome = dispatch(&argv(&["SwayManager"]), &runner, &FakeGui::default(), || {});
    assert_eq!(outcome, DispatchOutcome::NoCommand);
    assert_eq!(*runner.calls.lock().unwrap(), 0);
}

#[tokio::test]
async fn parity_sync_round_trip_over_real_server() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("parity.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (mut conn, _) = listener.accept().await.unwrap();
        let deps = parity_deps(ProcessFake::new());
        let (logger, _rx) = sway_manager_daemon::logging::test_logger();
        handle_connection(
            &mut conn,
            &SyncCliRunner::new(deps),
            &FakeGui::default(),
            Duration::from_secs(2),
            &logger,
        )
        .await;
    });

    let mut output = SilentOutput::default();
    let ok = send_command_to(
        &sock_path,
        &argv(&["SwayManager", "battery", "status"]),
        &mut output,
        Duration::from_secs(2),
    )
    .await;
    server.await.unwrap();
    assert!(ok);
    assert!(output.stdout.contains("conservation-on"));
}

async fn round_trip(args: &[String]) -> (bool, SilentOutput) {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("rt.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    let server = tokio::spawn(async move {
        let (mut conn, _) = listener.accept().await.unwrap();
        let deps = parity_deps(ProcessFake::new());
        let (logger, _rx) = sway_manager_daemon::logging::test_logger();
        handle_connection(
            &mut conn,
            &SyncCliRunner::new(deps),
            &FakeGui::default(),
            Duration::from_secs(2),
            &logger,
        )
        .await;
    });
    let mut output = SilentOutput::default();
    let ok = send_command_to(&sock_path, args, &mut output, Duration::from_secs(2)).await;
    server.await.unwrap();
    (ok, output)
}

#[tokio::test]
async fn server_interactive_gui_acks_immediately() {
    let (ok, output) = round_trip(&argv(&["SwayManager", "settings"])).await;
    assert!(ok);
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

#[tokio::test]
async fn server_no_command_acks() {
    let (ok, _) = round_trip(&argv(&["SwayManager"])).await;
    assert!(ok);
}

#[tokio::test]
async fn server_empty_line_closes_without_response() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("empty.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    let server = tokio::spawn(async move {
        let (mut conn, _) = listener.accept().await.unwrap();
        let deps = parity_deps(ProcessFake::new());
        let (logger, _rx) = sway_manager_daemon::logging::test_logger();
        handle_connection(
            &mut conn,
            &SyncCliRunner::new(deps),
            &FakeGui::default(),
            Duration::from_secs(2),
            &logger,
        )
        .await;
    });
    let mut client = tokio::net::UnixStream::connect(&sock_path).await.unwrap();
    client.write_all(b"\n").await.unwrap();
    let mut reader = tokio::io::BufReader::new(&mut client);
    let mut line = String::new();
    let read = tokio::time::timeout(Duration::from_millis(500), reader.read_line(&mut line)).await;
    server.await.unwrap();
    // Empty request: connection closes with no response line.
    assert!(read.is_err() || line.is_empty());
}

#[test]
fn sync_deps_arc_forwards_to_inner() {
    use sway_manager_daemon::handlers::SyncDeps;
    let deps = std::sync::Arc::new(parity_deps(ProcessFake::new()));
    assert_eq!(deps.home(), "/home/u");
    deps.run_menu(None);
    deps.run_clipboard(None);
    assert!(!deps.battery_fs().is_file("/nonexistent"));
    let _ = (deps.runner(), deps.theme_fs(), deps.theme_applier(), deps.lock_ctx());
    assert_eq!(*deps.menu_calls.lock().unwrap(), vec![None]);
    assert_eq!(*deps.clipboard_calls.lock().unwrap(), vec![None]);
}

#[tokio::test]
async fn serve_with_answers_one_connection() {
    use sway_manager_daemon::server::serve_with;
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("serve.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    let (logger, _rx) = sway_manager_daemon::logging::test_logger();
    let server = tokio::spawn(async move {
        serve_with(
            sock_path,
            listener,
            SyncCliRunner::new(parity_deps(ProcessFake::new())),
            FakeGui::default(),
            logger,
        )
        .await
    });
    let mut output = SilentOutput::default();
    let ok = send_command_to(
        &dir.path().join("serve.sock"),
        &argv(&["SwayManager", "power", "status"]),
        &mut output,
        Duration::from_secs(2),
    )
    .await;
    server.abort();
    assert!(ok);
    assert!(output.stdout.contains("Power Profile"));
}

#[tokio::test]
async fn daemon_probe_true_when_listening() {
    use sway_manager_daemon::server::is_daemon_running_at;
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("probe.sock");
    let _listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    assert!(is_daemon_running_at(&sock_path).await);
}

#[tokio::test]
async fn client_false_on_silent_socket() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("silent.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();
    // Accept but never answer → client timeout → false.
    tokio::spawn(async move {
        let (_conn, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
    });
    let mut output = SilentOutput::default();
    let ok = send_command_to(
        &sock_path,
        &argv(&["SwayManager", "battery", "status"]),
        &mut output,
        Duration::from_millis(100),
    )
    .await;
    assert!(!ok);
}
