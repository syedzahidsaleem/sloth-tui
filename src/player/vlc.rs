//! VLC media player integration.

use crate::config::PlayerConfig;
use crate::providers::models::StreamUrl;
use crate::SlothError;

/// Controller for an active VLC player process.
pub struct VlcPlayer {
    child: tokio::process::Child,
}

impl VlcPlayer {
    /// Spawns a VLC media player process for playback.
    pub async fn spawn(
        stream: &StreamUrl,
        resume_pos: Option<f64>,
        config: &PlayerConfig,
    ) -> Result<Self, SlothError> {
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

        let mut cmd = tokio::process::Command::new(executable);
        cmd.arg(&stream.url);
        cmd.arg("--play-and-exit");

        if let Some(pos) = resume_pos {
            if pos > 0.0 {
                cmd.arg(format!("--start-time={}", pos as u64));
            }
        }

        for (k, v) in &stream.headers {
            if k.eq_ignore_ascii_case("referer") {
                cmd.arg(format!("--http-referrer={v}"));
            } else if k.eq_ignore_ascii_case("user-agent") {
                cmd.arg(format!("--http-user-agent={v}"));
            }
        }

        if let Some(sub) = &stream.subtitle_url {
            cmd.arg(format!("--sub-file={sub}"));
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let child = cmd
            .spawn()
            .map_err(|e| SlothError::Internal(format!("Failed to spawn VLC: {e}")))?;

        Ok(Self { child })
    }

    /// Waits for VLC process to exit. VLC does not support IPC resume tracking.
    pub async fn wait_for_exit(&mut self) -> Option<f64> {
        let _ = self.child.wait().await;
        None
    }
}
