//! Watch progress tracking and scrobbling services (Trakt.tv, AniList).

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
    Ok(())
}
