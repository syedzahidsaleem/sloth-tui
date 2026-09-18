//! Watch history persistence and queries.

use sqlx::SqlitePool;

use crate::SlothError;
use crate::db::{HistoryRow, WatchEntry};

/// Inserts or updates a watch history record for a media episode or movie.
/// Sets completed to true if playback progress is >= 85% of total duration.
pub async fn upsert(pool: &SqlitePool, entry: &WatchEntry) -> Result<(), SlothError> {
    let is_completed = entry.completed
        || entry
            .duration
            .is_some_and(|d| d > 0.0 && (entry.resume_position / d) >= 0.85);
    let completed_val: i64 = if is_completed { 1 } else { 0 };
    let season = entry.season as i64;
    let episode = entry.episode as i64;

    // Ensure foreign key target exists in media cache if not yet cached
    sqlx::query!(
        r#"
        INSERT OR IGNORE INTO media (id, provider_id, title, kind)
        VALUES (?1, coalesce(?2, 'unknown'), ?1, 'movie')
        "#,
        entry.media_id,
        entry.source_provider
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO watch_history (
            media_id, season, episode, episode_title, resume_position,
            duration, completed, source_provider, quality, watched_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, unixepoch())
        ON CONFLICT(media_id, season, episode) DO UPDATE SET
            episode_title = excluded.episode_title,
            resume_position = excluded.resume_position,
            duration = excluded.duration,
            completed = excluded.completed,
            source_provider = excluded.source_provider,
            quality = excluded.quality,
            watched_at = unixepoch()
        "#,
        entry.media_id,
        season,
        episode,
        entry.episode_title,
        entry.resume_position,
        entry.duration,
        completed_val,
        entry.source_provider,
        entry.quality
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Retrieves the recorded resume timestamp in seconds for a specific episode, if any.
pub async fn get_resume_position(
    pool: &SqlitePool,
    media_id: &str,
    season: u32,
    episode: u32,
) -> Option<f64> {
    let season = season as i64;
    let episode = episode as i64;
    let row = sqlx::query!(
        r#"
        SELECT resume_position
        FROM watch_history
        WHERE media_id = ?1 AND season = ?2 AND episode = ?3
        "#,
        media_id,
        season,
        episode
    )
    .fetch_optional(pool)
    .await
    .ok()??;

    row.resume_position
}

/// Retrieves recently watched media items joined with metadata, ordered latest first.
pub async fn recent(pool: &SqlitePool, limit: u32) -> Result<Vec<HistoryRow>, SlothError> {
    let limit = limit as i64;
    let rows = sqlx::query_as!(
        HistoryRow,
        r#"
        SELECT
            wh.id as "id!",
            wh.media_id as "media_id!",
            wh.season as "season!",
            wh.episode as "episode!",
            wh.episode_title,
            wh.watched_at as "watched_at!",
            wh.resume_position as "resume_position!",
            wh.duration,
            wh.completed as "completed!",
            wh.source_provider,
            wh.quality,
            m.title as "title!",
            m.kind as "kind!",
            m.poster_url
        FROM watch_history wh
        JOIN media m ON m.id = wh.media_id
        ORDER BY wh.watched_at DESC
        LIMIT ?1
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Retrieves in-progress media items with playback between 5% and 85% of duration.
pub async fn continue_watching(pool: &SqlitePool) -> Result<Vec<HistoryRow>, SlothError> {
    let rows = sqlx::query_as!(
        HistoryRow,
        r#"
        SELECT
            wh.id as "id!",
            wh.media_id as "media_id!",
            wh.season as "season!",
            wh.episode as "episode!",
            wh.episode_title,
            wh.watched_at as "watched_at!",
            wh.resume_position as "resume_position!",
            wh.duration,
            wh.completed as "completed!",
            wh.source_provider,
            wh.quality,
            m.title as "title!",
            m.kind as "kind!",
            m.poster_url
        FROM watch_history wh
        JOIN media m ON m.id = wh.media_id
        WHERE wh.completed = 0
          AND wh.resume_position > 0.0
          AND wh.duration IS NOT NULL
          AND wh.duration > 0.0
          AND (wh.resume_position / wh.duration) BETWEEN 0.05 AND 0.85
        ORDER BY wh.watched_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Marks a specific media episode or movie as fully completed.
pub async fn mark_completed(
    pool: &SqlitePool,
    media_id: &str,
    season: u32,
    episode: u32,
) -> Result<(), SlothError> {
    let season = season as i64;
    let episode = episode as i64;
    sqlx::query!(
        r#"
        UPDATE watch_history
        SET completed = 1, watched_at = unixepoch()
        WHERE media_id = ?1 AND season = ?2 AND episode = ?3
        "#,
        media_id,
        season,
        episode
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_history_crud() {
        let temp = NamedTempFile::new().expect("create temp file");
        let pool = open(temp.path()).await.expect("open db");

        let entry = WatchEntry {
            media_id: "test:movie1".to_string(),
            season: 0,
            episode: 0,
            episode_title: Some("Pilot".to_string()),
            resume_position: 50.0,
            duration: Some(100.0),
            completed: false,
            source_provider: Some("test_provider".to_string()),
            quality: Some("1080p".to_string()),
        };

        upsert(&pool, &entry).await.expect("upsert watch entry");

        let pos = get_resume_position(&pool, "test:movie1", 0, 0).await;
        assert_eq!(pos, Some(50.0));

        let recents = recent(&pool, 10).await.expect("query recent");
        assert_eq!(recents.len(), 1);
        assert_eq!(recents[0].media_id, "test:movie1");

        let in_progress = continue_watching(&pool)
            .await
            .expect("query continue watching");
        assert_eq!(in_progress.len(), 1);

        mark_completed(&pool, "test:movie1", 0, 0)
            .await
            .expect("mark completed");
        let in_progress_after = continue_watching(&pool)
            .await
            .expect("query continue watching after completion");
        assert_eq!(in_progress_after.len(), 0);
    }
}
