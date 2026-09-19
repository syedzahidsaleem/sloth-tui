//! Discord Rich Presence integration via Discord IPC.

use crate::providers::models::{EpisodeRef, Media};

/// Default placeholder Discord Application ID for Sloth.
pub const DEFAULT_DISCORD_APP_ID: u64 = 1234567890;

#[cfg(feature = "discord")]
pub struct DiscordRpc {
    client: discord_presence::Client,
    enabled: bool,
}

#[cfg(feature = "discord")]
impl DiscordRpc {
    /// Creates a new Discord RPC client with Sloth's Discord app ID.
    ///
    /// Respects the `SLOTH_DISCORD_APP_ID` environment variable if configured.
    pub fn new(enabled: bool) -> Self {
        let app_id = std::env::var("SLOTH_DISCORD_APP_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_DISCORD_APP_ID);

        let mut client = discord_presence::Client::new(app_id);
        if enabled {
            let _ = client.start();
        }

        Self { client, enabled }
    }

    /// Toggles Discord Rich Presence on or off at runtime.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            let _ = self.client.start();
        } else {
            self.clear();
        }
    }

    /// Returns whether Discord Rich Presence is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Updates the Discord status to reflect active media playback.
    pub fn set_watching(&mut self, media: &Media, episode: Option<&EpisodeRef>) {
        if !self.enabled {
            return;
        }

        let details = media.title.clone();
        let state_line = if let Some(ep) = episode {
            if let Some(ref title) = ep.title {
                if !title.is_empty() {
                    format!("S{}E{} — {}", ep.season, ep.episode, title)
                } else {
                    format!("S{}E{}", ep.season, ep.episode)
                }
            } else {
                format!("S{}E{}", ep.season, ep.episode)
            }
        } else {
            "Movie".to_string()
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let _ = self.client.set_activity(|act| {
            act.details(details)
                .state(state_line)
                .assets(|assets| assets.large_image("sloth_logo").large_text("Sloth TUI"))
                .timestamps(|ts| ts.start(now))
        });
    }

    /// Updates the Discord status to browsing mode.
    pub fn set_browsing(&mut self) {
        if !self.enabled {
            return;
        }

        let _ = self.client.set_activity(|act| {
            act.details("Looking for something to watch")
                .state("Browsing")
                .assets(|assets| assets.large_image("sloth_logo").large_text("Sloth TUI"))
        });
    }

    /// Clears any active presence from Discord.
    pub fn clear(&mut self) {
        let _ = self.client.clear_activity();
    }
}

#[cfg(feature = "discord")]
impl Drop for DiscordRpc {
    fn drop(&mut self) {
        if self.enabled {
            self.clear();
        }
    }
}

/// Fallback Discord RPC stub when the `discord` feature is disabled.
#[cfg(not(feature = "discord"))]
pub struct DiscordRpc;

#[cfg(not(feature = "discord"))]
impl DiscordRpc {
    /// Creates a no-op Discord RPC stub.
    pub fn new(_enabled: bool) -> Self {
        Self
    }

    /// No-op toggle.
    pub fn set_enabled(&mut self, _enabled: bool) {}

    /// Returns false when disabled at compile time.
    pub fn is_enabled(&self) -> bool {
        false
    }

    /// No-op watching presence.
    pub fn set_watching(&mut self, _: &Media, _: Option<&EpisodeRef>) {}

    /// No-op browsing presence.
    pub fn set_browsing(&mut self) {}

    /// No-op clear presence.
    pub fn clear(&mut self) {}
}
