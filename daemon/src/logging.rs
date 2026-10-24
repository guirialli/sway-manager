//! Non-blocking file logger mirroring `AsyncLogger`.
//!
//! Format per line (async_logger.py:72-74):
//! `[YYYY-MM-DD HH:MM:SS.mmm] [LEVEL] msg` appended to
//! `~/.config/sway-manager/logs/log-YYYY-MM-DD.txt`. A background thread
//! drains an mpsc queue so logging never blocks the server loop.

use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;
use tokio::sync::mpsc;

const QUEUE_CAPACITY: usize = 1024;

fn logs_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config/sway-manager/logs")
}

/// Local `YYYY-MM-DD HH:MM:SS.mmm` using libc (no extra deps).
fn timestamp() -> String {
    // SAFETY: `localtime_r` with valid pointers; output buffer sized for
    // `%Y-%m-%d %H:%M:%S` + NUL.
    unsafe {
        let now = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut tm);
        let mut buf = [0 as libc::c_char; 20];
        libc::strftime(
            buf.as_mut_ptr(),
            buf.len(),
            c"%Y-%m-%d %H:%M:%S".as_ptr(),
            &tm,
        );
        let date = std::ffi::CStr::from_ptr(buf.as_ptr())
            .to_string_lossy()
            .into_owned();
        let ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.subsec_millis())
            .unwrap_or(0);
        format!("{date}.{ms:03}")
    }
}

fn today_log_path() -> PathBuf {
    let date = timestamp().split(' ').next().unwrap_or("").to_string();
    logs_dir().join(format!("log-{date}.txt"))
}

fn spawn_worker(rx: mpsc::Receiver<String>) {
    // Detached by dropping the JoinHandle: the thread outlives the
    // handle and dies with the process, like the Python daemon thread.
    let _ = std::thread::Builder::new()
        .name("sway-logger".to_string())
        .spawn(move || worker_loop(rx));
}

fn worker_loop(mut rx: mpsc::Receiver<String>) {
    while let Some(line) = rx.blocking_recv() {
        let path = today_log_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write as _;
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
        if rx.is_closed() && rx.is_empty() {
            break;
        }
    }
}

/// Cloneable async logger handle (mirrors the singleton `get_logger()`).
#[derive(Clone, Debug)]
pub struct Logger {
    tx: mpsc::Sender<String>,
}

impl Logger {
    fn enqueue(&self, level: &str, msg: &str) {
        let line = format!("[{}] [{level}] {msg}", timestamp());
        let _ = self.tx.try_send(line);
    }

    pub fn info(&self, msg: &str) {
        self.enqueue("INFO", msg);
    }

    pub fn cmd(&self, msg: &str) {
        self.enqueue("CMD", msg);
    }

    pub fn error(&self, msg: &str) {
        self.enqueue("ERROR", msg);
    }

    /// Today's log path (mirrors `get_today_log_path`, for `daemon log`).
    pub fn today_log_path() -> PathBuf {
        today_log_path()
    }
}

static GLOBAL: LazyLock<Logger> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel(QUEUE_CAPACITY);
    spawn_worker(rx);
    Logger { tx }
});

/// Process-wide logger (mirrors `AsyncLogger.get_instance`).
pub fn logger() -> Logger {
    GLOBAL.clone()
}

/// Test/CLI handle with its own queue (avoids global thread in tests).
pub fn test_logger() -> (Logger, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel(QUEUE_CAPACITY);
    (Logger { tx }, rx)
}

pub fn shared(logger: &Logger) -> Arc<Logger> {
    Arc::new(logger.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logger_enqueues_formatted_lines() {
        let (logger, mut rx) = test_logger();
        logger.info("hello");
        logger.cmd("run it");
        logger.error("boom");
        for level in ["INFO", "CMD", "ERROR"] {
            let line = rx.recv().await.expect("queued log line");
            assert!(line.contains(&format!("[{level}]")), "line: {line}");
        }
        // Today path mirrors `log-YYYY-MM-DD.txt`.
        let path = Logger::today_log_path().to_string_lossy().into_owned();
        assert!(path.ends_with(".txt"));
        assert!(path.contains("log-"));
        let _ = shared(&logger);
    }

    #[test]
    fn timestamp_has_millisecond_precision() {
        let stamp = timestamp();
        // `YYYY-MM-DD HH:MM:SS.mmm`
        assert_eq!(stamp.len(), 23, "stamp: {stamp}");
        assert_eq!(&stamp[19..20], ".");
    }
}
