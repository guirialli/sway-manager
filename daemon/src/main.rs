//! `SwayManager` daemon binary — CLI routing + server startup.
//!
//! Mirrors `src/main.py` → `presentation/cli/router.py::run_cli`:
//! `--standalone` bypass, help, `daemon [log]` startup, local-only
//! `portal`, and socket forwarding with the Python stderr literal +
//! exit 1 when the daemon is down.

use std::io::Write as _;
use std::sync::{Arc, Mutex};

use sway_manager_daemon::binary::binary_command;
use sway_manager_daemon::client::{Output, Stdio, send_command, send_command_to};
use sway_manager_daemon::handlers::SyncDeps;
use sway_manager_daemon::logging;
use sway_manager_daemon::process::{CommandRunner, SystemRunner};
use sway_manager_daemon::repos::{battery, lock, theme};
use sway_manager_daemon::runtime;
use sway_manager_daemon::server::{self, SyncCliRunner};
use sway_manager_daemon::socket::socket_path;

const HELP_TEXT: &str = "\
SwayManager — gerenciador do ambiente Sway
Uso: SwayManager <comando> [args] [--standalone]

Comandos do daemon:
  daemon              Inicia o daemon em foreground
  daemon log [-f]     Mostra o log de hoje (tail -f com -f/--follow)";

const DAEMON_DOWN_MSG: &str = "❌ Erro: O SwayManager Daemon não está em execução.\nInicie o serviço com 'SwayManager daemon' ou verifique a inicialização do Sway.\n";

/// Real dependency bundle: system runner + on-disk state.
struct LiveDeps {
    runner: SystemRunner,
    theme_fs: LiveThemeFs,
}

#[derive(Debug, Default)]
struct LiveBatteryFs;

impl battery::BatteryFs for LiveBatteryFs {
    fn is_file(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }
    fn read_value(&self, path: &str) -> Option<i32> {
        std::fs::read_to_string(path).ok()?.trim().parse().ok()
    }
    fn write_value(&self, path: &str, value: i32) -> bool {
        std::fs::write(path, value.to_string()).is_ok()
    }
}

#[derive(Debug, Default)]
struct LiveThemeFs;

impl theme::ThemeFs for LiveThemeFs {
    fn read_theme(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    fn write_theme(&self, path: &str, theme: &str) {
        let _ = std::fs::write(path, theme);
    }
    fn kitty_source(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }
    fn copy_kitty_theme(&self, src: &str, dst: &str) -> Result<(), String> {
        if let Some(parent) = std::path::Path::new(dst).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::copy(src, dst).map(|_| ()).map_err(|err| err.to_string())
    }
}

struct LiveLockCtx {
    wallpaper: Mutex<Option<String>>,
    dark: Mutex<bool>,
}

impl lock::LockContext for LiveLockCtx {
    fn swaylock_running(&self, runner: &dyn CommandRunner) -> bool {
        runner.run("pgrep", &["-x", "swaylock"]).succeeded()
    }
    fn swaylock_bin(&self, runner: &dyn CommandRunner) -> String {
        runner.which("swaylock").unwrap_or_else(|| "swaylock".to_string())
    }
    fn swayidle_available(&self, runner: &dyn CommandRunner) -> bool {
        runner.which("swayidle").is_some()
    }
    fn wallpaper(&self) -> Option<String> {
        self.wallpaper.lock().unwrap().clone()
    }
    fn theme(&self) -> lock::Theme {
        if *self.dark.lock().unwrap() { lock::Theme::Dark } else { lock::Theme::Light }
    }
}

impl SyncDeps for LiveDeps {
    fn runner(&self) -> &dyn CommandRunner {
        &self.runner
    }
    fn battery_fs(&self) -> &dyn battery::BatteryFs {
        &LiveBatteryFs
    }
    fn theme_fs(&self) -> &dyn theme::ThemeFs {
        &self.theme_fs
    }
    fn theme_applier(&self) -> &dyn theme::ThemeApplier {
        &theme::SystemApplier
    }
    fn lock_ctx(&self) -> &dyn lock::LockContext {
        // Seams read from the wallpaper/theme repos below; the static
        // holder is replaced per-request in `refresh_lock_state`.
        &LIVE_LOCK_CTX
    }
    fn home(&self) -> &str {
        // SAFETY-free: leak once, stable for process lifetime.
        use std::sync::LazyLock;
        static HOME: LazyLock<String> = LazyLock::new(|| {
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        });
        &HOME
    }
    fn run_menu(&self, filter: Option<&str>) {
        use sway_manager_daemon::repos::{desktop_index, icons, menu_exec};
        let home = self.home().to_string();
        let items = desktop_index::load_menu_items(&home);
        let runner = &self.runner;
        let home_clone = home.clone();
        let icon = move |name: &str| icons::resolve_icon(runner, &home_clone, name, 32);
        menu_exec::execute_menu(
            &self.runner,
            &items,
            filter,
            &icon,
            &home,
            std::path::Path::new(&format!("{home}/.config/wofi/config")).exists(),
            std::path::Path::new(&format!("{home}/.config/wofi/style.css")).exists(),
        );
    }
    fn run_clipboard(&self, action: Option<&str>) {
        use sway_manager_daemon::repos::{clipboard_exec, icons};
        let home = self.home().to_string();
        let runner = &self.runner;
        let home_clone = home.clone();
        let icon = move |name: &str| icons::resolve_icon(runner, &home_clone, name, 32);
        let icon_ref = &icon;
        let list = || {
            let favorites = clipboard_exec::load_favorites_pub(&home);
            clipboard_exec::list_items(runner, favorites, &|_| false)
        };
        let decode = |id: &str| {
            let output = runner.run("cliphist", &["decode", id]);
            output.succeeded().then(|| output.stdout.into_bytes())
        };
        let select = |items: &[clipboard_exec::ClipItem], prompt: &str| {
            clipboard_exec::launch_clipboard_menu(runner, &home, icon_ref, items, prompt)
        };
        clipboard_exec::execute(runner, &home, &icon, &list, &decode, &select, action);
    }
    fn spawn_gui(&self, args: &[String]) {
        spawn_gui_detached(args);
    }
    fn wallpaper_dir_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }
}

static LIVE_LOCK_CTX: LiveLockCtx = LiveLockCtx {
    wallpaper: Mutex::new(None),
    dark: Mutex::new(false),
};

fn expand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{home}/{rest}")
    } else {
        path.to_string()
    }
}

/// Refresh the lock seam from the real wallpaper/theme repos (mirrors
/// `SwayLockRepository` constructor defaults).
fn refresh_lock_state() {
    let wallpaper = [expand_home("~/.config/sway/wallpaper")]
        .into_iter()
        .find(|path| std::path::Path::new(path).is_file());
    *LIVE_LOCK_CTX.wallpaper.lock().unwrap() = wallpaper;
    let theme_file = expand_home("~/.config/theme_toggle/theme");
    let dark = std::fs::read_to_string(&theme_file)
        .map(|text| text.trim() == "dark")
        .unwrap_or(false);
    *LIVE_LOCK_CTX.dark.lock().unwrap() = dark;
}

fn daemon_log(args: &[String]) {
    let path = logging::Logger::today_log_path();
    let path_str = path.to_string_lossy().into_owned();
    if !path.exists() {
        println!("Nenhum log registrado para hoje em: {path_str}");
        return;
    }
    let follow = args.iter().any(|arg| arg == "-f" || arg == "--follow");
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let tail = if lines.len() > 50 { &lines[lines.len() - 50..] } else { &lines[..] };
            for line in tail {
                println!("{line}");
            }
        }
        Err(err) => {
            eprintln!("Erro ao ler arquivo de log: {err}");
            return;
        }
    }
    if follow {
        use std::io::{BufRead, Seek as _, SeekFrom};
        let mut file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Erro ao ler arquivo de log: {err}");
                return;
            }
        };
        let _ = file.seek(SeekFrom::End(0));
        let mut reader = std::io::BufReader::new(file);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => std::thread::sleep(std::time::Duration::from_millis(200)),
                Ok(_) => print!("{line}"),
                Err(err) => {
                    eprintln!("Erro ao ler arquivo de log: {err}");
                    return;
                }
            }
        }
    }
}

fn spawn_gui_detached(args: &[String]) {
    let gui_cmd = binary_command("SwayManagerGUI");
    let mut cmd = gui_cmd.clone();
    if args.len() > 1 {
        cmd.extend(args[1..].iter().cloned());
    }
    if let Some((program, rest)) = cmd.split_first() {
        SystemRunner.spawn_detached(
            program,
            &rest.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
}

/// Argv da GUI anexada: `[SwayManagerGUI, ...args[1:]]` (paridade `gui_command`).
fn attached_gui_cmd(args: &[String]) -> Vec<String> {
    let mut cmd = binary_command("SwayManagerGUI");
    if args.len() > 1 {
        cmd.extend(args[1..].iter().cloned());
    }
    cmd
}

/// Spawna a GUI ANEXADA (stdin/stdout/stderr herdados) e propaga o exit.
///
/// Só o chooser `portal` usa: o xdg-desktop-portal-wlr o invoca como
/// subprocesso e lê a seleção no stdout. Detached (`/dev/null`) = o xdpw
/// recebe EOF vazio e o compartilhamento nunca se estabelece.
fn spawn_gui_attached(args: &[String]) -> i32 {
    let cmd = attached_gui_cmd(args);
    let Some((program, rest)) = cmd.split_first() else {
        return 1;
    };
    match std::process::Command::new(program)
        .args(rest)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
    {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            eprintln!("Erro ao disparar processo GUI isolado: {err}");
            1
        }
    }
}

async fn run_daemon() -> i32 {
    runtime::setup_environment();
    let logger = logging::logger();
    if server::is_daemon_running_at(&socket_path()).await {
        println!("🚀 SwayManager Daemon já está em execução.");
        return 0;
    }
    runtime::cleanup_stale_socket(&socket_path());
    let listener = match tokio::net::UnixListener::bind(socket_path()) {
        Ok(listener) => listener,
        Err(err) => {
            let msg = format!(
                "❌ Erro ao iniciar daemon no socket: {} ({err})",
                socket_path().to_string_lossy()
            );
            println!("{msg}");
            logger.error(&msg);
            return 1;
        }
    };
    let msg = format!(
        "🚀 SwayManager Daemon rodando no socket: {}",
        socket_path().to_string_lossy()
    );
    println!("{msg}");
    logger.info(&msg);

    // Preload: count .desktop files (mirrors WofiRepository.preload_cache
    // logging; the full index stays with the Python GUI for now).
    let desktop_count = count_desktop_files();
    let cache_msg = format!(
        "📦 Cache do Wofi e aplicativos pré-carregado em RAM ({desktop_count} itens)!"
    );
    println!("{cache_msg}");
    logger.info(&cache_msg);

    refresh_lock_state();
    let deps = Arc::new(LiveDeps {
        runner: SystemRunner,
        theme_fs: LiveThemeFs,
    });
    let runner = SyncCliRunner::new(deps);
    if let Err(err) = server::serve_with(socket_path(), listener, runner, logger).await {
        eprintln!("❌ Erro no loop do daemon: {err}");
        return 1;
    }
    0
}

fn count_desktop_files() -> usize {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dirs = [
        format!("{home}/.local/share/applications"),
        "/usr/share/applications".to_string(),
    ];
    dirs.iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "desktop"))
        .count()
}

fn local_invoke(args: &[String], stdout: &mut dyn Output) -> i32 {
    // Mirrors `run_cli` local execution: `--standalone` runs the CLI
    // handlers in-process; `portal` always runs locally (never via the
    // socket) since xdg-desktop-portal-wlr invokes it as a subprocess.
    // `portal status` prints the diagnostics report; `portal test` and
    // the bare chooser resolve through the same status/test text path
    // the daemon ships (the Qt chooser UI stays in Python).
    let cmd = args.get(1).map(|cmd| cmd.to_lowercase()).unwrap_or_default();
    if cmd == "portal" {
        let sub = args.get(2).map(|sub| sub.to_lowercase()).unwrap_or_default();
        match sub.as_str() {
            "status" => {
                let runner = SystemRunner;
                let report = sway_manager_daemon::repos::portal::diagnose(&runner);
                stdout.print_stdout(&sway_manager_daemon::repos::portal::status_text(&report));
                return 0;
            }
            "test" => {
                stdout.print_stdout("[TESTE] Cancelado pelo usuário\n");
                return 0;
            }
            "install" => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                let runner = SystemRunner;
                match sway_manager_daemon::repos::portal::install_config(&home, &runner) {
                    Ok(log) => {
                        for line in &log {
                            stdout.print_stdout(&format!("  {line}\n"));
                        }
                        return 0;
                    }
                    Err(err) => {
                        stdout.print_stderr(&format!("Aviso: falha ao configurar portal: {err}\n"));
                        return 1;
                    }
                }
            }
            _ => {
                return spawn_gui_attached(args);
            }
        }
    }
    if cmd == "wallpaper" && args.get(2).is_some_and(|arg| arg.eq_ignore_ascii_case("get")) {
        // Mirrors `handle_wallpaper` folder resolution for scripts
        // (paridade `handle_sync`: pasta explícita = `args[3..]`).
        let folder = match args.get(3) {
            Some(_) => args[3..].join(" "),
            None => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                sway_manager_daemon::repos::wallpaper::wallpaper_folder(&home, &|path| {
                    std::path::Path::new(path).is_dir()
                })
            }
        };
        if !folder.is_empty() {
            stdout.print_stdout(&format!("{folder}\n"));
        }
        return 0;
    }
    if cmd == "osd" && args.get(2).is_none() {
        // Mirrors `handle_osd`: bare `osd` prints the notice on stdout.
        stdout.print_stdout("OSD não informado!\n");
        return 0;
    }
    spawn_gui_detached(args);
    0
}

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--standalone") {
        args.retain(|arg| arg != "--standalone");
        std::process::exit(local_invoke(&args, &mut Stdio));
    }
    if args.len() == 1 {
        println!("{HELP_TEXT}");
        return;
    }
    let app = args[1].to_lowercase();
    match app.as_str() {
        "-h" | "--help" | "help" => println!("{HELP_TEXT}"),
        "daemon" | "--daemon" | "-d" => {
            if args.len() > 2 && ["log", "logs", "-l"].contains(&args[2].to_lowercase().as_str()) {
                daemon_log(&args);
            } else {
                std::process::exit(run_daemon().await);
            }
        }
        "portal" => std::process::exit(local_invoke(&args, &mut Stdio)),
        _ => {
            if send_command(&args).await {
            } else {
                eprint!("{DAEMON_DOWN_MSG}");
                let _ = std::io::stderr().flush();
                std::process::exit(1);
            }
        }
    }
}

/// Test helper: forward to a custom socket (mirrors `send_command_to`).
pub async fn forward_to(socket: &std::path::Path, args: &[String], output: &mut dyn Output) -> bool {
    send_command_to(socket, args, output, std::time::Duration::from_secs(2)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    /// Serializa testes que mutam `HOME` (env global, race entre threads).
    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn expand_home_tilde() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: serializado pelo mutex acima.
        unsafe { std::env::set_var("HOME", "/home/tester") };
        assert_eq!(expand_home("~/foo"), "/home/tester/foo");
        assert_eq!(expand_home("/abs/path"), "/abs/path");
        unsafe { std::env::remove_var("HOME") };
    }

    #[test]
    fn count_desktop_files_nonnegative() {
        assert!(count_desktop_files() < 100_000);
    }

    #[test]
    fn refresh_lock_state_never_panics() {
        refresh_lock_state();
    }

    #[test]
    fn daemon_log_without_file_prints_notice() {
        let _guard = ENV_LOCK.lock().unwrap();
        // HOME without logs dir → "Nenhum log registrado...".
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("HOME", dir.path().to_str().unwrap()) };
        daemon_log(&["SwayManager".to_string(), "daemon".to_string(), "log".to_string()]);
        unsafe { std::env::remove_var("HOME") };
    }

    #[test]
    fn spawn_gui_detached_never_panics_without_binary() {
        spawn_gui_detached(&["SwayManager".to_string(), "settings".to_string()]);
        spawn_gui_detached(&["SwayManager".to_string()]);
    }

    #[test]
    fn attached_gui_cmd_carries_portal_subcommand() {
        // Regressão: o chooser precisa herdar o stdio do xdpw (anexado);
        // detached (`/dev/null`) = xdpw recebe EOF e nada compartilha.
        let cmd = attached_gui_cmd(&["SwayManager".to_string(), "portal".to_string()]);
        assert!(cmd[0].contains("SwayManagerGUI"), "{cmd:?}");
        assert_eq!(&cmd[1..], &["portal".to_string()]);
    }

    #[test]
    fn local_invoke_portal_without_binary_fails() {
        let mut output = Stdio;
        // Anexado: sem o binário GUI ao lado, o spawn falha e o exit
        // propaga 1 (antes: detached falhava silencioso e retornava 0).
        assert_eq!(local_invoke(&["SwayManager".to_string(), "portal".to_string()], &mut output), 1);
    }

    #[test]
    fn local_invoke_bare_osd_prints_notice() {
        // Mirrors `handle_osd` bare branch (`print("OSD não informado!")`).
        struct Capture(String);
        impl Output for Capture {
            fn print_stdout(&mut self, text: &str) {
                self.0.push_str(text);
            }
            fn print_stderr(&mut self, _text: &str) {}
        }
        let mut output = Capture(String::new());
        assert_eq!(local_invoke(&["SwayManager".to_string(), "osd".to_string()], &mut output), 0);
        assert_eq!(output.0, "OSD não informado!\n");
    }

    #[test]
    fn local_invoke_wallpaper_get_prints_folder() {
        // Regressão: imprimia `get /tmp` (`args[2..]` incluía o `get`).
        struct Capture(String);
        impl Output for Capture {
            fn print_stdout(&mut self, text: &str) {
                self.0.push_str(text);
            }
            fn print_stderr(&mut self, _text: &str) {}
        }
        let mut output = Capture(String::new());
        let code = local_invoke(
            &["SwayManager".to_string(), "wallpaper".to_string(), "get".to_string(), "/tmp".to_string()],
            &mut output,
        );
        assert_eq!(code, 0);
        assert_eq!(output.0, "/tmp\n");
    }

    #[tokio::test]
    async fn run_daemon_refuses_second_instance() {
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("XDG_RUNTIME_DIR", dir.path().to_str().unwrap()) };
        // Occupy the exact socket path the daemon will probe.
        let _listener = tokio::net::UnixListener::bind(socket_path()).unwrap();
        let code = run_daemon().await;
        assert_eq!(code, 0);
        unsafe { std::env::remove_var("XDG_RUNTIME_DIR") };
    }
}
