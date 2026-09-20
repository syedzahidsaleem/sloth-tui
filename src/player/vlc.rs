//! VLC media player integration with HTTP interface for playback control and state tracking.

use parking_lot::Mutex;
use serde::Deserialize;
use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use crate::SlothError;
use crate::config::PlayerConfig;
use crate::providers::models::StreamUrl;

/// Deserialized VLC HTTP interface status response (`/requests/status.json`).
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct VlcStatus {
    /// Current playback position in seconds.
    #[serde(default)]
    pub time: f64,
    /// Total stream duration in seconds.
    #[serde(default)]
    pub length: f64,
    /// Playback state ("playing", "paused", "stopped").
    pub state: Option<String>,
}

impl VlcStatus {
    /// Parses status JSON string into a VlcStatus instance.
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    /// Extracts current position in seconds if playing or paused and time > 0.
    pub fn position(&self) -> Option<f64> {
        if self.is_stopped() || self.time <= 0.0 {
            None
        } else {
            Some(self.time)
        }
    }

    /// Extracts total stream duration in seconds.
    pub fn duration(&self) -> Option<f64> {
        if self.is_stopped() || self.length <= 0.0 {
            None
        } else {
            Some(self.length)
        }
    }

    /// Checks if playback is active (playing or paused).
    pub fn is_playing(&self) -> bool {
        match self.state.as_deref() {
            Some(s) => s.eq_ignore_ascii_case("playing") || s.eq_ignore_ascii_case("paused"),
            None => false,
        }
    }

    /// Checks if playback is stopped.
    pub fn is_stopped(&self) -> bool {
        match self.state.as_deref() {
            Some(s) => s.eq_ignore_ascii_case("stopped"),
            None => false,
        }
    }
}

/// Controller for an active VLC media player process communicating via HTTP interface.
pub struct VlcPlayer {
    child: tokio::process::Child,
    pub http_port: u16,
    pub http_password: String,
    pub http_client: reqwest::Client,
}

/// Finds an available TCP port in the 8080..=8180 range.
pub fn pick_free_port() -> u16 {
    for _ in 0..50 {
        let port = fastrand::u16(8080..=8180);
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    for port in 8080..=8180 {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    if let Ok(listener) = TcpListener::bind(("127.0.0.1", 0)) {
        if let Ok(addr) = listener.local_addr() {
            return addr.port();
        }
    }
    8080
}

/// Generates a random alphanumeric password of the given length.
pub fn generate_password(len: usize) -> String {
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        s.push(fastrand::alphanumeric());
    }
    s
}

/// Constructs the CLI arguments list for launching VLC with HTTP interface.
pub fn build_vlc_args(
    stream: &StreamUrl,
    resume_pos: Option<f64>,
    port: u16,
    password: &str,
    config: &PlayerConfig,
) -> Vec<String> {
    let mut args = vec![
        stream.url.clone(),
        "--play-and-exit".to_string(),
        "--no-one-instance".to_string(),
        "--extraintf=http".to_string(),
        "--http-host=127.0.0.1".to_string(),
        format!("--http-port={port}"),
        format!("--http-password={password}"),
        "--no-loop".to_string(),
        "--no-repeat".to_string(),
    ];

    if let Some(pos) = resume_pos {
        if pos > 10.0 {
            args.push(format!("--start-time={}", pos as u64));
        }
    }

    for (k, v) in &stream.headers {
        if k.eq_ignore_ascii_case("referer") {
            args.push(format!("--http-referrer={v}"));
        } else if k.eq_ignore_ascii_case("user-agent") {
            args.push(format!("--http-user-agent={v}"));
        }
    }

    if let Some(sub) = &stream.subtitle_url {
        args.push(format!("--sub-file={sub}"));
    }

    for arg in &config.extra_args {
        args.push(arg.clone());
    }

    args
}

impl VlcPlayer {
    /// Constructs a VlcPlayer instance wrapping an existing child process (useful for testing).
    pub fn new_with_child(
        child: tokio::process::Child,
        http_port: u16,
        http_password: String,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap_or_default();

        Self {
            child,
            http_port,
            http_password,
            http_client,
        }
    }

    /// Spawns a VLC media player process configured with HTTP remote control.
    pub async fn spawn(
        stream: &StreamUrl,
        resume_pos: Option<f64>,
        config: &PlayerConfig,
    ) -> Result<Self, SlothError> {
        let port = config.vlc_http_port.unwrap_or_else(pick_free_port);
        let password = generate_password(12);

        let fallback = if cfg!(target_os = "windows") {
            "vlc.exe"
        } else {
            "vlc"
        };
        let executable = config
            .custom_command
            .as_deref()
            .filter(|c| !c.is_empty())
            .unwrap_or(fallback);

        let args = build_vlc_args(stream, resume_pos, port, &password, config);

        let mut cmd = tokio::process::Command::new(executable);
        cmd.args(&args);

        let child = cmd.spawn().map_err(SlothError::Io)?;

        // Wait 800ms for VLC HTTP interface to initialize
        tokio::time::sleep(Duration::from_millis(800)).await;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap_or_default();

        Ok(Self {
            child,
            http_port: port,
            http_password: password,
            http_client,
        })
    }

    /// Fetches the playback status JSON from the VLC HTTP interface.
    pub async fn fetch_status(&self) -> Option<VlcStatus> {
        fetch_vlc_status(&self.http_client, self.http_port, &self.http_password).await
    }

    /// Reads current playback position in seconds via the VLC HTTP interface.
    pub async fn get_position(&self) -> Option<f64> {
        self.fetch_status().await.and_then(|s| s.position())
    }

    /// Reads total stream duration in seconds via the VLC HTTP interface.
    pub async fn get_duration(&self) -> Option<f64> {
        self.fetch_status().await.and_then(|s| s.duration())
    }

    /// Checks if playback is currently active (playing or paused).
    pub async fn is_playing(&self) -> bool {
        self.fetch_status()
            .await
            .map(|s| s.is_playing())
            .unwrap_or(false)
    }

    /// Polls playback position and duration once.
    pub async fn poll_position(&self) -> (Option<f64>, Option<f64>) {
        poll_position(self.http_port, &self.http_password).await
    }

    /// Waits for the VLC process to exit while polling playback position and duration.
    /// Returns `(Option<position>, Option<duration>)`.
    pub async fn wait_for_exit(&mut self) -> (Option<f64>, Option<f64>) {
        let last_state = Arc::new(Mutex::new((None::<f64>, None::<f64>)));

        let state_clone = Arc::clone(&last_state);
        let port = self.http_port;
        let password = self.http_password.clone();

        let poll_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                let (pos_opt, dur_opt) = poll_position(port, &password).await;
                let mut lock = state_clone.lock();
                if let Some(pos) = pos_opt {
                    lock.0 = Some(pos);
                }
                if let Some(dur) = dur_opt {
                    lock.1 = Some(dur);
                }
            }
        });

        let _ = self.child.wait().await;
        poll_handle.abort();

        // Final poll attempt after process exit
        let (pos_opt, dur_opt) = self.poll_position().await;
        let mut lock = last_state.lock();
        if let Some(pos) = pos_opt {
            lock.0 = Some(pos);
        }
        if let Some(dur) = dur_opt {
            lock.1 = Some(dur);
        }

        let lock = last_state.lock();
        *lock
    }
}

/// Polls the VLC HTTP endpoint once for (position, duration).
/// Returns `(None, None)` if the endpoint fails, is stopped, or times out.
pub async fn poll_position(port: u16, password: &str) -> (Option<f64>, Option<f64>) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(1500))
        .build()
    {
        Ok(c) => c,
        Err(_) => return (None, None),
    };

    if let Some(status) = fetch_vlc_status(&client, port, password).await {
        (status.position(), status.duration())
    } else {
        (None, None)
    }
}

/// Helper function to query the VLC HTTP endpoint.
pub async fn fetch_vlc_status(
    client: &reqwest::Client,
    port: u16,
    password: &str,
) -> Option<VlcStatus> {
    let url = format!("http://127.0.0.1:{port}/requests/status.json");
    let resp = client
        .get(&url)
        .basic_auth("", Some(password))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let status = resp.json::<VlcStatus>().await.ok()?;
    Some(status)
}
