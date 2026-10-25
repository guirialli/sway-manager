//! Polkit authentication agent backend.
//!
//! Native replacement for the external LXDE-era agents (`lxpolkit`,
//! launched via `polkit-runner.sh`): registers the session on the
//! **system bus** as `org.freedesktop.PolicyKit1.AuthenticationAgent`,
//! drives the PAM conversation through the setuid helper, and enforces
//! the fail-secure policy learned from CVE-2021-3560.
//!
//! Layout mirrors the sibling repos: pure protocol core at the top
//! (`PolkitChallenge`, `HelperEvent`, line parser, redacted `Cookie`),
//! then the blocking helper conversation driver (`session`), then the
//! `zbus` system-bus service (`agent`).
//!
//! Contracts: `specs/domains/polkit.md` §5 + `specs/adrs/0004-polkit-agent.md`.
//! The cookie is only ever written to the helper stdin — never to argv,
//! env, logs, the GUI query string, or error strings (INV-POLKIT-3).
//! Password bytes are [`zeroize`]d after use (INV-POLKIT-4).
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use zeroize::Zeroize;

// ---------------------------------------------------------------------------
// Pure core
// ---------------------------------------------------------------------------

/// Well-known polkit D-Bus strings (also asserted in unit tests so a typo
/// fails loudly instead of silently never matching `polkitd`).
pub const AUTH_AGENT_IFACE: &str = "org.freedesktop.PolicyKit1.AuthenticationAgent";
pub const AUTH_AGENT_PATH: &str = "/org/swaymanager/PolicyKit1/AuthenticationAgent";
pub const AUTHORITY_NAME: &str = "org.freedesktop.PolicyKit1";
pub const AUTHORITY_PATH: &str = "/org/freedesktop/PolicyKit1/Authority";
pub const AUTHORITY_IFACE: &str = "org.freedesktop.PolicyKit1.Authority";
pub const CANCELLED_ERROR: &str = "org.freedesktop.PolicyKit1.Error.Cancelled";
pub const LOGIN1_NAME: &str = "org.freedesktop.login1";
pub const LOGIN1_MANAGER_PATH: &str = "/org/freedesktop/login1";
pub const LOGIN1_MANAGER_IFACE: &str = "org.freedesktop.login1.Manager";
pub const AGENT_HELPER_SOCKET: &str = "/run/polkit/agent-helper.socket";
pub const AGENT_HELPER_FALLBACK: &str = "/usr/lib/polkit-1/polkit-agent-helper-1";
pub const POLKIT_OBJECT_PATH_PREFIX: &str = "/org/freedesktop/PolicyKit1/AuthenticationAgent/";

/// Usage text for bare `SwayManager polkit` (mirrors the Python CLI style:
/// one purpose line + subcommand list).
pub const USAGE: &str = "\
Uso: SwayManager polkit <status|agent|dismiss>
  status    mostra o estado do agente (registered/not-registered)
  agent     executa o agente em foreground (para o autostart do Sway)
  dismiss   cancela o desafio de autenticação em andamento
";

/// In-flight authentication challenge delivered by polkitd's
/// `BeginAuthentication` call.
#[derive(Debug, Clone)]
pub struct PolkitChallenge {
    pub action_id: String,
    pub message: String,
    pub icon_name: String,
    pub details: HashMap<String, String>,
    pub cookie: Cookie,
    /// `(kind, details)` identity pairs as delivered on D-Bus
    /// (`a(sa{sv})`), e.g. `("unix-user", {"uid": 1000})`.
    pub identities: Vec<(String, HashMap<String, String>)>,
}

/// Opaque helper cookie. `Debug` is redacted so the secret can never leak
/// through logs or error strings (INV-POLKIT-3).
#[derive(Clone, Default)]
pub struct Cookie(Vec<u8>);

impl Cookie {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cookie([redacted])")
    }
}

/// Events parsed from one helper stdout line (upstream
/// `polkitagentsession.c` parity). Unknown lines map to `None` and the
/// caller must treat them as fail-secure `Completed(false)` (INV-POLKIT-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperEvent {
    Request { prompt: String, echo: bool },
    ShowError(String),
    ShowInfo(String),
    Completed(bool),
}

/// Parse one helper stdout line (`\n` already stripped).
///
/// `PAM_PROMPT_ECHO_OFF <t>` → `Request { echo: false }`,
/// `PAM_PROMPT_ECHO_ON <t>` → `Request { echo: true }`,
/// `PAM_ERROR_MSG <t>` → `ShowError`,
/// `PAM_TEXT_INFO <t>` → `ShowInfo`,
/// `SUCCESS` → `Completed(true)`, `FAILURE` → `Completed(false)`,
/// anything else (incl. empty/EOF) → `None` (fail-secure).
pub fn parse_helper_line(line: &str) -> Option<HelperEvent> {
    // Upstream uses `g_str_has_prefix` on the unescaped line; a bare
    // `SUCCESS`/`FAILURE` token with no payload counts as completion.
    if let Some(prompt) = line.strip_prefix("PAM_PROMPT_ECHO_OFF ") {
        return Some(HelperEvent::Request {
            prompt: prompt.to_string(),
            echo: false,
        });
    }
    if let Some(prompt) = line.strip_prefix("PAM_PROMPT_ECHO_ON ") {
        return Some(HelperEvent::Request {
            prompt: prompt.to_string(),
            echo: true,
        });
    }
    if let Some(text) = line.strip_prefix("PAM_ERROR_MSG ") {
        return Some(HelperEvent::ShowError(text.to_string()));
    }
    if let Some(text) = line.strip_prefix("PAM_TEXT_INFO ") {
        return Some(HelperEvent::ShowInfo(text.to_string()));
    }
    if line == "SUCCESS" {
        return Some(HelperEvent::Completed(true));
    }
    if line == "FAILURE" {
        return Some(HelperEvent::Completed(false));
    }
    None
}

/// Serialize a `password` response line for the helper stdin.
///
/// The returned buffer holds `<password>\n` (newline appended when the
/// caller did not include one, matching `polkit_agent_session_response`).
/// Callers MUST [`zeroize`](Zeroize::zeroize) the buffer after writing it
/// to the fd (INV-POLKIT-4).
pub fn response_bytes(password: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(password.len() + 1);
    out.extend_from_slice(password.as_bytes());
    if !password.ends_with('\n') {
        out.push(b'\n');
    }
    out
}

/// Write `password\n` to `writer`, then zeroize the staging buffer.
pub fn write_password(writer: &mut dyn Write, password: &str) -> std::io::Result<()> {
    let mut buf = response_bytes(password);
    let res = writer.write_all(&buf);
    buf.zeroize();
    res
}

/// One-line agent state for `polkit status` (`status_text` contract).
pub fn status_text(registered: bool, session: Option<&str>) -> String {
    if registered {
        match session {
            Some(id) if !id.is_empty() => format!("polkit: registered (session {id})\n"),
            _ => String::from("polkit: registered\n"),
        }
    } else {
        String::from("polkit: not-registered\n")
    }
}

/// Resolve a uid to its login name via `libc::getpwuid` (upstream
/// `polkitagentsession.c` uses the uid of the challenged identity).
pub fn username_for_uid(uid: u32) -> Option<String> {
    // SAFETY: `getpwuid` returns either NULL or a pointer to a static
    // struct; we copy the name bytes out before returning.
    unsafe {
        let entry = libc::getpwuid(uid);
        if entry.is_null() || (*entry).pw_name.is_null() {
            return None;
        }
        std::ffi::CStr::from_ptr((*entry).pw_name)
            .to_str()
            .ok()
            .map(|name| name.to_string())
    }
}

/// Resolve the current username (`getuid` → `getpwuid`), if any.
pub fn current_username() -> Option<String> {
    // SAFETY: `getuid` always succeeds and has no side effects.
    let uid = unsafe { libc::getuid() };
    username_for_uid(uid)
}

// ---------------------------------------------------------------------------
// Single-instance guard (pidfile)
// ---------------------------------------------------------------------------

/// Default pidfile location: `$XDG_RUNTIME_DIR/sway-manager-polkit.pid`
/// with `/tmp/user-$UID/` fallback (mirrors `socket.rs` layout).
pub fn pidfile_path() -> PathBuf {
    pidfile_path_from(std::env::var("XDG_RUNTIME_DIR").ok().as_deref())
}

fn pidfile_path_from(runtime_dir: Option<&str>) -> PathBuf {
    // SAFETY: `getuid` always succeeds.
    let uid = unsafe { libc::getuid() };
    let dir = runtime_dir
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/tmp/user-{uid}")));
    dir.join("sway-manager-polkit.pid")
}

fn pid_is_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // SAFETY: `kill(pid, 0)` performs no action beyond error checking.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

/// Claim the pidfile: refuse when a live process owns it, overwrite stale
/// entries. Returns the path on success; the caller removes it on exit.
pub fn claim_pidfile(path: &Path) -> Result<PathBuf, String> {
    if let Ok(previous) = std::fs::read_to_string(path)
        && let Ok(pid) = previous.trim().parse::<u32>()
        && pid_is_alive(pid)
    {
        return Err(format!("polkit agent already running (pid {pid})"));
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // SAFETY: `getpid` always succeeds.
    let pid = unsafe { libc::getpid() };
    std::fs::write(path, format!("{pid}\n"))
        .map_err(|err| format!("cannot write pidfile {}: {err}", path.display()))?;
    Ok(path.to_path_buf())
}

/// Remove the pidfile, ignoring errors (shutdown path).
pub fn release_pidfile(path: &Path) {
    let _ = std::fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// Helper session driver (blocking; runs on a dedicated thread)
// ---------------------------------------------------------------------------

/// Injectable helper endpoints for tests: socket path tried first
/// (polkit ≥ 127), then the setuid spawn fallback.
#[derive(Debug, Clone)]
pub struct SessionPaths {
    pub socket_path: PathBuf,
    pub helper_bin: PathBuf,
}

impl Default for SessionPaths {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from(AGENT_HELPER_SOCKET),
            helper_bin: PathBuf::from(AGENT_HELPER_FALLBACK),
        }
    }
}

/// Outcome of [`HelperSession::next_event_deadline`].
#[derive(Debug)]
pub enum NextEvent {
    Event(HelperEvent),
    Timeout,
    Closed,
}

/// Blocking PAM conversation with the setuid helper.
///
/// Prefer the `/run/polkit/agent-helper.socket` path (write
/// `<user>\n<cookie>\n`, read lines on the same fd); fall back to
/// spawning `<helper-bin> <username>` with `<cookie>\n` on stdin.
/// Line I/O goes through `read_line`; EOF/HUP/unknown line or spawn
/// failure all collapse to `Completed(false)` (INV-POLKIT-1).
pub struct HelperSession {
    /// Lines from the helper (socket fd or child stdout pipe).
    reader: Box<dyn std::io::BufRead + Send>,
    /// Bytes to the helper (socket fd or child stdin pipe).
    writer: Box<dyn Write + Send>,
    child: Option<Child>,
    /// Pollable fd for bounded waits (`None` → blocking reads).
    helper_fd: Option<RawFd>,
}

impl HelperSession {
    /// Open the conversation for `username` + `cookie`.
    pub fn start(username: &str, cookie: &Cookie, paths: &SessionPaths) -> std::io::Result<Self> {
        if Path::new(&paths.socket_path).exists() {
            match UnixStream::connect(&paths.socket_path) {
                Ok(stream) => return Self::over_socket(stream, username, cookie),
                Err(_) => { /* fall back to the setuid spawn, like upstream */ }
            }
        }
        Self::spawn_helper(&paths.helper_bin, username, cookie)
    }

    fn over_socket(
        stream: UnixStream,
        username: &str,
        cookie: &Cookie,
    ) -> std::io::Result<Self> {
        use std::os::unix::io::AsRawFd as _;
        let helper_fd = Some(stream.as_raw_fd());
        let mut writer = stream.try_clone()?;
        writer.write_all(username.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.write_all(cookie.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        let reader: Box<dyn std::io::BufRead + Send> =
            Box::new(std::io::BufReader::new(stream));
        Ok(Self {
            reader,
            writer: Box::new(writer),
            child: None,
            helper_fd,
        })
    }

    fn spawn_helper(
        helper_bin: &Path,
        username: &str,
        cookie: &Cookie,
    ) -> std::io::Result<Self> {
        use std::os::unix::io::AsRawFd as _;
        // Retry `ETXTBSY` briefly: a just-installed helper binary can
        // still be open for writing when the agent spawns it.
        let mut attempt = 0;
        let mut child = loop {
            match std::process::Command::new(helper_bin)
                .arg(username)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(child) => break child,
                Err(err)
                    if err.raw_os_error() == Some(libc::ETXTBSY) && attempt < 10 =>
                {
                    attempt += 1;
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(err) => return Err(err),
            }
        };
        let mut stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "helper stdin unavailable")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "helper stdout unavailable")
        })?;
        let helper_fd = Some(stdout.as_raw_fd());
        stdin.write_all(cookie.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        let reader: Box<dyn std::io::BufRead + Send> = Box::new(std::io::BufReader::new(stdout));
        Ok(Self {
            reader,
            writer: Box::new(stdin),
            child: Some(child),
            helper_fd,
        })
    }

    /// Read the next helper event. `Ok(None)` means EOF/HUP (fail-secure:
    /// the caller completes with `false`).
    pub fn next_event(&mut self) -> std::io::Result<Option<HelperEvent>> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        while line.ends_with('\n') || line.ends_with('\r') {
            line.pop();
        }
        // Trace (stderr): helper stdout never carries secrets (passwords
        // flow stdin-only), so logging the head is safe and decisive.
        eprintln!("polkit agent: helper line len={} head={:?}", line.len(), line.chars().take(24).collect::<String>());
        // Upstream `g_strcompress` unescapes `\\n`-style sequences before
        // prefix matching; mirror that so payloads decode identically.
        let unescaped = unescape_line(&line);
        Ok(Some(parse_helper_line(&unescaped).unwrap_or(HelperEvent::Completed(false))))
    }

    /// `next_event` after a bounded `poll(2)` wait on the helper fd:
    /// `Timeout` when nothing arrives within `timeout`, `Closed` on
    /// EOF/HUP/`POLLERR`. Fail-secure mapping stays with the caller
    /// (`Closed` ends the challenge with `false`; `Timeout` keeps the
    /// relay responsive to cancel/GUI traffic).
    pub fn next_event_deadline(&mut self, timeout: Duration) -> std::io::Result<NextEvent> {
        if let Some(fd) = self.helper_fd() {
            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: valid single-entry pollfd; bounded wait.
            let ready = unsafe { libc::poll(&mut pollfd, 1, timeout.as_millis() as libc::c_int) };
            if ready == 0 {
                return Ok(NextEvent::Timeout);
            }
            if ready < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if pollfd.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
                return Ok(NextEvent::Closed);
            }
        }
        match self.next_event()? {
            Some(event) => Ok(NextEvent::Event(event)),
            None => Ok(NextEvent::Closed),
        }
    }

    /// Pollable fd captured at construction (`None` → blocking reads).
    fn helper_fd(&self) -> Option<RawFd> {
        self.helper_fd
    }

    /// Forward the password to the helper, zeroizing the staging buffer.
    pub fn respond(&mut self, password: &str) -> std::io::Result<()> {
        write_password(&mut *self.writer, password)
    }

    /// `true` while a spawned helper child is still referenced.
    pub fn has_child(&self) -> bool {
        self.child.is_some()
    }

    /// Cancel: SIGTERM + `waitpid`, then SIGKILL after a grace period
    /// when the helper ignores SIGTERM, and close the fds
    /// (INV-POLKIT-7: no orphaned setuid process).
    pub fn cancel(&mut self) {
        self.cancel_with_grace(Duration::from_millis(500));
    }

    fn cancel_with_grace(&mut self, grace: Duration) {
        use std::os::unix::process::ExitStatusExt as _;

        if let Some(mut child) = self.child.take() {
            let pid = child.id();
            // Send SIGTERM first (matching upstream `kill_helper`).
            let _ = child.kill();
            // Bounded `waitpid` loop: upstream blocks indefinitely, but
            // `CancelAuthentication` must not hang the D-Bus reply.
            let deadline = std::time::Instant::now() + grace;
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if std::time::Instant::now() >= deadline {
                            // SAFETY: raw SIGKILL to our own helper child.
                            unsafe { libc::kill(pid as libc::pid_t, libc::SIGKILL) };
                            let _ = child.wait();
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
            // Reap zombies even if signalling failed mid-flight.
            let _ = child.try_wait().map(|status| {
                if let Some(status) = status {
                    let _ = status.signal();
                }
            });
            let _ = child.wait();
        }
        // Dropping reader/writer closes the socket fds / pipes.
    }
}

impl Drop for HelperSession {
    fn drop(&mut self) {
        self.cancel_with_grace(Duration::from_millis(100));
    }
}

/// Unescape `\\`-sequences the way upstream `g_strcompress` does before
/// prefix matching (`\n`, `\r`, `\t`, `\\`, octal `\ooo`).
fn unescape_line(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 1;
            match bytes[i] {
                b'n' => out.push(b'\n'),
                b'r' => out.push(b'\r'),
                b't' => out.push(b'\t'),
                b'\\' => out.push(b'\\'),
                b'\'' => out.push(b'\''),
                b'"' => out.push(b'"'),
                b'0'..=b'7' => {
                    let mut value: u8 = bytes[i] - b'0';
                    let mut consumed = 1;
                    while consumed < 3 && i + 1 < bytes.len() && matches!(bytes[i + 1], b'0'..=b'7') {
                        i += 1;
                        value = value.wrapping_mul(8).wrapping_add(bytes[i] - b'0');
                        consumed += 1;
                    }
                    out.push(value);
                }
                other => {
                    out.push(b'\\');
                    out.push(other);
                }
            }
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---------------------------------------------------------------------------
// D-Bus agent service (`zbus 5`)
// ---------------------------------------------------------------------------

/// Cloneable handle to the in-flight challenge; `dismiss` cancels through it.
#[derive(Debug, Clone)]
pub struct AgentHandle {
    inner: Arc<AgentState>,
}

#[derive(Debug, Default)]
struct AgentState {
    cancelled: Mutex<bool>,
    registered: Mutex<bool>,
    session_id: Mutex<Option<String>>,
}

impl AgentHandle {
    /// Request cancellation of the in-flight challenge (if any).
    /// The relay loop observes the flag between helper/GUI polls.
    pub fn cancel_current(&self) {
        *self.inner.cancelled.lock().unwrap() = true;
    }

    /// `true` once `RegisterAuthenticationAgent` succeeded.
    pub fn is_registered(&self) -> bool {
        *self.inner.registered.lock().unwrap()
    }

    /// Last resolved logind session id, if any.
    pub fn session_id(&self) -> Option<String> {
        self.inner.session_id.lock().unwrap().clone()
    }

    /// One-line state for the `polkit status` socket command.
    pub fn status_line(&self) -> String {
        let registered = self.is_registered();
        let session = self.session_id();
        status_text(registered, session.as_deref())
    }
}

impl Default for AgentHandle {
    fn default() -> Self {
        Self {
            inner: Arc::new(AgentState::default()),
        }
    }
}

/// Resolve the logind session id: `XDG_SESSION_ID` →
/// `org.freedesktop.login1.Manager.GetSessionByPID(getpid())` →
/// fail-secure abort (INV-POLKIT-2).
pub async fn resolve_session_id(
    conn: &zbus::Connection,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(id) = std::env::var("XDG_SESSION_ID")
        && !id.trim().is_empty()
    {
        return Ok(id);
    }
    // SAFETY: `getpid` always succeeds.
    let pid = unsafe { libc::getpid() } as u32;
    let proxy = zbus::Proxy::new(
        conn,
        LOGIN1_NAME,
        LOGIN1_MANAGER_PATH,
        LOGIN1_MANAGER_IFACE,
    )
    .await?;
    let path: zbus::zvariant::OwnedObjectPath =
        proxy.call("GetSessionByPID", &(pid,)).await?;
    let session = path
        .as_ref()
        .as_str()
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    if session.is_empty() {
        return Err("logind returned an empty session path".into());
    }
    Ok(session)
}

/// Current locale for the register call: `LC_ALL` → `LC_MESSAGES` →
/// `LANG` → `en_US.UTF-8`.
pub fn current_locale() -> String {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            let value = value.trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    "en_US.UTF-8".to_string()
}

/// Read `LockedHint` on a logind session path. Query errors propagate so
/// the caller can apply the repo decision (proceed + warn: only an
/// explicit `LockedHint=true` vetoes; see `agent::locked_hint`).
pub async fn read_locked_hint(
    conn: &zbus::Connection,
    session_path: &zbus::zvariant::ObjectPath<'_>,
) -> zbus::Result<bool> {
    let proxy = zbus::Proxy::new(
        conn,
        LOGIN1_NAME,
        session_path,
        "org.freedesktop.login1.Session",
    )
    .await?;
    proxy.get_property::<bool>("LockedHint").await
}

/// Disable core dumps for the agent process (`prctl(PR_SET_DUMPABLE, 0)`,
/// KDE parity); best-effort, never fatal.
fn drop_dumpable() {
    // SAFETY: `prctl(PR_SET_DUMPABLE, 0)` is always a valid call.
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
    }
}

/// The `zbus` agent module: system-bus object, authority registration,
/// per-challenge GUI+helper relay. Only compiled against the real bus;
/// unit tests cover the pure core above without D-Bus.
pub mod agent {
    use super::*;

    /// D-Bus error carrying `org.freedesktop.PolicyKit1.Error.Cancelled`.
    #[derive(Debug, zbus::DBusError)]
    #[zbus(prefix = "org.freedesktop.PolicyKit1.Error")]
    pub enum AgentError {
        #[zbus(name = "Cancelled")]
        Cancelled(String),
        #[zbus(name = "Failed")]
        Failed(String),
    }

    /// Per-challenge guard shared between the D-Bus object and the
    /// `CancelAuthentication` path: the active cookie bytes (single
    /// challenge at a time — INV-POLKIT-5).
    pub struct ChallengeSlot {
        cookie: Mutex<Option<Vec<u8>>>,
    }

    impl ChallengeSlot {
        fn new() -> Self {
            Self {
                cookie: Mutex::new(None),
            }
        }
    }

    /// The exported `AuthenticationAgent` object. Holds the `zbus`
    /// connection for re-registration plus the single-challenge guard
    /// (`m_inProgress` parity: a second `BeginAuthentication` while one
    /// is active is rejected, never queued — INV-POLKIT-5).
    pub struct PolkitAgent {
        conn: zbus::Connection,
        handle: AgentHandle,
        slot: Arc<ChallengeSlot>,
        gui_bin: OsString,
        session_exe: SessionExe,
    }

    /// How a challenge resolves identities to a helper username.
    #[derive(Debug, Clone)]
    struct SessionExe {
        paths: SessionPaths,
    }

    impl PolkitAgent {
        pub fn new(
            conn: zbus::Connection,
            handle: AgentHandle,
            gui_bin: OsString,
        ) -> Self {
            Self {
                conn,
                handle,
                slot: Arc::new(ChallengeSlot::new()),
                gui_bin,
                session_exe: SessionExe {
                    paths: SessionPaths::default(),
                },
            }
        }

    }

    #[zbus::interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
    impl PolkitAgent {
        async fn begin_authentication(
            &self,
            action_id: String,
            message: String,
            icon_name: String,
            details: HashMap<String, String>,
            cookie: String,
            identities: Vec<(
                String,
                HashMap<String, zbus::zvariant::OwnedValue>,
            )>,
        ) -> Result<(), AgentError> {
            // Trace log (stderr): challenges are rare; observability beats
            // silence when the dialog fails to appear. Never logs the cookie.
            eprintln!("polkit agent: challenge action_id={action_id} identities={}", identities.len());
            // LockedHint veto: only explicit `true` refuses (repo
            // decision: query errors proceed + warn).
            if locked_hint_active(&self.conn).await {
                return Err(AgentError::Cancelled("session is locked".to_string()));
            }

            // Single-challenge guard.
            {
                let mut current = self.slot.cookie.lock().unwrap();
                if current.is_some() {
                    return Err(AgentError::Cancelled(
                        "authentication already in progress".to_string(),
                    ));
                }
                *current = Some(cookie.as_bytes().to_vec());
            }

            let identities: Vec<(String, HashMap<String, String>)> = identities
                .into_iter()
                .map(|(kind, attrs)| {
                    let attrs = attrs
                        .into_iter()
                        .map(|(key, value)| {
                            let text = value.to_string().trim_matches('"').to_string();
                            (key, text)
                        })
                        .collect();
                    (kind, attrs)
                })
                .collect();
            let challenge = PolkitChallenge {
                action_id,
                message,
                icon_name,
                details,
                cookie: Cookie::new(cookie.into_bytes()),
                identities,
            };

            // Run the blocking helper+GUI relay on a dedicated thread
            // so the D-Bus executor stays responsive to
            // `CancelAuthentication` (which flips the handle's cancel
            // flag, polled by the relay loop). `spawn_blocking` (not
            // `block_in_place`) because zbus method handlers may run on
            // the async-io executor thread where blocking is forbidden.
            let gui_bin = self.gui_bin.clone();
            let paths = self.session_exe.paths.clone();
            let handle = self.handle.clone();
            let outcome = tokio::task::spawn_blocking(move || {
                eprintln!("polkit agent: relay start");
                let out = relay_challenge_cancelable(&gui_bin, &paths, &handle, challenge);
                eprintln!("polkit agent: relay end out={out:?}");
                out
            })
            .await
            .map_err(|err| AgentError::Failed(format!("relay thread failed: {err}")))?;

            *self.slot.cookie.lock().unwrap() = None;

            match outcome {
                Ok(true) => Ok(()),
                Ok(false) => Err(AgentError::Cancelled("authentication dismissed".to_string())),
                Err(err) => {
                    eprintln!("polkit agent: relay error: {err}");
                    Err(AgentError::Failed(err))
                }
            }
        }

        async fn cancel_authentication(&self, cookie: String) -> Result<(), AgentError> {
            let active = self
                .slot
                .cookie
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|current| current == cookie.as_bytes());
            if active {
                self.handle.cancel_current();
            }
            Ok(())
        }
    }

    async fn locked_hint_active(conn: &zbus::Connection) -> bool {
        let session_id = match resolve_session_id(conn).await {
            Ok(id) => id,
            Err(_) => return false,
        };
        let path = format!("/org/freedesktop/login1/session/{session_id}");
        let object_path = match zbus::zvariant::ObjectPath::try_from(path.as_str()) {
            Ok(path) => path,
            Err(_) => return false,
        };
        read_locked_hint(conn, &object_path).await.unwrap_or_default()
    }

    /// Serve the agent until shutdown: object export, authority
    /// registration (3× retry), `NameOwnerChanged` re-registration
    /// watch. Returns `Err` (fail-secure) when no logind session
    /// resolves or polkitd registration fails after retries.
    pub async fn serve(
        handle: AgentHandle,
        gui_bin: OsString,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        drop_dumpable();
        let conn = zbus::connection::Builder::system()?.build().await?;
        let session_id = resolve_session_id(&conn).await.map_err(|err| {
            eprintln!("polkit agent: cannot resolve logind session: {err}");
            err
        })?;
        *handle.inner.session_id.lock().unwrap() = Some(session_id.clone());

        let object_path = zbus::zvariant::ObjectPath::try_from(AUTH_AGENT_PATH)?;
        let agent = PolkitAgent::new(conn.clone(), handle.clone(), gui_bin);
        conn.object_server().at(&object_path, agent).await?;

        register_agent_retry(&conn, &session_id, &object_path).await?;
        *handle.inner.registered.lock().unwrap() = true;

        // Re-register when polkitd reappears on the bus; cancel the
        // in-flight challenge when it vanishes (fail-secure: the dialog
        // closes, the action errors, nothing is ever approved).
        spawn_reregister_watch(
            conn.clone(),
            handle.clone(),
            object_path.clone(),
            session_id.clone(),
        );

        // Block until SIGINT/SIGTERM (the installer smoke test and Sway
        // use plain `kill`); unregister on shutdown.
        shutdown_signal().await;
        unregister_agent(&conn, &session_id, &object_path).await.ok();
        *handle.inner.registered.lock().unwrap() = false;
        Ok(())
    }

    /// Resolve when SIGINT or SIGTERM arrives (Unix).
    async fn shutdown_signal() {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            match signal(SignalKind::terminate()) {
                Ok(mut term) => {
                    tokio::select! {
                        _ = tokio::signal::ctrl_c() => {}
                        _ = term.recv() => {}
                    }
                }
                Err(_) => {
                    tokio::signal::ctrl_c().await.ok();
                }
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await.ok();
        }
    }
    fn spawn_reregister_watch(
        conn: zbus::Connection,
        handle: AgentHandle,
        path: zbus::zvariant::ObjectPath<'static>,
        session_id: String,
    ) {
        tokio::spawn(async move {
            use futures_lite::StreamExt as _;
            let Ok(proxy) = zbus::fdo::DBusProxy::new(&conn).await else {
                return;
            };
            let Ok(mut stream) = proxy
                .receive_name_owner_changed_with_args(&[(0u8, AUTHORITY_NAME)])
                .await
            else {
                return;
            };
            while let Some(signal) = stream.next().await {
                let Ok(args) = signal.args() else { continue };
                if args.new_owner.is_none() {
                    // polkitd vanished: mark unregistered + cancel.
                    *handle.inner.registered.lock().unwrap() = false;
                    handle.cancel_current();
                    continue;
                }
                // polkitd (re)appeared: re-register (retry loop).
                let mut attempts = 0;
                while attempts < 3 {
                    match register_agent(&conn, &session_id, &path).await {
                        Ok(()) => {
                            *handle.inner.registered.lock().unwrap() = true;
                            break;
                        }
                        Err(err) => {
                            attempts += 1;
                            eprintln!("polkit agent: re-register failed ({err}); retry {attempts}/3");
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            }
        });
    }

    /// Standalone foreground entry for `SwayManager polkit agent`
    /// (also spawned detached by `run_daemon` at startup).
    pub async fn run_foreground() -> i32 {
        let handle = AgentHandle::default();
        // Sibling-first resolution (compiled layout: `SwayManagerGUI` next
        // to this binary); `SWAY_MANAGER_GUI` env overrides for tests.
        let gui_bin = std::env::var_os("SWAY_MANAGER_GUI").unwrap_or_else(|| {
            OsString::from(
                crate::binary::binary_command("SwayManagerGUI")
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| "SwayManagerGUI".to_string()),
            )
        });
        // Single-instance guard (pidfile claimed/released here; `serve`
        // no longer touches it so `run_daemon`'s embedded spawn can share
        // the same path).
        let pidfile = super::pidfile_path();
        if let Err(err) = super::claim_pidfile(&pidfile) {
            eprintln!("polkit agent: {err}");
            return 1;
        }
        let result = serve(handle, gui_bin).await;
        super::release_pidfile(&pidfile);
        match result {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("polkit agent: {err}");
                1
            }
        }
    }

    async fn register_agent_retry(
        conn: &zbus::Connection,
        session_id: &str,
        path: &zbus::zvariant::ObjectPath<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut last: Option<Box<dyn std::error::Error + Send + Sync>> = None;
        for _ in 0..3 {
            match register_agent(conn, session_id, path).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    last = Some(err);
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
        Err(last.unwrap_or_else(|| "registration failed".into()))
    }

    async fn register_agent(
        conn: &zbus::Connection,
        session_id: &str,
        path: &zbus::zvariant::ObjectPath<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use zbus::zvariant::{OwnedValue, Value};
        let mut attrs = HashMap::new();
        attrs.insert(
            "session-id".to_string(),
            OwnedValue::try_from(Value::from(session_id))?,
        );
        let subject = ("unix-session".to_string(), attrs);
        let proxy = zbus::Proxy::new(conn, AUTHORITY_NAME, AUTHORITY_PATH, AUTHORITY_IFACE).await?;
        let locale = super::current_locale();
        let path_str = path.as_str();
        let _: () = proxy
            .call("RegisterAuthenticationAgent", &(subject, locale, path_str))
            .await?;
        Ok(())
    }

    async fn unregister_agent(
        conn: &zbus::Connection,
        session_id: &str,
        path: &zbus::zvariant::ObjectPath<'_>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use zbus::zvariant::{OwnedValue, Value};
        let mut attrs = HashMap::new();
        attrs.insert(
            "session-id".to_string(),
            OwnedValue::try_from(Value::from(session_id))?,
        );
        let subject = ("unix-session".to_string(), attrs);
        let proxy = zbus::Proxy::new(conn, AUTHORITY_NAME, AUTHORITY_PATH, AUTHORITY_IFACE).await?;
        let path_str = path.as_str();
        let _: () = proxy
            .call("UnregisterAuthenticationAgent", &(subject, path_str))
            .await?;
        Ok(())
    }

    // -- per-challenge relay (blocking; dedicated thread) ------------------

    /// GUI→BE pipe line (`{"password":"…"}` | `{"cancel":true}`).
    #[derive(Debug)]
    enum GuiRequest {
        Password(String),
        Cancel,
        Invalid,
    }

    fn parse_gui_line(line: &str) -> GuiRequest {
        let line = line.trim();
        if line.is_empty() {
            return GuiRequest::Invalid;
        }
        // Minimal JSON-line parse (no serde_json here to keep the relay
        // dependency-free): look for the two known shapes.
        if line.contains("\"cancel\"") && line.contains("true") {
            return GuiRequest::Cancel;
        }
        if let Some(rest) = line.split("\"password\"").nth(1) {
            let rest = rest.trim_start_matches([':', ' ', '\t', '"']);
            // Extract until the closing quote (passwords cannot contain
            // `\n`; reject escapes minimally).
            let mut password = String::new();
            let mut chars = rest.chars();
            let mut closed = false;
            while let Some(ch) = chars.next() {
                match ch {
                    '"' => {
                        closed = true;
                        break;
                    }
                    '\\' => {
                        if let Some(escaped) = chars.next() {
                            match escaped {
                                'n' | 'r' => return GuiRequest::Invalid,
                                '"' => password.push('"'),
                                '\\' => password.push('\\'),
                                _ => {
                                    password.push('\\');
                                    password.push(escaped);
                                }
                            }
                        }
                    }
                    _ => password.push(ch),
                }
            }
            if closed && !password.contains('\n') {
                return GuiRequest::Password(password);
            }
            return GuiRequest::Invalid;
        }
        GuiRequest::Invalid
    }

    fn gui_event_line(prompt: &str, echo: bool) -> String {
        format!(
            "{{\"prompt\":{},\"echo\":{}}}\n",
            json_escape(prompt),
            if echo { "true" } else { "false" }
        )
    }

    fn json_escape(text: &str) -> String {
        let mut out = String::with_capacity(text.len() + 2);
        out.push('"');
        for ch in text.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                ch if (ch as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", ch as u32)),
                ch => out.push(ch),
            }
        }
        out.push('"');
        out
    }

    /// Spawn the GUI child for one challenge: argv per the §5 contract
    /// with pipe fds passed as `--pipe <fd> --events <fd>`; CLOEXEC is
    /// cleared on the child ends via `libc` before exec so only the two
    /// relay fds (plus stdio) survive.
    fn spawn_gui_child(
        gui_bin: &std::ffi::OsStr,
        challenge: &PolkitChallenge,
        gui_read_fd: RawFd,
        be_read_fd: RawFd,
    ) -> std::io::Result<Child> {
        use std::os::unix::process::CommandExt as _;

        let identities_csv = challenge
            .identities
            .iter()
            .map(|(kind, attrs)| {
                // `unix-user` identities resolve to `unix-user:<name>`;
                // anything else keeps the raw kind label so the GUI can
                // still display the list (password applies to the first).
                if kind == "unix-user" {
                    if let Some(uid) = attrs.get("uid").and_then(|uid| uid.parse::<u32>().ok())
                        && let Some(name) = super::username_for_uid(uid)
                    {
                        return format!("unix-user:{name}");
                    }
                    if let Some(name) = attrs.get("name") {
                        return format!("unix-user:{name}");
                    }
                }
                kind.clone()
            })
            .collect::<Vec<_>>()
            .join(",");
        let args = [
            OsString::from("polkit"),
            OsString::from("--action-id"),
            OsString::from(&challenge.action_id),
            OsString::from("--message"),
            OsString::from(&challenge.message),
            OsString::from("--identities"),
            OsString::from(identities_csv),
            OsString::from("--pipe"),
            OsString::from(gui_read_fd.to_string()),
            OsString::from("--events"),
            OsString::from(be_read_fd.to_string()),
        ];
        let mut cmd = std::process::Command::new(gui_bin);
        cmd.args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        // SAFETY: `pre_exec` runs between fork and exec; only async-safe
        // `fcntl` calls happen here. CLOEXEC is cleared on the two relay
        // fds so they survive exec; every other fd stays as-is
        // (`relay_pair` creates its parent ends CLOEXEC).
        unsafe {
            cmd.pre_exec(move || {
                for fd in [gui_read_fd, be_read_fd] {
                    let flags = libc::fcntl(fd, libc::F_GETFD);
                    if flags != -1 {
                        libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC);
                    }
                }
                Ok(())
            });
        }
        cmd.spawn()
    }

    /// Create a `socketpair` whose child ends have CLOEXEC cleared
    /// (inherited across exec) and whose parent ends are owned `File`s.
    fn relay_pair() -> std::io::Result<(std::fs::File, RawFd)> {
        let mut fds = [0 as RawFd; 2];
        // SAFETY: `socketpair` fills `fds` on success.
        let ret = unsafe {
            libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr())
        };
        if ret != 0 {
            return Err(std::io::Error::last_os_error());
        }
        // Parent end: CLOEXEC (never leaks into exec'd children).
        // SAFETY: valid fresh fds.
        unsafe {
            for fd in fds {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags != -1 {
                    libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
                }
            }
            // Child end: clear CLOEXEC so it survives exec.
            let flags = libc::fcntl(fds[1], libc::F_GETFD);
            if flags != -1 {
                libc::fcntl(fds[1], libc::F_SETFD, flags & !libc::FD_CLOEXEC);
            }
            Ok((std::fs::File::from_raw_fd(fds[0]), fds[1]))
        }
    }

    /// Blocking relay for one challenge: spawn helper session + GUI
    /// child, shuttle prompts/responses until `Completed`, reap both.
    /// Returns `Ok(true)` on successful authentication.
    fn relay_challenge_cancelable(
        gui_bin: &std::ffi::OsStr,
        paths: &SessionPaths,
        handle: &AgentHandle,
        challenge: PolkitChallenge,
    ) -> Result<bool, String> {
        // Resolve the helper username from the first `unix-user`
        // identity; fall back to the current user.
        let username = challenge
            .identities
            .iter()
            .find(|(kind, _)| kind == "unix-user")
            .and_then(|(_, attrs)| {
                attrs
                    .get("uid")
                    .and_then(|uid| uid.parse::<u32>().ok())
                    .and_then(super::username_for_uid)
                    .or_else(|| attrs.get("name").cloned())
            })
            .or_else(super::current_username)
            .ok_or_else(|| "cannot resolve helper username".to_string())?;

        eprintln!("polkit agent: helper user={username}");
        let mut session = HelperSession::start(&username, &challenge.cookie, paths)
            .map_err(|err| format!("helper start failed: {err}"))?;
        eprintln!("polkit agent: helper started");

        // Relay pipes: BE writes prompts to `events_w`, reads GUI
        // passwords from `pipe_r`.
        let (pipe_parent, pipe_child_fd) = relay_pair().map_err(|err| err.to_string())?;
        let (events_parent, events_child_fd) = relay_pair().map_err(|err| err.to_string())?;
        // `pipe_parent` = BE read end (fd 3 for GUI writes);
        // `events_parent` = BE write end (fd 4 for GUI reads).
        eprintln!("polkit agent: spawning GUI {gui_bin:?}");
        let mut gui = spawn_gui_child(gui_bin, &challenge, pipe_child_fd, events_child_fd)
            .map_err(|err| format!("GUI spawn failed: {err}"))?;
        eprintln!("polkit agent: GUI pid={}", gui.id());
        // SAFETY: the child ends were dup'd into the forked GUI before
        // exec; close our copies in the parent.
        unsafe {
            libc::close(pipe_child_fd);
            libc::close(events_child_fd);
        }

        let mut pipe_reader = std::io::BufReader::new(pipe_parent);
        let mut events_writer = events_parent;

        // Initial prompt comes from the first helper line; drive the
        // loop line-by-line with periodic cancel checks.
        let outcome = relay_loop(
            &mut session,
            &mut gui,
            &mut pipe_reader,
            &mut events_writer,
            handle,
        );

        // Reap GUI + helper (helper reaped via `cancel`/Drop).
        let _ = gui.kill();
        let _ = gui.wait();
        session.cancel();
        drop(events_writer);

        outcome
    }

    #[allow(clippy::too_many_arguments)]
    fn relay_loop(
        session: &mut HelperSession,
        gui: &mut Child,
        pipe_reader: &mut std::io::BufReader<std::fs::File>,
        events_writer: &mut std::fs::File,
        handle: &AgentHandle,
    ) -> Result<bool, String> {
        use std::os::unix::io::AsRawFd as _;
        // `poll(2)` on both fds with a 50ms quantum: helper traffic and
        // GUI traffic are both drained without blocking, so
        // `CancelAuthentication` (flag polled each lap) stays
        // responsive even when the helper goes silent.
        let pipe_fd = pipe_reader.get_ref().as_raw_fd();
        let mut gui_line = String::new();

        loop {
            if *handle.inner.cancelled.lock().unwrap() {
                let _ = events_writer.write_all(b"{\"completed\":false}\n");
                return Ok(false);
            }
            if gui.try_wait().map_err(|err| err.to_string())?.is_some() {
                // GUI died mid-challenge → treat as cancel (fail-secure).
                return Ok(false);
            }

            // Drain any pending GUI request without blocking.
            if drain_gui_request(pipe_reader, pipe_fd, &mut gui_line, session, events_writer)? {
                return Ok(false);
            }

            // Bounded wait for the next helper line; `Timeout` laps keep
            // the loop responsive to cancel/GUI traffic.
            match session.next_event_deadline(Duration::from_millis(50)) {
                Ok(NextEvent::Timeout) => continue,
                Ok(NextEvent::Event(HelperEvent::Request { prompt, echo })) => {
                    let line = gui_event_line(&prompt, echo);
                    let _ = events_writer.write_all(line.as_bytes());
                }
                Ok(NextEvent::Event(HelperEvent::ShowError(text))) => {
                    let line = format!("{{\"error\":{}}}\n", json_escape(&text));
                    let _ = events_writer.write_all(line.as_bytes());
                }
                Ok(NextEvent::Event(HelperEvent::ShowInfo(text))) => {
                    let line = format!("{{\"info\":{}}}\n", json_escape(&text));
                    let _ = events_writer.write_all(line.as_bytes());
                }
                Ok(NextEvent::Event(HelperEvent::Completed(ok))) => {
                    let line = format!(
                        "{{\"completed\":{}}}\n",
                        if ok { "true" } else { "false" }
                    );
                    let _ = events_writer.write_all(line.as_bytes());
                    return Ok(ok);
                }
                Ok(NextEvent::Closed) => {
                    // EOF/HUP → fail-secure.
                    let _ = events_writer.write_all(b"{\"completed\":false}\n");
                    return Ok(false);
                }
                Err(_) => {
                    let _ = events_writer.write_all(b"{\"completed\":false}\n");
                    return Ok(false);
                }
            }
        }
    }

    /// Drain one pending GUI pipe line, if any. Returns `Ok(true)` when
    /// the challenge ends here (cancel / pipe closed).
    fn drain_gui_request(
        pipe_reader: &mut std::io::BufReader<std::fs::File>,
        pipe_fd: RawFd,
        gui_line: &mut String,
        session: &mut HelperSession,
        events_writer: &mut std::fs::File,
    ) -> Result<bool, String> {
        let mut fds = libc::pollfd {
            fd: pipe_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: valid single-entry pollfd; zero timeout = non-blocking.
        let ready = unsafe { libc::poll(&mut fds, 1, 0) };
        if ready <= 0 || fds.revents & (libc::POLLIN | libc::POLLHUP) == 0 {
            return Ok(false);
        }
        gui_line.clear();
        match std::io::BufRead::read_line(pipe_reader, gui_line) {
            Ok(0) => Ok(true), // GUI closed the pipe → cancel.
            Ok(_) => match parse_gui_line(gui_line) {
                GuiRequest::Password(password) => {
                    let mut password = password;
                    let res = session
                        .respond(&password)
                        .map_err(|err| format!("helper write failed: {err}"));
                    password.zeroize();
                    res?;
                    Ok(false)
                }
                GuiRequest::Cancel => {
                    let _ = events_writer.write_all(b"{\"completed\":false}\n");
                    Ok(true)
                }
                GuiRequest::Invalid => {
                    // Ignore malformed GUI lines (never fail open).
                    Ok(false)
                }
            }
            Err(_) => Ok(true),
        }
    }


    #[cfg(test)]
    pub(crate) mod test_hooks {
        use super::*;
        pub fn parse_gui(text: &str) -> String {
            format!("{:?}", parse_gui_line(text))
        }
        pub fn escape(text: &str) -> String {
            json_escape(text)
        }
        pub fn event(prompt: &str, echo: bool) -> String {
            gui_event_line(prompt, echo)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- parser table (upstream parity + fail-secure unknowns) ----------

    #[test]
    fn parser_prompt_echo_off() {
        assert_eq!(
            parse_helper_line("PAM_PROMPT_ECHO_OFF Password: "),
            Some(HelperEvent::Request {
                prompt: "Password: ".to_string(),
                echo: false,
            })
        );
    }

    #[test]
    fn parser_prompt_echo_on() {
        assert_eq!(
            parse_helper_line("PAM_PROMPT_ECHO_ON Username: "),
            Some(HelperEvent::Request {
                prompt: "Username: ".to_string(),
                echo: true,
            })
        );
    }

    #[test]
    fn parser_error_and_info() {
        assert_eq!(
            parse_helper_line("PAM_ERROR_MSG sorry"),
            Some(HelperEvent::ShowError("sorry".to_string()))
        );
        assert_eq!(
            parse_helper_line("PAM_TEXT_INFO hello"),
            Some(HelperEvent::ShowInfo("hello".to_string()))
        );
    }

    #[test]
    fn parser_success_failure() {
        assert_eq!(parse_helper_line("SUCCESS"), Some(HelperEvent::Completed(true)));
        assert_eq!(
            parse_helper_line("FAILURE"),
            Some(HelperEvent::Completed(false))
        );
    }

    #[test]
    fn parser_unknown_is_none_fail_secure() {
        // Unknown line / empty / EOF-ish → None; the driver maps None +
        // unknown to `Completed(false)` (INV-POLKIT-1). `SUCCESS`/`FAILURE`
        // require an exact match: trailing garbage is rejected, never
        // trusted as success.
        for line in [
            "",
            "HELLO",
            "PAM_PROMPT_ECHO_OFF",
            "success",
            "FAIL",
            "  SUCCESS",
            "SUCCESS ",
            "SUCCESS extra",
            "FAILURE ",
            "FAILURE extra",
        ] {
            assert_eq!(parse_helper_line(line), None, "{line:?}");
        }
    }

    #[test]
    fn cookie_debug_is_redacted() {
        let cookie = Cookie::new(b"super-secret".to_vec());
        let shown = format!("{cookie:?} len={}", cookie.as_bytes().len());
        assert!(!shown.contains("super-secret"));
        assert!(shown.contains("redacted"));
    }

    #[test]
    fn response_bytes_appends_newline() {
        assert_eq!(response_bytes("pw"), b"pw\n");
        assert_eq!(response_bytes("pw\n"), b"pw\n");
    }

    #[test]
    fn status_text_variants() {
        assert_eq!(
            status_text(true, Some("8")),
            "polkit: registered (session 8)\n"
        );
        assert_eq!(status_text(false, None), "polkit: not-registered\n");
        assert_eq!(status_text(false, Some("")), "polkit: not-registered\n");
        // Session id is only shown when registered (never on failures).
        assert_eq!(status_text(false, Some("8")), "polkit: not-registered\n");
        assert_eq!(status_text(true, None), "polkit: registered\n");
    }

    // -- session against a fake helper (shell script) -------------------

    /// Fake helper script: reads `<cookie>` on stdin, emits one prompt,
    /// checks the password, prints SUCCESS/FAILURE. The socket path is
    /// pointed at a missing file so the session always takes the spawn
    /// path against this script.
    /// Write an executable fake-helper script, fsyncing before return
    /// so a racing `exec` never hits `ETXTBSY` under parallel tests.
    fn write_helper(dir: &tempfile::TempDir, name: &str, script: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt as _;
        let path = dir.path().join(name);
        {
            let mut file = std::fs::File::create(&path).unwrap();
            use std::io::Write as _;
            file.write_all(script.as_bytes()).unwrap();
            file.sync_all().unwrap();
        }
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn fake_helper(dir: &tempfile::TempDir, password: &str) -> PathBuf {
        let script = format!(
            "#!/bin/sh\nread cookie\nprintf 'PAM_PROMPT_ECHO_OFF Password: \\n'\nread pw\nif [ \"$pw\" = \"{password}\" ]; then printf 'SUCCESS\\n'; else printf 'FAILURE\\n'; fi\n"
        );
        write_helper(dir, "fake-helper", &script)
    }

    fn session_paths(helper: &Path) -> SessionPaths {
        SessionPaths {
            socket_path: helper.with_extension("no-socket"),
            helper_bin: helper.to_path_buf(),
        }
    }

    #[test]
    fn session_success_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let helper = fake_helper(&dir, "s3cret");
        let paths = session_paths(&helper);
        let cookie = Cookie::new(b"cookie-1".to_vec());
        let mut session = HelperSession::start("someone", &cookie, &paths).unwrap();
        assert!(session.has_child());
        assert_eq!(
            session.next_event().unwrap(),
            Some(HelperEvent::Request {
                prompt: "Password: ".to_string(),
                echo: false,
            })
        );
        session.respond("s3cret").unwrap();
        assert_eq!(
            session.next_event().unwrap(),
            Some(HelperEvent::Completed(true))
        );
    }

    #[test]
    fn session_wrong_password_fails() {
        let dir = tempfile::tempdir().unwrap();
        let helper = fake_helper(&dir, "s3cret");
        let paths = session_paths(&helper);
        let cookie = Cookie::new(b"cookie-1".to_vec());
        let mut session = HelperSession::start("someone", &cookie, &paths).unwrap();
        assert!(matches!(
            session.next_event().unwrap(),
            Some(HelperEvent::Request { .. })
        ));
        session.respond("wrong").unwrap();
        assert_eq!(
            session.next_event().unwrap(),
            Some(HelperEvent::Completed(false))
        );
    }

    #[test]
    fn session_unknown_line_completes_false() {
        let dir = tempfile::tempdir().unwrap();
        let helper = write_helper(
            &dir,
            "unknown-helper",
            "#!/bin/sh\nread cookie\nprintf 'WEIRD-LINE\\n'\nsleep 30\n",
        );
        let paths = session_paths(&helper);
        let mut session =
            HelperSession::start("someone", &Cookie::new(b"c".to_vec()), &paths).unwrap();
        // Unknown line → fail-secure Completed(false).
        assert_eq!(
            session.next_event().unwrap(),
            Some(HelperEvent::Completed(false))
        );
    }

    #[test]
    fn session_eof_is_fail_secure() {
        let dir = tempfile::tempdir().unwrap();
        let helper = write_helper(&dir, "eof-helper", "#!/bin/sh\nread cookie\nexit 0\n");
        let paths = session_paths(&helper);
        let mut session =
            HelperSession::start("someone", &Cookie::new(b"c".to_vec()), &paths).unwrap();
        // Immediate EOF → None → caller completes false.
        assert_eq!(session.next_event().unwrap(), None);
    }

    #[test]
    fn session_spawn_failure_errors() {
        let paths = SessionPaths {
            socket_path: PathBuf::from("/nonexistent-polkit-socket-xyz"),
            helper_bin: PathBuf::from("/nonexistent-helper-bin-xyz"),
        };
        let err = HelperSession::start("someone", &Cookie::new(b"c".to_vec()), &paths);
        assert!(err.is_err());
    }

    #[test]
    fn session_cancel_reaps_helper_without_orphans() {
        let dir = tempfile::tempdir().unwrap();
        let helper = write_helper(&dir, "sleepy-helper", "#!/bin/sh\nread cookie\nsleep 30\n");
        let paths = session_paths(&helper);
        let mut session =
            HelperSession::start("someone", &Cookie::new(b"c".to_vec()), &paths).unwrap();
        assert!(session.has_child());
        let pid = session.child.as_ref().map(|child| child.id());
        session.cancel();
        assert!(!session.has_child());
        // No orphan: the pid must be gone (kill-0 fails or zombie reaped).
        if let Some(pid) = pid {
            // SAFETY: `kill(pid, 0)` is a read-only liveness probe; the
            // child was our own helper, already reaped above.
            let alive = unsafe { libc::kill(pid as libc::pid_t, 0) == 0 };
            if alive {
                // A living pid here would mean the helper survived
                // SIGTERM+SIGKILL — fail loudly (INV-POLKIT-7).
                panic!("helper pid {pid} survived cancel");
            }
        }
    }

    #[test]
    fn session_socket_path_prefers_socket() {
        // Bind a fake agent-helper socket: server reads `<user>\n<cookie>\n`
        // then emits SUCCESS.
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("agent-helper.socket");
        let listener = std::os::unix::net::UnixListener::bind(&sock).unwrap();
        let server = std::thread::spawn(move || {
            use std::io::Read as _;
            let (mut conn, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            // Read two lines: user + cookie.
            let mut lines = Vec::new();
            while lines.len() < 2 {
                conn.read_exact(&mut byte).unwrap();
                if byte[0] == b'\n' {
                    lines.push(std::mem::take(&mut buf));
                } else {
                    buf.push(byte[0]);
                }
            }
            assert_eq!(lines[0], b"alice");
            assert_eq!(lines[1], b"cookie-9");
            conn.write_all(b"SUCCESS\n").unwrap();
        });
        let paths = SessionPaths {
            socket_path: sock,
            helper_bin: PathBuf::from("/nonexistent-helper-bin-xyz"),
        };
        let mut session =
            HelperSession::start("alice", &Cookie::new(b"cookie-9".to_vec()), &paths).unwrap();
        assert!(!session.has_child(), "socket path spawns no child");
        assert_eq!(
            session.next_event().unwrap(),
            Some(HelperEvent::Completed(true))
        );
        server.join().unwrap();
    }

    // -- pidfile guard ----------------------------------------------------

    #[test]
    fn pidfile_stale_is_overwritten_live_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sway-manager-polkit.pid");
        // Stale pid (unlikely to exist) → claim succeeds.
        std::fs::write(&path, "4294967290\n").unwrap();
        assert!(claim_pidfile(&path).is_ok());
        // Our own pid is live → second claim refuses.
        let err = claim_pidfile(&path).unwrap_err();
        assert!(err.contains("already running"), "{err}");
        release_pidfile(&path);
        assert!(!path.exists());
        // After release, claim succeeds again.
        assert!(claim_pidfile(&path).is_ok());
        release_pidfile(&path);
    }

    #[test]
    fn agent_handle_cancel_and_status() {
        let handle = AgentHandle::default();
        assert!(!handle.is_registered());
        assert_eq!(handle.status_line(), "polkit: not-registered\n");
        handle.cancel_current(); // no challenge: no-op, never panics.
    }

    #[test]
    fn gui_line_parser_shapes() {
        use agent::test_hooks as hooks;
        assert!(hooks::parse_gui("{\"password\":\"s3cret\"}").contains("Password"));
        assert!(hooks::parse_gui("{\"cancel\":true}").contains("Cancel"));
        assert!(hooks::parse_gui("garbage").contains("Invalid"));
        assert!(hooks::parse_gui("{\"password\":\"a\nb\"}").contains("Invalid"));
        assert_eq!(
            hooks::event("Password: ", false),
            "{\"prompt\":\"Password: \",\"echo\":false}\n"
        );
        assert_eq!(hooks::escape("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }
}
