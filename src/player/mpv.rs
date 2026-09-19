//! MPV video player integration with JSON IPC over Unix socket or Windows named pipe.

use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

use crate::config::PlayerConfig;
use crate::providers::models::StreamUrl;
use crate::SlothError;

/// Deserialized mpv JSON IPC response.
#[derive(Debug, Clone, serde::Deserialize)]
struct MpvIpcResponse {
    #[serde(default)]
    data: Option<f64>,
    #[serde(default)]
    error: Option<String>,
}

/// Controller for an active mpv process communicating via IPC.
pub struct MpvPlayer {
    child: tokio::process::Child,
    ipc_path: PathBuf,
}

impl MpvPlayer {
    /// Spawns an mpv instance configured with an IPC server and starts playback.
    pub async fn spawn(
        stream: &StreamUrl,
        resume_pos: Option<f64>,
        config: &PlayerConfig,
    ) -> Result<Self, SlothError> {
        let ipc_path = Self::ipc_socket_path();

        let executable = config
            .custom_command
            .as_deref()
            .filter(|c| !c.is_empty())
            .unwrap_or("mpv");

        let mut cmd = tokio::process::Command::new(executable);

        cmd.arg(&stream.url)
            .arg("--no-terminal")
            .arg(format!("--input-ipc-server={}", ipc_path.display()));

        // Custom headers via --http-header-fields
        for (k, v) in &stream.headers {
            cmd.arg(format!("--http-header-fields={}: {}", k, v));
        }

        // Resume playback position
        if let Some(pos) = resume_pos {
            if pos > 0.0 {
                cmd.arg(format!("--start={}", pos));
            }
        }

        // External subtitle track
        if let Some(sub) = &stream.subtitle_url {
            cmd.arg(format!("--sub-file={}", sub));
        }

        // HLS demuxer whitelist
        if stream.is_hls {
            cmd.arg("--demuxer-lavf-o=protocol_whitelist=file,http,https,tcp,tls,crypto");
        }

        // Additional user-specified arguments from config
        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd.spawn().map_err(SlothError::Io)?;

        // Wait 300ms for IPC socket to be created
        tokio::time::sleep(Duration::from_millis(300)).await;

        Ok(Self { child, ipc_path })
    }

    /// Reads current playback position in seconds via IPC.
    pub async fn get_position(&self) -> Option<f64> {
        tokio::time::timeout(Duration::from_secs(1), self.read_ipc_position())
            .await
            .ok()
            .flatten()
    }

    /// Waits for mpv process to exit, returning the last captured position.
    pub async fn wait_for_exit(&mut self) -> Option<f64> {
        let mut last_pos = self.get_position().await;

        // Poll position periodically while mpv is running
        loop {
            tokio::select! {
                _ = self.child.wait() => {
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    if let Some(pos) = self.get_position().await {
                        last_pos = Some(pos);
                    }
                }
            }
        }

        // Final position poll if mpv hasn't fully cleaned up socket
        if let Some(pos) = self.get_position().await {
            last_pos = Some(pos);
        }

        #[cfg(unix)]
        {
            let _ = tokio::fs::remove_file(&self.ipc_path).await;
        }

        last_pos
    }

    /// Generates platform-specific IPC socket or named pipe path.
    fn ipc_socket_path() -> PathBuf {
        let id = Uuid::new_v4();
        #[cfg(unix)]
        {
            std::env::temp_dir().join(format!("sloth-mpv-{id}.sock"))
        }
        #[cfg(windows)]
        {
            PathBuf::from(format!(r"\\.\pipe\sloth-mpv-{id}"))
        }
        #[cfg(not(any(unix, windows)))]
        {
            std::env::temp_dir().join(format!("sloth-mpv-{id}"))
        }
    }

    #[cfg(windows)]
    async fn read_ipc_position(&self) -> Option<f64> {
        use tokio::net::windows::named_pipe::ClientOptions;
        let client = ClientOptions::new().open(&self.ipc_path).ok()?;
        let (reader, mut writer) = tokio::io::split(client);
        writer
            .write_all(b"{\"command\": [\"get_property\", \"time-pos\"]}\n")
            .await
            .ok()?;
        writer.flush().await.ok()?;

        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        while reader.read_line(&mut line).await.ok()? > 0 {
            if let Ok(resp) = serde_json::from_str::<MpvIpcResponse>(&line) {
                if resp.error.as_deref() == Some("success") {
                    return resp.data;
                }
            }
            line.clear();
        }
        None
    }

    #[cfg(unix)]
    async fn read_ipc_position(&self) -> Option<f64> {
        let stream = tokio::net::UnixStream::connect(&self.ipc_path).await.ok()?;
        let (reader, mut writer) = stream.into_split();
        writer
            .write_all(b"{\"command\": [\"get_property\", \"time-pos\"]}\n")
            .await
            .ok()?;
        writer.flush().await.ok()?;

        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        while reader.read_line(&mut line).await.ok()? > 0 {
            if let Ok(resp) = serde_json::from_str::<MpvIpcResponse>(&line) {
                if resp.error.as_deref() == Some("success") {
                    return resp.data;
                }
            }
            line.clear();
        }
        None
    }

    #[cfg(not(any(windows, unix)))]
    async fn read_ipc_position(&self) -> Option<f64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_socket_path_format() {
        let path = MpvPlayer::ipc_socket_path();
        let path_str = path.to_string_lossy();
        #[cfg(windows)]
        {
            assert!(path_str.starts_with(r"\\.\pipe\sloth-mpv-"));
        }
        #[cfg(unix)]
        {
            assert!(path_str.contains("sloth-mpv-"));
            assert!(path_str.ends_with(".sock"));
        }
    }

    #[test]
    fn test_mpv_ipc_response_parsing() {
        let success_json = r#"{"data": 3847.2, "request_id": 0, "error": "success"}"#;
        let resp: MpvIpcResponse = serde_json::from_str(success_json).expect("valid json");
        assert_eq!(resp.error.as_deref(), Some("success"));
        assert_eq!(resp.data, Some(3847.2));

        let err_json = r#"{"error": "property unavailable"}"#;
        let err_resp: MpvIpcResponse = serde_json::from_str(err_json).expect("valid json");
        assert_eq!(err_resp.error.as_deref(), Some("property unavailable"));
        assert_eq!(err_resp.data, None);
    }
}
