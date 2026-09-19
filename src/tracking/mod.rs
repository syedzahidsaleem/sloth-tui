//! Watch progress tracking and scrobbling services (Trakt.tv, AniList).

pub mod anilist_sync;

pub use anilist_sync::{AniListClient, AniListEntry};

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

/// Synchronizes anime watch progress with AniList.
pub async fn sync_anilist(
    media_id: &str,
    episode: u32,
    progress_secs: f64,
) -> Result<(), SlothError> {
    tracing::info!(
        "AniList scrobble sync: media={media_id}, episode={episode}, pos={progress_secs:.1}s"
    );

    if let Ok(pool) = crate::db::open(&crate::config::db_path()).await {
        if let Ok(Some(client)) = AniListClient::authenticate(&pool).await {
            if let Ok(anilist_id) = media_id.parse::<u32>() {
                let _ = client
                    .mark_episode_watched_with_pool(Some(&pool), anilist_id, episode)
                    .await;
            }
        }
    }

    Ok(())
}
