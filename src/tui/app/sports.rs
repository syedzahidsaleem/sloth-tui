//! Sports application event handlers for loading matches, fetching streams, and initiating playback.

use tokio::sync::mpsc;

use crate::providers::models::{PlaybackSource, ProviderKind};
use crate::providers::sports::StreamedProvider;
use crate::tui::action::Action;
use crate::tui::state::{AppState, SportsColumnFocus};

/// Loads matches for a given sport category and dispatches `Action::MatchListReceived`.
pub fn handle_sport_selected(
    state: &mut AppState,
    tx: &mpsc::UnboundedSender<Action>,
    sport_name: String,
) {
    state.is_loading = true;
    state.sports_tab.focus = SportsColumnFocus::Matches;

    if let Some(pos) = state
        .sports_tab
        .sports
        .iter()
        .position(|s| s.eq_ignore_ascii_case(&sport_name))
    {
        state.sports_tab.selected_sport_idx = pos;
    }

    state.sports_tab.selected_match_idx = 0;
    state.sports_tab.streams.clear();
    state.sports_tab.selected_stream_idx = 0;

    let tx = tx.clone();
    let sport_query = sport_name.to_lowercase();
    tokio::spawn(async move {
        let provider = StreamedProvider::new();
        match provider.fetch_matches_by_sport(&sport_query).await {
            Ok(matches) => {
                let state_matches = matches.into_iter().map(Into::into).collect();
                let _ = tx.send(Action::MatchListReceived(state_matches));
            }
            Err(e) => {
                tracing::warn!("Failed to fetch matches for sport '{sport_query}': {e}");
                let _ = tx.send(Action::MatchListReceived(Vec::new()));
            }
        }
    });
}

/// Loads stream sources for a specific match id and dispatches `Action::StreamListReceived`.
pub fn handle_match_selected(
    state: &mut AppState,
    tx: &mpsc::UnboundedSender<Action>,
    match_id: String,
) {
    state.is_loading = true;
    state.sports_tab.focus = SportsColumnFocus::Streams;

    if let Some(pos) = state
        .sports_tab
        .matches
        .iter()
        .position(|m| m.id == match_id)
    {
        state.sports_tab.selected_match_idx = pos;
    }

    state.sports_tab.streams.clear();
    state.sports_tab.selected_stream_idx = 0;

    let category = state
        .sports_tab
        .selected_match()
        .map(|m| m.category.clone())
        .unwrap_or_else(|| "all".to_string());

    let tx = tx.clone();
    let cat = category.to_lowercase();
    let m_id = match_id.clone();
    tokio::spawn(async move {
        let provider = StreamedProvider::new();
        match provider.fetch_streams(&cat, &m_id).await {
            Ok(streams) => {
                let state_streams = streams.into_iter().map(Into::into).collect();
                let _ = tx.send(Action::StreamListReceived(state_streams));
            }
            Err(e) => {
                tracing::warn!("Failed to fetch streams for match '{m_id}': {e}");
                let _ = tx.send(Action::StreamListReceived(Vec::new()));
            }
        }
    });
}

/// Resolves the selected stream and initiates playback.
pub fn handle_stream_play(state: &mut AppState, tx: &mpsc::UnboundedSender<Action>) {
    let stream = match state.sports_tab.selected_stream() {
        Some(s) => s,
        None => return,
    };

    let title = state
        .sports_tab
        .selected_match()
        .map(|m| m.title.as_str())
        .unwrap_or("Live Sports");

    let stream_url = stream
        .hd_url
        .as_ref()
        .or(stream.sd_url.as_ref())
        .or(stream.embed_url.as_ref());

    if let Some(url) = stream_url {
        let language = stream.language.as_deref().unwrap_or("Stream");
        let source = PlaybackSource {
            provider: ProviderKind::FourKHdHub,
            url: url.clone(),
            headers: vec![
                ("Referer".into(), "https://streamed.pk".into()),
                ("User-Agent".into(), "Sloth-TUI/0.1.0".into()),
            ],
            subtitle: None,
            source_label: format!("{title} ({language})"),
        };
        let _ = tx.send(Action::DispatchPlayback(source));
        let _ = tx.send(Action::PlaybackStarted);
    }
}

/// Re-fetches current sport matches from streamed.pk (e.g. on 'r' or periodic 5-minute tick).
pub fn refresh_live_data(state: &mut AppState, tx: &mpsc::UnboundedSender<Action>) {
    state.sports_tab.last_refresh = Some(std::time::Instant::now());
    state.is_loading = true;

    let sport = state
        .sports_tab
        .selected_sport()
        .unwrap_or("football")
        .to_string();

    let tx = tx.clone();
    let sport_query = sport.to_lowercase();
    tokio::spawn(async move {
        let provider = StreamedProvider::new();
        match provider.fetch_matches_by_sport(&sport_query).await {
            Ok(matches) => {
                let state_matches = matches.into_iter().map(Into::into).collect();
                let _ = tx.send(Action::MatchListReceived(state_matches));
            }
            Err(e) => {
                tracing::warn!("Failed to refresh live matches for '{sport_query}': {e}");
                let _ = tx.send(Action::MatchListReceived(Vec::new()));
            }
        }
    });
}
