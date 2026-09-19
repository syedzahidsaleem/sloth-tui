//! IINA player integration (macOS native with mpv IPC support).

use crate::config::PlayerConfig;
use crate::providers::models::StreamUrl;
use crate::SlothError;

#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::time::Duration;
#[cfg(target_os = "macos")]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(target_os = "macos")]
use uuid::Uuid;

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, serde::Deserialize)]
struct IinaIpcResponse {
    #[serde(default)]
    data: Option<f64>,
    #[serde(default)]
    error: Option<String>,
}

/// Controller for an active IINA player instance.
#[cfg(target_os = "macos")]
pub struct IinaPlayer {
    child: tokio::process::Child,
    ipc_path: PathBuf,
}

#[cfg(target_os = "macos")]
impl IinaPlayer {
    /// Spawns IINA with mpv IPC server arguments.
    pub async fn spawn(
        stream: &StreamUrl,
        resume_pos: Option<f64>,
        config: &PlayerConfig,
    ) -> Result<Self, SlothError> {
        let id = Uuid::new_v4();
        let ipc_path = std::env::temp_dir().join(format!("sloth-iina-{id}.sock"));

        let executable = config
            .custom_command
            .as_deref()
            .filter(|c| !c.is_empty())
            .unwrap_or("iina-cli");

        let mut cmd = tokio::process::Command::new(executable);
        cmd.arg(&stream.url)
            .arg("--keep-running")
            .arg(format!("--mpv-input-ipc-server={}", ipc_path.display()));

        if let Some(pos) = resume_pos {
            if pos > 0.0 {
                cmd.arg(format!("--mpv-start={pos}"));
            }
        }

        if let Some(sub) = &stream.subtitle_url {
            cmd.arg(format!("--mpv-sub-file={sub}"));
        }

        for (k, v) in &stream.headers {
            cmd.arg(format!("--mpv-http-header-fields={k}: {v}"));
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd
            .spawn()
            .map_err(|e| SlothError::Internal(format!("Failed to spawn IINA: {e}")))?;

        tokio::time::sleep(Duration::from_millis(300)).await;

        Ok(Self { child, ipc_path })
    }

    /// Reads position via mpv IPC over Unix socket.
    pub async fn get_position(&self) -> Option<f64> {
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
            if let Ok(resp) = serde_json::from_str::<IinaIpcResponse>(&line) {
                if resp.error.as_deref() == Some("success") {
                    return resp.data;
                }
            }
            line.clear();
        }
        None
    }

    /// Waits for IINA to exit and returns the captured position.
    pub async fn wait_for_exit(&mut self) -> Option<f64> {
        let pos = self.get_position().await;
        let _ = self.child.wait().await;
        let _ = tokio::fs::remove_file(&self.ipc_path).await;
        pos
    }
}

/// Stub implementation for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
pub struct IinaPlayer;

#[cfg(not(target_os = "macos"))]
impl IinaPlayer {
    pub async fn spawn(
        _stream: &StreamUrl,
        _resume_pos: Option<f64>,
        _config: &PlayerConfig,
    ) -> Result<Self, SlothError> {
        Err(SlothError::Internal(
            "IINA player is only supported on macOS".to_string(),
        ))
    }

    pub async fn wait_for_exit(&mut self) -> Option<f64> {
        None
    }
}
