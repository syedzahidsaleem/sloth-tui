//! Anime application handlers for searching, toggling sub/dub, and resolving episodes.

use tokio::sync::mpsc;

use crate::config::Config;
use crate::providers::ProviderRegistry;
use crate::providers::models::{EpisodeRef, MediaType, PlaybackSource, ProviderKind};
use crate::tui::action::Action;
use crate::tui::state::{AnimePanelFocus, AppState};

/// Handles keyword search for anime titles across healthy anime providers.
pub fn handle_anime_search(
    state: &mut AppState,
    tx: &mpsc::UnboundedSender<Action>,
    query: String,
) {
    if query.trim().is_empty() {
        return;
    }

    state.is_loading = true;
    state.has_search_settled = false;
    state.search_error = None;
    state.anime_tab.selected_idx = 0;
    state.anime_tab.selected_episode_idx = 0;
    state.anime_tab.focus = AnimePanelFocus::Results;

    let tx = tx.clone();
    tokio::spawn(async move {
        let config = Config::default();
        let registry = ProviderRegistry::new(&config);
        let results = registry.search(&query, MediaType::Anime).await;
        let _ = tx.send(Action::SearchResultsReceived(results));
    });
}

/// Toggles audio and subtitle preference between dubbed and subbed.
pub fn handle_sub_dub_toggle(state: &mut AppState) {
    state.anime_tab.is_dub = !state.anime_tab.is_dub;
    // If a media is selected and is currently playing, just update state without interrupting
}

/// Handles selecting an anime from the search results, moving focus to episodes.
pub fn handle_anime_select(state: &mut AppState, tx: &mpsc::UnboundedSender<Action>) {
    if let Some(media) = state.anime_tab.selected_anime().cloned() {
        state.anime_tab.focus = AnimePanelFocus::Episodes;
        state.anime_tab.selected_episode_idx = 0;

        // Populate initial episode list or fetch from provider
        let ep_count = media.episodes_count.unwrap_or(0);
        if ep_count > 0 {
            state.anime_tab.episodes = (1..=ep_count)
                .map(|i| EpisodeRef {
                    season: 1,
                    episode: i,
                    title: Some(format!("Episode {i}")),
                    duration_secs: None,
                })
                .collect();
        }

        let tx = tx.clone();
        tokio::spawn(async move {
            let config = Config::default();
            let registry = ProviderRegistry::new(&config);
            if let Some(provider) = registry.anime_chain.first() {
                if let Ok(eps) = provider.episodes(&media, 1).await {
                    let _ = tx.send(Action::AnimeEpisodesReceived(eps));
                }
            }
        });
    }
}

/// Resolves playback stream for the selected anime and episode.
pub fn handle_anime_episode_select(state: &mut AppState, tx: &mpsc::UnboundedSender<Action>) {
    let media = match state.anime_tab.selected_anime().cloned() {
        Some(m) => m,
        None => return,
    };

    let episode = state.anime_tab.selected_episode().cloned().or_else(|| {
        Some(EpisodeRef {
            season: 1,
            episode: 1,
            title: Some("Episode 1".to_string()),
            duration_secs: None,
        })
    });

    state.is_loading = true;
    let tx = tx.clone();

    tokio::spawn(async move {
        let config = Config::default();
        let registry = ProviderRegistry::new(&config);
        match registry.resolve(&media, episode.as_ref()).await {
            Ok(streams) if !streams.is_empty() => {
                let stream = &streams[0];
                let source = PlaybackSource {
                    provider: ProviderKind::MovieBox,
                    url: stream.url.clone(),
                    headers: stream.headers.clone(),
                    subtitle: stream.subtitle_url.clone(),
                    source_label: format!("{} Anime Stream", media.title),
                };
                let _ = tx.send(Action::DispatchPlayback(source));
                let _ = tx.send(Action::PlaybackStarted);
            }
            _ => {
                let _ = tx.send(Action::SetStatus(format!(
                    "Failed to resolve streams for {}",
                    media.title
                )));
            }
        }
    });
}
