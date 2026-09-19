//! Formula 1 application event handlers for loading calendars, resolving live streams, and updating countdown timers.

use tokio::sync::mpsc;

use crate::providers::f1::calendar::{fetch_calendar, next_session, time_until, F1Session};
use crate::providers::f1::streams::fetch_f1_streams;
use crate::providers::models::{PlaybackSource, ProviderKind};
use crate::tui::action::Action;
use crate::tui::state::AppState;

/// Fetches the F1 calendar (season 2026) asynchronously and dispatches `Action::F1CalendarReceived`.
pub fn handle_f1_calendar_load(
    state: &mut AppState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    state.f1_tab.loading = true;
    let tx = tx.clone();

    tokio::spawn(async move {
        match fetch_calendar(2026).await {
            Ok(calendar) => {
                let _ = tx.send(Action::F1CalendarReceived(calendar));
            }
            Err(e) => {
                tracing::warn!("Failed to fetch F1 calendar: {e}");
                let _ = tx.send(Action::F1CalendarReceived(Vec::new()));
            }
        }
    });
}

/// Resolves an F1 live stream for the session and initiates playback.
pub fn handle_f1_session_play(
    state: &mut AppState,
    tx: &mpsc::UnboundedSender<Action>,
    session: Option<&F1Session>,
) {
    let target_session = session
        .cloned()
        .or_else(|| state.f1_tab.selected_session().cloned())
        .or_else(|| next_session(&state.f1_tab.calendar).map(|(s, _)| s));

    let session_name = target_session
        .as_ref()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Formula 1 Grand Prix".to_string());

    let tx = tx.clone();
    tokio::spawn(async move {
        match fetch_f1_streams().await {
            Ok(streams) if !streams.is_empty() => {
                let first_stream = &streams[0];
                let source = PlaybackSource {
                    provider: ProviderKind::MovieBox,
                    url: first_stream.url.clone(),
                    headers: first_stream.headers.clone(),
                    subtitle: None,
                    source_label: format!("F1 Live — {session_name}"),
                };
                let _ = tx.send(Action::DispatchPlayback(source));
                let _ = tx.send(Action::PlaybackStarted);
            }
            Ok(_) => {
                tracing::warn!("No active Formula 1 streams found in sports playlist");
            }
            Err(e) => {
                tracing::warn!("Failed to fetch F1 streams: {e}");
            }
        }
    });
}

/// Calculates the duration until the next session and updates `F1TabState.countdown`.
pub fn update_countdown(state: &mut AppState) {
    if let Some((_, slot)) = next_session(&state.f1_tab.calendar) {
        state.f1_tab.countdown = Some(time_until(&slot));
    } else {
        state.f1_tab.countdown = None;
    }
}
