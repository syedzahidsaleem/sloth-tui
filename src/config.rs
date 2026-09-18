//! Application configuration management.

use serde::{Deserialize, Serialize};

/// Main application configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    /// Player configuration settings.
    #[serde(default)]
    pub player: PlayerConfig,
    /// Provider configuration settings.
    #[serde(default)]
    pub providers: ProvidersConfig,
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
