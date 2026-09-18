//! Database layer using SQLite and SQLx.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::path::Path;

use crate::SlothError;

pub mod favorites;
pub mod history;

/// Represents a watch history entry to be recorded or updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchEntry {
    /// Associated media item identifier.
    pub media_id: String,
    /// Season number (0 for movies or non-seasonal content).
    pub season: u32,
    /// Episode number (0 for movies or session id).
    pub episode: u32,
    /// Optional title or name of the episode.
    pub episode_title: Option<String>,
    /// Playback position in seconds where playback was stopped or resumed.
    pub resume_position: f64,
    /// Total duration of the media in seconds if known.
    pub duration: Option<f64>,
    /// Whether the entry is considered fully watched (>= 85% of duration).
    pub completed: bool,
    /// Name or identifier of the provider that served this stream.
    pub source_provider: Option<String>,
    /// Video quality of the stream watched (e.g. "1080p", "4K").
    pub quality: Option<String>,
}

/// A consolidated record joining watch history with cached media metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct HistoryRow {
    /// Watch history record primary key.
    pub id: i64,
    /// Identifier of the media item.
    pub media_id: String,
    /// Season number.
    pub season: i64,
    /// Episode number.
    pub episode: i64,
    /// Optional episode title.
    pub episode_title: Option<String>,
    /// Unix timestamp when last watched.
    pub watched_at: i64,
    /// Last playback position in seconds.
    pub resume_position: f64,
    /// Total media duration in seconds.
    pub duration: Option<f64>,
    /// Whether completed (1 if true, 0 if false).
    pub completed: i64,
    /// Provider that resolved the stream.
    pub source_provider: Option<String>,
    /// Quality tier string.
    pub quality: Option<String>,
    /// Media title from joined media table.
    pub title: String,
    /// Media kind (e.g. "movie", "series", "anime") from joined media table.
    pub kind: String,
    /// Poster image URL from joined media table.
    pub poster_url: Option<String>,
}

/// Opens a SQLite connection pool with WAL mode, foreign keys, and executes migrations.
pub async fn open(path: &Path) -> Result<SqlitePool, SlothError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .pragma("foreign_keys", "ON");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| SlothError::Database(sqlx::Error::Migrate(Box::new(e))))?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_db() {
        let pool = open(Path::new("sloth.db"))
            .await
            .expect("failed to open/migrate sloth.db");
        assert!(pool.is_closed() == false);
    }
}
