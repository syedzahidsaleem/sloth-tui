//! Sloth TUI library root.
//!
//! Terminal interface for movies, anime, sports, F1, and live TV.

pub mod cache;
pub mod config;
pub mod daemon;
pub mod db;
pub mod download;
pub mod favorites;
pub mod history;
pub mod logging;
pub mod metadata;
pub mod models;
pub mod net;
pub mod player;
pub mod providers;
pub mod proxy;
pub mod service;
pub mod tracking;
pub mod tui;
pub mod updater;

/// Top-level application error type for Sloth.
#[derive(Debug, thiserror::Error)]
pub enum SlothError {
    /// No providers were able to fulfill the request.
    #[error("no providers available: {context:?}")]
    NoProvidersAvailable {
        /// Optional context or last error message describing why providers failed.
        context: Option<String>,
    },

    /// Database errors from SQLx.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Configuration parsing or loading errors.
    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),

    /// Standard I/O errors.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors originating from media providers.
    #[error("provider error: {0}")]
    Provider(#[from] providers::ProviderError),
}
