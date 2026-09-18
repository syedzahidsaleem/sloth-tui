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

/// Artifact detection and platform environment utilities.
pub mod artifact {
    pub const TERMUX_PREFIX_USR: &str = "/data/data/com.termux/files/usr";

    /// Detects whether the current process is running inside an Android / Termux environment.
    pub fn is_termux_environment() -> bool {
        cfg!(target_os = "android")
            || std::env::var("TERMUX_VERSION").is_ok()
            || std::env::var("PREFIX").is_ok_and(|p| p.contains("com.termux"))
            || std::path::Path::new(TERMUX_PREFIX_USR).exists()
    }
}

/// Update apply and installation environment detection.
pub mod apply {
    use std::path::Path;

    /// Installation environment categories.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InstallationEnvironment {
        DirectReplace,
        Homebrew,
        Termux,
        Flatpak,
        Snap,
        ReadOnly,
        WindowsHelper,
    }

    impl InstallationEnvironment {
        /// Checks if the installation is managed by a system package manager that should notify the user.
        pub fn has_managed_notice(&self) -> bool {
            !matches!(self, Self::DirectReplace | Self::WindowsHelper)
        }
    }

    /// Detects whether the executable path belongs to a Homebrew prefix.
    pub fn is_homebrew_managed(exe_path: &Path) -> bool {
        let s = exe_path.to_string_lossy();
        s.contains("/Cellar/")
            || s.contains("/opt/homebrew/")
            || s.contains("/usr/local/Cellar/")
            || s.contains("/home/linuxbrew/.linuxbrew/Cellar/")
    }

    /// Detects the target environment for the given executable path.
    pub fn detect_environment(exe_path: &Path) -> InstallationEnvironment {
        if std::env::var_os("FLATPAK_ID").is_some() || Path::new("/.flatpak-info").exists() {
            return InstallationEnvironment::Flatpak;
        }

        if std::env::var_os("SNAP").is_some() {
            return InstallationEnvironment::Snap;
        }

        if super::artifact::is_termux_environment() {
            return InstallationEnvironment::Termux;
        }

        if is_homebrew_managed(exe_path) {
            return InstallationEnvironment::Homebrew;
        }

        if cfg!(windows) {
            return InstallationEnvironment::WindowsHelper;
        }

        InstallationEnvironment::DirectReplace
    }
}

/// Release check utilities.
pub mod check {
    pub const OWNER: &str = "syedzahidsaleem";
    pub const REPOSITORY: &str = "sloth-tui";

    /// Formats the GitHub releases tag URL.
    pub fn release_tag_url(tag: &str) -> String {
        let tag_clean = tag.trim_start_matches('v');
        format!("https://github.com/{OWNER}/{REPOSITORY}/releases/tag/v{tag_clean}")
    }
}

/// Checks GitHub releases for available application updates.
pub async fn check() -> Result<Option<Release>, String> {
    Ok(None)
}

/// Checks if a newer release is available compared to the current version.
pub async fn check_release(_current: &str) -> Result<Option<Release>, String> {
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
