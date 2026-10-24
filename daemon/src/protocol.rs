//! Line-delimited JSON protocol spoken over the Unix socket.
//!
//! Mirrors `daemon_client.send_command` and
//! `SwayManagerDaemon._process_request`: one UTF-8 JSON object per line,
//! `{"args": [...]}` upstream and
//! `{"stdout": ..., "stderr": ..., "exit_code": ...}` downstream.

use serde::{Deserialize, Serialize};

/// Client request payload: the original CLI argv.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub args: Vec<String>,
}

/// Server response payload with captured stdio and exit code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
    #[serde(default)]
    pub exit_code: i32,
}

impl Response {
    pub fn ok(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            exit_code: 0,
        }
    }

    pub fn daemon_error(stderr: impl Into<String>) -> Self {
        Self {
            stdout: String::new(),
            stderr: stderr.into(),
            exit_code: 1,
        }
    }

    /// Immediate empty ack for interactive/GUI commands (<1ms release).
    pub fn ack() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
        }
    }
}

/// Serialize a message with the mandatory trailing `\n`.
pub fn encode_line<T: Serialize>(message: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(message).map(|mut line| {
        line.push('\n');
        line
    })
}

/// Parse one received line (trailing newline optional).
pub fn decode_line<'a, T: Deserialize<'a>>(line: &'a str) -> Result<T, serde_json::Error> {
    serde_json::from_str(line.trim())
}
