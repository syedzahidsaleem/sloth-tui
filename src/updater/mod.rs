//! Application updater and release checks.

use std::path::Path;

/// GitHub release information.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Release {
    /// Semantic version or release title.
    pub version: String,
    /// Git tag name.
    pub tag_name: String,
    /// Release notes description.
    pub notes: String,
}

/// Result outcome of an in-app self-update attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfUpdateOutcome {
    /// Updated successfully and ready to restart.
    Success,
    /// Manual package manager upgrade required.
    RequiresManualUpgrade(String),
}

/// Checks GitHub releases for available application updates.
pub async fn check() -> Result<Option<Release>, String> {
    Ok(None)
}

/// Executes self-update process for the given release.
pub async fn perform_self_update(
    _release: &Release,
    _progress_sender: Option<&tokio::sync::mpsc::UnboundedSender<String>>,
) -> Result<SelfUpdateOutcome, String> {
    Ok(SelfUpdateOutcome::RequiresManualUpgrade(
        "Self-update is not enabled. Please update via git pull or your package manager."
            .to_string(),
    ))
}

/// Spawns the newly updated binary and exits the current process.
pub fn restart_process(exe_path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(exe_path).exec();
        Err(format!("failed to exec restart process: {err}"))
    }
    #[cfg(not(unix))]
    {
        std::process::Command::new(exe_path)
            .spawn()
            .map_err(|e| format!("failed to spawn restart process: {e}"))?;
        std::process::exit(0);
    }
}
