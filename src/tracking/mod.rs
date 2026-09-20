//! Watch progress tracking and scrobbling services (Trakt.tv, AniList).

pub mod anilist_sync;
pub mod discord_rpc;
pub mod trakt;

pub use anilist_sync::{AniListClient, AniListEntry};
pub use discord_rpc::DiscordRpc;
pub use trakt::{
    DeviceCodeResponse, DeviceTokenPollStatus, DeviceTokenResponse, ScrobbleResponse, TraktClient,
    TraktEntry, TraktEpisode, TraktHistoryItem, TraktIds, TraktMovie, TraktShow,
    DEFAULT_TRAKT_CLIENT_ID, TRAKT_API_URL, TRAKT_API_VERSION,
};

use crate::SlothError;
use sqlx::Row;

/// Synchronizes watch progress with Trakt.tv.
pub async fn sync_trakt(
    media_id: &str,
    season: u32,
    episode: u32,
    progress_secs: f64,
) -> Result<(), SlothError> {
    tracing::info!(
        "Trakt scrobble sync: media={media_id}, season={season}, episode={episode}, pos={progress_secs:.1}s"
    );

    if let Ok(pool) = crate::db::open(&crate::config::db_path()).await {
        if let Ok(Some(client)) =
            TraktClient::authenticate(&pool, DEFAULT_TRAKT_CLIENT_ID, None).await
        {
            let is_movie = season == 0 && episode == 0;

            let history_info = sqlx::query(
                "SELECT wh.duration, wh.completed, m.title, m.kind \
                 FROM watch_history wh \
                 LEFT JOIN media m ON m.id = wh.media_id \
                 WHERE wh.media_id = ?1 AND wh.season = ?2 AND wh.episode = ?3",
            )
            .bind(media_id)
            .bind(season as i64)
            .bind(episode as i64)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

            let duration = history_info.as_ref().and_then(|r| r.get::<Option<f64>, _>("duration"));
            let completed_flag = history_info
                .as_ref()
                .map(|r| r.get::<i64, _>("completed") == 1)
                .unwrap_or(false);
            let title = history_info
                .as_ref()
                .and_then(|r| r.get::<Option<String>, _>("title"))
                .unwrap_or_else(|| media_id.to_string());
            let kind_str = history_info
                .as_ref()
                .and_then(|r| r.get::<Option<String>, _>("kind"))
                .unwrap_or_else(|| if is_movie { "movie".to_string() } else { "series".to_string() });

            let is_movie_resolved = kind_str == "movie" || is_movie;
            let progress_pct = match duration {
                Some(d) if d > 0.0 => (progress_secs / d) * 100.0,
                _ => 0.0,
            };
            let is_completed = completed_flag || (duration.is_some() && progress_pct >= 85.0);

            let media = crate::providers::models::Media {
                id: media_id.to_string(),
                provider_id: "unknown",
                title,
                media_type: if is_movie_resolved {
                    crate::providers::models::MediaType::Movie
                } else {
                    crate::providers::models::MediaType::Series
                },
                year: None,
                poster_url: None,
                backdrop_url: None,
                rating: None,
                duration_secs: duration,
                overview: None,
                genres: vec![],
                episodes_count: None,
                seasons_count: None,
                external_ids: crate::providers::models::ExternalIds {
                    imdb: if media_id.starts_with("tt") {
                        Some(media_id.to_string())
                    } else {
                        None
                    },
                    tmdb: media_id
                        .strip_prefix("tmdb:")
                        .and_then(|s| s.parse::<u32>().ok()),
                    ..Default::default()
                },
                cast: vec![],
            };

            let s_opt = if is_movie_resolved { None } else { Some(season) };
            let e_opt = if is_movie_resolved { None } else { Some(episode) };

            if is_completed {
                let _ = client
                    .mark_watched(Some(&pool), &media, s_opt, e_opt)
                    .await;
            } else if progress_pct > 0.0 {
                let _ = client
                    .scrobble_pause(&media, s_opt, e_opt, progress_pct)
                    .await;
            }

            let _ = client.sync_dirty_entries(&pool).await;
        }
    }

    Ok(())
}

/// Synchronizes anime watch progress with AniList if completed (>85% watched).
pub async fn sync_anilist(
    media_id: &str,
    episode: u32,
    progress_secs: f64,
    duration_secs: Option<f64>,
) -> Result<(), SlothError> {
    let completed = duration_secs
        .map(|d| d > 0.0 && (progress_secs / d) >= 0.85)
        .unwrap_or(false);

    if !completed {
        tracing::debug!("AniList sync skipped: not completed (>85% required)");
        return Ok(());
    }

    tracing::info!(
        "AniList scrobble sync: media={media_id}, episode={episode}, pos={progress_secs:.1}s (completed)"
    );

    if let Ok(pool) = crate::db::open(&crate::config::db_path()).await {
        if let Ok(Some(client)) = AniListClient::authenticate(&pool).await {
            let anilist_id = media_id
                .parse::<u32>()
                .ok()
                .or_else(|| media_id.split('-').find_map(|s| s.parse::<u32>().ok()));

            if let Some(id) = anilist_id {
                if let Err(err) = client
                    .mark_episode_watched_with_pool(Some(&pool), id, episode)
                    .await
                {
                    tracing::warn!("AniList sync failed ({err}); queued as dirty");
                }
            }

            // Attempt to flush any previously dirty entries
            let _ = client.sync_dirty_entries(&pool).await;
        }
    }

    Ok(())
}
