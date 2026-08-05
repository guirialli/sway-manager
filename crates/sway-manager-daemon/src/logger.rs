use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

pub fn init_logger() -> Result<(), String> {
    let log_dir = get_log_dir();
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| format!("Failed to create log directory at {:?}: {}", log_dir, e))?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Box guard or leak it so non_blocking logger stays active for process lifetime
    Box::leak(Box::new(_guard));

    fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    Ok(())
}

pub fn get_log_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    path.push("sway-manager");
    path.push("logs");
    path
}

#[allow(dead_code)]
pub fn get_today_log_path() -> PathBuf {
    let now = chrono_lite_date();
    get_log_dir().join(format!("log.{}", now))
}

#[allow(dead_code)]
fn chrono_lite_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let days = secs / 86400;
    // Estimate YYYY-MM-DD for logging filename prefix
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}
