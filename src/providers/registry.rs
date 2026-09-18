//! Provider registry implementation managing prioritized provider chains and health states.

use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::SlothError;
use crate::config::Config;
use crate::providers::Provider;
use crate::providers::models::{EpisodeRef, Media, MediaType, ProviderError, StreamUrl};

/// Registry managing active providers, prioritized fallback chains, and health state.
pub struct ProviderRegistry {
    /// Provider chain for movies and TV series.
    pub movie_chain: Vec<Arc<dyn Provider>>,
    /// Provider chain for anime content.
    pub anime_chain: Vec<Arc<dyn Provider>>,
    /// Provider chain for live sports events.
    pub sports_chain: Vec<Arc<dyn Provider>>,
    /// Provider chain for Formula 1 sessions.
    pub f1_chain: Vec<Arc<dyn Provider>>,
    /// Provider chain for live IPTV channels.
    pub tv_chain: Vec<Arc<dyn Provider>>,
    /// Thread-safe map tracking provider health status.
    pub health: Arc<RwLock<HashMap<&'static str, bool>>>,
}

impl ProviderRegistry {
    /// Creates a new `ProviderRegistry` instance with empty chains according to configuration.
    pub fn new(config: &Config) -> Self {
        let _ = config;
        Self {
            movie_chain: Vec::new(),
            anime_chain: Vec::new(),
            sports_chain: Vec::new(),
            f1_chain: Vec::new(),
            tv_chain: Vec::new(),
            health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the provider fallback chain corresponding to the given media type.
    pub fn chain_for(&self, kind: &MediaType) -> &[Arc<dyn Provider>] {
        match kind {
            MediaType::Movie | MediaType::Series => &self.movie_chain,
            MediaType::Anime => &self.anime_chain,
            MediaType::LiveSport => &self.sports_chain,
            MediaType::F1 => &self.f1_chain,
            MediaType::IptvChannel => &self.tv_chain,
        }
    }

    /// Searches all healthy providers in the corresponding chain in parallel,
    /// merges results, and deduplicates them by lowercase title.
    pub async fn search(&self, query: &str, kind: MediaType) -> Vec<Media> {
        let chain = self.chain_for(&kind);
        let futures: Vec<_> = chain
            .iter()
            .filter(|p| self.is_healthy(p.id()))
            .map(|p| p.search(query, kind))
            .collect();

        // Run all providers in parallel, merge and deduplicate
        let results = futures::future::join_all(futures).await;
        let mut merged: Vec<Media> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect();

        // Deduplicate by title (simple fuzzy match)
        merged.dedup_by(|a, b| a.title.to_lowercase() == b.title.to_lowercase());
        merged
    }

    /// Resolves stream URLs using the chain for the media type.
    /// Tries providers sequentially, skipping unhealthy ones and falling back on errors.
    pub async fn resolve(
        &self,
        media: &Media,
        episode: Option<&EpisodeRef>,
    ) -> Result<Vec<StreamUrl>, SlothError> {
        let chain = self.chain_for(&media.media_type);
        let mut last_err = None;

        for provider in chain {
            if !self.is_healthy(provider.id()) {
                continue;
            }

            match provider.resolve(media, episode).await {
                Ok(urls) if !urls.is_empty() => return Ok(urls),
                Ok(_) => continue,
                Err(ProviderError::RateLimited(_)) => continue,
                Err(e) => {
                    tracing::warn!(
                        provider = provider.id(),
                        error = ?e,
                        "Provider failed, trying next"
                    );
                    last_err = Some(e);
                }
            }
        }

        Err(SlothError::NoProvidersAvailable {
            context: last_err.map(|e| e.to_string()),
        })
    }

    /// Checks if a provider is marked healthy, defaulting to true if not present.
    pub fn is_healthy(&self, id: &'static str) -> bool {
        self.health.read().get(id).copied().unwrap_or(true)
    }

    /// Returns all registered providers across all chains, deduplicated by identifier.
    pub fn all_providers(&self) -> Vec<Arc<dyn Provider>> {
        let mut seen = HashSet::new();
        let mut all = Vec::new();
        for chain in [
            &self.movie_chain,
            &self.anime_chain,
            &self.sports_chain,
            &self.f1_chain,
            &self.tv_chain,
        ] {
            for p in chain {
                if seen.insert(p.id()) {
                    all.push(Arc::clone(p));
                }
            }
        }
        all
    }

    /// Runs health checks on all registered providers in parallel and updates the health map.
    pub async fn run_health_checks(&self) {
        let checks: Vec<_> = self
            .all_providers()
            .iter()
            .map(|p| {
                let p = Arc::clone(p);
                async move { (p.id(), p.health().await) }
            })
            .collect();

        let results = futures::future::join_all(checks).await;
        let mut map = self.health.write();
        for (id, ok) in results {
            map.insert(id, ok);
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}
