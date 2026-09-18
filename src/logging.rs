//! Logging and tracing configuration.

use std::path::{Path, PathBuf};

/// Initializes the application logging and tracing subscribers.
pub fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    tracing::info!(
        app = crate::config::APP_NAME,
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        log_file = %display_path(),
        "session started"
    );
}

/// Returns the path to the primary application log file.
pub fn log_file_path() -> PathBuf {
    crate::config::logs_dir().join(format!("{}.log", crate::config::APP_NAME))
}

/// Returns a sanitized string representation of the log file path.
pub fn display_path() -> String {
    sanitize_path(log_file_path())
}

/// Replaces the home directory prefix with ~ to prevent leaking user paths.
pub fn sanitize_path(path: impl AsRef<Path>) -> String {
    let raw = path.as_ref().to_string_lossy().into_owned();
    dirs::home_dir()
        .and_then(|home| home.to_str().map(|home| home.to_string()))
        .map(|home| raw.replacen(&home, "~", 1))
        .unwrap_or(raw)
}

/// Redacts sensitive URL query parameters and user credentials, preserving scheme and host.
pub fn sanitize_url(raw: &str) -> String {
    match url::Url::parse(raw) {
        Ok(parsed) => {
            let host = parsed.host_str().unwrap_or("unknown");
            let scheme = parsed.scheme();
            if host.is_empty() {
                "[redacted]".to_string()
            } else {
                format!("{scheme}://{host}")
            }
        }
        Err(_) => {
            let lower = raw.to_ascii_lowercase();
            let host_start = lower.find("://").map(|index| index + 3);
            match host_start {
                Some(start) => {
                    let rest = &raw[start..];
                    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
                    let host = &rest[..host_end];
                    if host.is_empty() {
                        "[redacted]".to_string()
                    } else {
                        format!("https://{host}")
                    }
                }
                None => "[redacted]".to_string(),
            }
        }
    }
}
