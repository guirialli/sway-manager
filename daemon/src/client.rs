//! IPC client: forward argv to a running daemon.
//!
//! Mirrors `SwayManagerClient.send_command`
//! (`src/infrastructure/daemon/daemon_client.py`): return `false` when
//! the socket file is absent, else send `{"args": [...]}\n`, read one
//! JSON response line, replay stdio, and return `true`. Every transport
//! or protocol failure collapses to `false` (caller falls back).

use std::io::Write;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::protocol::{Request, Response, decode_line, encode_line};
use crate::socket::socket_path;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// Test seam: where to print replayed daemon output.
pub trait Output {
    fn print_stdout(&mut self, text: &str);
    fn print_stderr(&mut self, text: &str);
}

/// Real stdio; appends `\n` only when the payload lacks a trailing one.
pub struct Stdio;

impl Output for Stdio {
    fn print_stdout(&mut self, text: &str) {
        if text.ends_with('\n') {
            print!("{text}");
        } else {
            println!("{text}");
        }
        let _ = std::io::stdout().flush();
    }

    fn print_stderr(&mut self, text: &str) {
        if text.ends_with('\n') {
            eprint!("{text}");
        } else {
            eprintln!("{text}");
        }
        let _ = std::io::stderr().flush();
    }
}

async fn send_to(
    socket: &std::path::Path,
    args: &[String],
    output: &mut dyn Output,
    timeout: Duration,
) -> bool {
    let payload = match encode_line(&Request {
        args: args.to_vec(),
    }) {
        Ok(payload) => payload,
        Err(_) => return false,
    };

    let connect = tokio::net::UnixStream::connect(socket);
    let mut stream = match tokio::time::timeout(timeout, connect).await {
        Ok(Ok(stream)) => stream,
        _ => return false,
    };

    if tokio::time::timeout(timeout, stream.write_all(payload.as_bytes()))
        .await
        .is_err()
    {
        return false;
    }

    let mut reader = tokio::io::BufReader::new(&mut stream);
    let mut line = String::new();
    let read = tokio::time::timeout(timeout, reader.read_line(&mut line)).await;
    match read {
        Ok(Ok(_)) if !line.is_empty() => {}
        _ => return false,
    }

    let response: Response = match decode_line(&line) {
        Ok(response) => response,
        Err(_) => return false,
    };

    if !response.stdout.is_empty() {
        output.print_stdout(&response.stdout);
    }
    if !response.stderr.is_empty() {
        output.print_stderr(&response.stderr);
    }
    true
}

/// Forward `args` to the daemon at `socket`; see module docs.
pub async fn send_command_to(
    socket: &std::path::Path,
    args: &[String],
    output: &mut dyn Output,
    timeout: Duration,
) -> bool {
    if !socket.exists() {
        return false;
    }
    send_to(socket, args, output, timeout).await
}

/// Forward `args` to the default runtime socket.
pub async fn send_command(args: &[String]) -> bool {
    send_command_to(&socket_path(), args, &mut Stdio, DEFAULT_TIMEOUT).await
}
