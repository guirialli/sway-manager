//! Server side: single-instance probe, per-connection dispatch, loop.
//!
//! Mirrors `SwayManagerDaemon` (`daemon_server.py`): probe with a short
//! connect, answer interactive commands with an immediate empty ack,
//! run synchronous commands through [`handle_sync`](crate::handlers)
//! capturing output, answer malformed requests with `exit_code: 1`
//! instead of crashing, and run the Tokio accept loop.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::commands::{command_name, is_interactive_args};
use crate::dispatch::{CliRunner, DispatchOutcome, dispatch};
use crate::handlers::{self, SyncDeps};
use crate::logging::Logger;
use crate::protocol::{Request, Response, decode_line, encode_line};
use crate::runtime;

/// Probe timeout mirroring `is_daemon_running` (`settimeout(0.2)`).
pub const PROBE_TIMEOUT: Duration = Duration::from_millis(200);
/// Client I/O timeout mirroring `send_command` (`timeout=2.0`).
pub const IO_TIMEOUT: Duration = Duration::from_secs(2);

/// `true` when a daemon answers at `socket` (single-instance check).
pub async fn is_daemon_running_at(socket: &Path) -> bool {
    if !socket.exists() {
        return false;
    }
    matches!(
        tokio::time::timeout(PROBE_TIMEOUT, tokio::net::UnixStream::connect(socket)).await,
        Ok(Ok(_))
    )
}

/// [`CliRunner`] adapter over the real synchronous handlers.
pub struct SyncCliRunner<D> {
    deps: D,
}

impl<D> SyncCliRunner<D> {
    pub fn new(deps: D) -> Self {
        Self { deps }
    }
}

impl<D: SyncDeps> CliRunner for SyncCliRunner<D> {
    type Error = std::convert::Infallible;

    fn run_cli(&self, args: &[String]) -> Result<(String, String), Self::Error> {
        Ok(handlers::handle_sync(&self.deps, args))
    }
}

/// Handle one already-accepted connection to completion.
pub async fn handle_connection<R>(
    stream: &mut tokio::net::UnixStream,
    runner: &R,
    timeout: Duration,
    logger: &Logger,
) where
    R: CliRunner,
{
    let request_line = {
        let mut reader = tokio::io::BufReader::new(&mut *stream);
        let mut line = String::new();
        match tokio::time::timeout(timeout, reader.read_line(&mut line)).await {
            Ok(Ok(_)) => line,
            _ => return,
        }
    };

    if request_line.trim().is_empty() {
        return;
    }

    let request: Request = match decode_line(&request_line) {
        Ok(request) => request,
        Err(err) => {
            let response = Response::daemon_error(format!("Erro no daemon: {err}\n"));
            respond(stream, &response, timeout, logger).await;
            return;
        }
    };

    // Mirrors `self.logger.cmd(f"Executando comando via socket: ...")`.
    let cmd_str = if request.args.len() > 1 {
        request.args[1..].join(" ")
    } else {
        "help".to_string()
    };
    logger.cmd(&format!("Executando comando via socket: '{cmd_str}'"));

    let cmd = command_name(&request.args).unwrap_or_else(|| "help".to_string());
    if is_interactive_args(&cmd, &request.args) && !request.args.is_empty() {
        // Immediate empty ack (<1ms), then deferred execution — mirrors
        // `write + flush + disconnectFromServer` + `QTimer.singleShot`.
        // Sync subcommands (`wallpaper get`) bypass this: their stdout
        // ships back in the response below.
        respond(stream, &Response::ack(), timeout, logger).await;
        let outcome = dispatch(&request.args, runner, runtime::perform_gc);
        log_outcome(logger, &request.args, &outcome);
        return;
    }

    let outcome = dispatch(&request.args, runner, runtime::perform_gc);
    log_outcome(logger, &request.args, &outcome);
    let response = match outcome {
        DispatchOutcome::CliExecuted { stdout, stderr } => Response {
            stdout,
            stderr,
            exit_code: 0,
        },
        DispatchOutcome::NoCommand => Response::ack(),
    };
    respond(stream, &response, timeout, logger).await;
}

async fn respond(
    stream: &mut tokio::net::UnixStream,
    response: &Response,
    timeout: Duration,
    logger: &Logger,
) {
    match encode_line(response) {
        Ok(line) => {
            if tokio::time::timeout(timeout, stream.write_all(line.as_bytes()))
                .await
                .is_err()
            {
                logger.error("Falha ao responder cliente no socket (timeout de escrita)");
            }
        }
        Err(err) => logger.error(&format!("Falha ao codificar resposta IPC: {err}")),
    }
}

/// Mirrors the `logger.error("Falha na execução do comando ...")` in
/// `CliServerHandler` / `_safe_dispatch_command` (no-op on success).
fn log_outcome(logger: &Logger, args: &[String], outcome: &DispatchOutcome) {
    let DispatchOutcome::CliExecuted { stderr, .. } = outcome else {
        return;
    };
    if !stderr.is_empty() {
        let cmd = command_name(args).unwrap_or_default();
        logger.error(&format!("Falha na execução do comando '{cmd}': {stderr}"));
    }
}

/// Serve forever on a bound `listener` (mirrors `start` accept phase).
pub async fn serve_with<R>(
    socket_path: PathBuf,
    listener: tokio::net::UnixListener,
    runner: R,
    logger: Logger,
) -> std::io::Result<()>
where
    R: CliRunner + Send + Sync + 'static,
{
    let _ = socket_path;
    let runner = Arc::new(runner);

    let gc_logger = logger.clone();
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(runtime::GC_INTERVAL_SECS));
        loop {
            interval.tick().await;
            runtime::perform_gc();
            gc_logger.info("GC periódica executada");
        }
    });

    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let (runner, logger) = (Arc::clone(&runner), logger.clone());
        tokio::spawn(async move {
            handle_connection(&mut stream, &*runner, IO_TIMEOUT, &logger).await;
        });
    }
}

/// Serve forever on `socket_path` (mirrors `start` from bind to `exec`).
pub async fn serve<R>(socket_path: PathBuf, runner: R, logger: Logger) -> std::io::Result<()>
where
    R: CliRunner + Send + Sync + 'static,
{
    runtime::cleanup_stale_socket(&socket_path);
    let listener = tokio::net::UnixListener::bind(&socket_path)?;
    serve_with(socket_path, listener, runner, logger).await
}
