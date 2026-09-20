//! Application configuration management and directory resolution.

use crate::providers::addons::models::InstalledAddon;
use crate::providers::models::ProviderKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application name identifier used for directories, logs, and user-agent.
pub const APP_NAME: &str = "sloth-tui";

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Video player settings.
    pub player: PlayerConfig,
    /// Provider-specific settings.
    pub providers: ProvidersConfig,
    /// Whether auto update check is enabled.
    pub auto_update: bool,
    /// Epoch timestamp of last update check.
    pub last_update_check: u64,
    /// Currently active interface mode (e.g. "streaming", "addons", "tv").
    pub active_mode: String,
    /// Currently selected streaming provider.
    pub active_provider: ProviderKind,
    /// Selected UI theme name.
    pub active_theme: String,
    /// Whether MovieBox provider is enabled.
    pub moviebox_enabled: bool,
    /// Whether 4KHDHub provider is enabled.
    pub fourkhdhub_enabled: bool,
    /// Whether BDIX CircleFTP provider is enabled.
    pub bdix_circleftp_enabled: bool,
    /// Whether BDIX DhakaFlix provider is enabled.
    pub bdix_dhakaflix_enabled: bool,
    /// Whether BDIX connectivity has been probed.
    pub bdix_probed: bool,
    /// Whether streaming mode is enabled.
    pub streaming_enabled: bool,
    /// Whether Live TV mode is enabled.
    pub tv_enabled: bool,
    /// Whether Stremio addons mode is enabled.
    pub addons_enabled: bool,
    /// Optional default external player override.
    pub default_player: Option<String>,
    /// Optional custom download directory.
    pub download_dir: Option<String>,
    /// Whether Discord Rich Presence is enabled.
    pub discord_rpc_enabled: bool,
    /// Background notification settings.
    pub notifications: NotificationsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            player: PlayerConfig::default(),
            providers: ProvidersConfig::default(),
            auto_update: true,
            last_update_check: 0,
            active_mode: "streaming".to_string(),
            active_provider: ProviderKind::MovieBox,
            active_theme: String::new(),
            moviebox_enabled: true,
            fourkhdhub_enabled: true,
            bdix_circleftp_enabled: false,
            bdix_dhakaflix_enabled: false,
            bdix_probed: false,
            streaming_enabled: true,
            tv_enabled: true,
            addons_enabled: false,
            default_player: None,
            download_dir: None,
            discord_rpc_enabled: true,
            notifications: NotificationsConfig::default(),
        }
    }
}

/// Notifications configuration settings.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct NotificationsConfig {
    /// Whether background OS notifications are enabled.
    pub enabled: bool,
    /// Alert lead time in minutes for F1 sessions.
    pub f1_lead_time_minutes: u32,
    /// Whether anime episode alerts are enabled.
    pub anime_alerts: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            f1_lead_time_minutes: 15,
            anime_alerts: true,
        }
    }
}

/// Player configuration settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerConfig {
    /// Preferred video player executable name.
    pub preferred: String,
    /// Optional custom command override to launch the player.
    pub custom_command: Option<String>,
    /// Additional command-line arguments to pass to the player.
    pub extra_args: Vec<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            preferred: "mpv".to_string(),
            custom_command: None,
            extra_args: Vec::new(),
        }
    }
}

/// Provider-related configuration options.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProvidersConfig {
    /// List of disabled provider identifiers.
    #[serde(default)]
    pub disabled: Vec<String>,
}

/// Returns configuration directory for the application.
pub fn config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SLOTH_CONFIG_DIR").or_else(|_| std::env::var("MOVIEBOX_CONFIG_DIR")) {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = dirs::config_dir() {
        return Some(dir.join(APP_NAME));
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg);
        if !p.as_os_str().is_empty() {
            return Some(p.join(APP_NAME));
        }
    }
    if let Ok(prefix) = std::env::var("PREFIX") {
        let p = PathBuf::from(prefix).join("etc").join(APP_NAME);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(dir) = dirs::home_dir().map(|h| h.join(".config").join(APP_NAME)) {
        return Some(dir);
    }
    let fallback = std::env::temp_dir().join(APP_NAME).join("config");
    log::warn!(
        "unable to locate user config directory, falling back to {}",
        fallback.display()
    );
    Some(fallback)
}

/// Returns persistent data directory for history, favorites, and databases.
pub fn data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SLOTH_DATA_DIR").or_else(|_| std::env::var("MOVIEBOX_DATA_DIR")) {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = dirs::data_dir() {
        return Some(dir.join(APP_NAME));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg);
        if !p.as_os_str().is_empty() {
            return Some(p.join(APP_NAME));
        }
    }
    if let Ok(prefix) = std::env::var("PREFIX") {
        let p = PathBuf::from(prefix).join("var").join("lib").join(APP_NAME);
        if p.exists() {
            return Some(p);
        }
    }
    if let Some(dir) = dirs::home_dir().map(|h| h.join(".local").join("share").join(APP_NAME)) {
        return Some(dir);
    }
    let fallback = std::env::temp_dir().join(APP_NAME).join("data");
    log::warn!(
        "unable to locate user data directory, falling back to {}",
        fallback.display()
    );
    Some(fallback)
}

/// Returns cache directory for temporary thumbnails, manifests, and catalogs.
pub fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SLOTH_CACHE_DIR").or_else(|_| std::env::var("MOVIEBOX_CACHE_DIR")) {
        return PathBuf::from(dir);
    }
    if let Some(dir) = dirs::cache_dir() {
        return dir.join(APP_NAME);
    }
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        let p = PathBuf::from(xdg);
        if !p.as_os_str().is_empty() {
            return p.join(APP_NAME);
        }
    }
    if let Ok(prefix) = std::env::var("PREFIX") {
        let p = PathBuf::from(prefix)
            .join("var")
            .join("cache")
            .join(APP_NAME);
        if p.exists() {
            return p;
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".cache").join(APP_NAME))
        .unwrap_or_else(|| std::env::temp_dir().join(APP_NAME))
}

/// Returns directory for log files.
pub fn logs_dir() -> PathBuf {
    data_dir()
        .map(|dir| dir.join("logs"))
        .unwrap_or_else(|| std::env::temp_dir().join(APP_NAME).join("logs"))
}

/// Returns directory for custom player script integrations.
pub fn scripts_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("scripts"))
}

/// Returns directory for player playback state persistence.
pub fn playback_state_dir() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("playback"))
}

/// Returns path to main config.json file.
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.json"))
}

/// Returns path to addons_config.json file.
pub fn addons_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("addons_config.json"))
}

/// Returns path to tv_config.json file.
pub fn tv_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("tv_config.json"))
}

/// Returns path to legacy history.json file.
pub fn history_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("history.json"))
}

/// Returns path to legacy favorites.json file.
pub fn favorites_path() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join("favorites.json"))
}

/// Returns the path to the SQLite database file (`sloth.db`).
pub fn db_path() -> PathBuf {
    if let Ok(path) = std::env::var("SLOTH_DB_PATH") {
        return PathBuf::from(path);
    }
    if let Some(dir) = data_dir() {
        return dir.join("sloth.db");
    }
    PathBuf::from("sloth.db")
}

/// Checks PATH for available video players in preference order: mpv, vlc, iina, celluloid.
pub fn detect_available_players() -> Vec<String> {
    let candidates = ["mpv", "vlc", "iina", "celluloid"];
    candidates
        .iter()
        .filter(|&&name| is_player_in_path(name))
        .map(|&s| s.to_string())
        .collect()
}

/// Checks whether an executable name is available on the system PATH.
pub fn is_player_in_path(name: &str) -> bool {
    let mut paths_to_search: Vec<PathBuf> = Vec::new();
    if let Some(path_var) = std::env::var_os("PATH") {
        paths_to_search.extend(std::env::split_paths(&path_var));
    }

    #[cfg(target_os = "windows")]
    {
        let candidate_exts = ["", ".exe", ".com", ".cmd", ".bat"];
        for dir in paths_to_search {
            for ext in &candidate_exts {
                let candidate = dir.join(format!("{name}{ext}"));
                if candidate.is_file() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    {
        for dir in paths_to_search {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return true;
            }
        }
        false
    }
}

/// Resolves the effective player: preferred if installed, otherwise first detected, or fallback to preferred.
/// Returns (player_name, optional_warning_message).
pub fn resolve_player(config: &PlayerConfig) -> (String, Option<String>) {
    let detected = detect_available_players();
    if detected.iter().any(|p| p.eq_ignore_ascii_case(&config.preferred)) {
        return (config.preferred.clone(), None);
    }
    if let Some(first) = detected.first() {
        let warning = format!(
            "Preferred player '{}' not found in PATH. Using detected player '{}'.",
            config.preferred, first
        );
        return (first.clone(), Some(warning));
    }
    let warning = "No video player (mpv, vlc, iina, celluloid) found in PATH. Playback may fail.".to_string();
    (config.preferred.clone(), Some(warning))
}

/// Loads configuration from disk with corrupt file recovery.
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let mut val: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let old_bdix = val
                .get("bdix_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if let Some(obj) = val.as_object_mut() {
                obj.remove("bdix_enabled");
            }
            if let Ok(mut config) = serde_json::from_value::<Config>(val) {
                if old_bdix {
                    config.bdix_circleftp_enabled = true;
                    config.bdix_dhakaflix_enabled = true;
                }
                return config;
            }
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let corrupt_path = path.with_extension(format!("corrupt.{stamp}"));
        log::error!(
            "failed to parse config from {}, rotating to {}",
            crate::logging::sanitize_path(&path),
            crate::logging::sanitize_path(&corrupt_path)
        );
        let _ = std::fs::rename(&path, corrupt_path);
    }
    Config::default()
}

/// Saves configuration to disk atomically.
pub fn save(config: &Config) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        if let Err(error) = crate::cache::atomic_write_file(&path, json.as_bytes()) {
            log::warn!("failed to write config: {error}");
        }
    }
}

/// Loads installed Stremio addons list, ensuring default Cinemeta is always present.
pub fn load_addons() -> Vec<InstalledAddon> {
    let mut list = if let Some(path) = addons_path() {
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Vec<InstalledAddon>>(&content) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        let stamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let corrupt_path = path.with_extension(format!("corrupt.{stamp}"));
                        log::error!(
                            "failed to parse addons config from {} ({e}), rotating to {}",
                            crate::logging::sanitize_path(&path),
                            crate::logging::sanitize_path(&corrupt_path)
                        );
                        let _ = std::fs::rename(&path, corrupt_path);
                        Vec::new()
                    }
                },
                Err(e) => {
                    log::warn!(
                        "failed to read addons config from {}: {e}",
                        crate::logging::sanitize_path(&path)
                    );
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if !list.iter().any(|a| a.is_core()) {
        list.insert(0, InstalledAddon::cinemeta_default());
        save_addons(&list);
    } else {
        for a in &mut list {
            if a.is_core() {
                a.enabled = true;
            }
        }
    }
    list
}

/// Saves installed Stremio addons list to disk atomically.
pub fn save_addons(addons: &[InstalledAddon]) {
    let Some(path) = addons_path() else {
        return;
    };
    if let Some(app_dir) = path.parent() {
        if std::fs::create_dir_all(app_dir).is_err() {
            return;
        }
    }
    let Ok(json) = serde_json::to_string_pretty(addons) else {
        return;
    };
    if let Err(error) = crate::cache::atomic_write_file(&path, json.as_bytes()) {
        log::warn!("failed to write addons config: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_and_serde() {
        let config = Config::default();
        assert!(config.auto_update);
        assert_eq!(config.active_provider, ProviderKind::MovieBox);

        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: Config = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.active_mode, config.active_mode);
    }

    #[test]
    fn test_player_detection_and_resolution() {
        let config = PlayerConfig {
            preferred: "nonexistent_custom_player_xyz".to_string(),
            custom_command: None,
            extra_args: Vec::new(),
        };
        let (resolved, warning) = resolve_player(&config);
        assert!(!resolved.is_empty());
        assert!(warning.is_some());
    }
}
