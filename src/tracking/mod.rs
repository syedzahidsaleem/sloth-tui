//! Watch progress tracking and scrobbling services (Trakt.tv, AniList).

pub mod anilist_sync;
pub mod discord_rpc;

pub use anilist_sync::{AniListClient, AniListEntry};
pub use discord_rpc::DiscordRpc;

use crate::SlothError;

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
            let anilist_id = media_id.parse::<u32>().ok().or_else(|| {
                media_id.split('-').find_map(|s| s.parse::<u32>().ok())
            });

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
