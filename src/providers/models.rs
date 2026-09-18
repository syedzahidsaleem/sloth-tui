//! Provider models and error definitions.

/// Errors that can occur during provider operations.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Underlying HTTP request failure.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The remote provider responded with HTTP 429 Too Many Requests.
    #[error("provider is rate limiting us (HTTP 429)")]
    RateLimited,

    /// The requested content was not found on this provider.
    #[error("content not found on this provider")]
    NotFound,

    /// Failed to parse HTML, JSON, or stream manifests from provider response.
    #[error("provider response parsing failed: {0}")]
    Parsing(String),

    /// The provider service is unreachable or temporarily down.
    #[error("provider is temporarily unavailable")]
    Unavailable,

    /// Authentication credentials or account login are required.
    #[error("auth required — configure in Settings")]
    AuthRequired,
}

impl ProviderError {
    /// Returns a human-friendly error message suitable for displaying in the TUI.
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Network(_) => "Network error. Check your connection.",
            Self::RateLimited => "Too many requests. Trying next source...",
            Self::NotFound => "Not found on this provider.",
            Self::Parsing(_) => "Provider response changed. Trying fallback...",
            Self::Unavailable => "Provider is down. Trying next source...",
            Self::AuthRequired => "Login required. Check Settings > Accounts.",
        }
    }

    /// Checks whether the error is considered transient and retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::RateLimited | Self::Unavailable
        )
    }
}
