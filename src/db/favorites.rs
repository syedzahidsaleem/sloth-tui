//! Favorites and bookmarks persistence.

use sqlx::SqlitePool;

use crate::SlothError;
use crate::providers::models::{ExternalIds, Media, MediaType};

fn media_type_to_str(kind: MediaType) -> &'static str {
    match kind {
        MediaType::Movie => "movie",
        MediaType::Series => "series",
        MediaType::Anime => "anime",
        MediaType::LiveSport => "live_sport",
        MediaType::F1 => "f1",
        MediaType::IptvChannel => "iptv_channel",
    }
}

fn str_to_media_type(s: &str) -> MediaType {
    match s {
        "series" => MediaType::Series,
        "anime" => MediaType::Anime,
        "live_sport" => MediaType::LiveSport,
        "f1" => MediaType::F1,
        "iptv_channel" => MediaType::IptvChannel,
        _ => MediaType::Movie,
    }
}

/// Adds a media item to user favorites, ensuring the media metadata is upserted first.
pub async fn add(pool: &SqlitePool, media: &Media) -> Result<(), SlothError> {
    let kind_str = media_type_to_str(media.media_type);
    let year_val = media.year.map(|y| y as i64);
    let rating_val = media.rating.map(|r| r as f64);
    let total_episodes_val = media.episodes_count.map(|e| e as i64);
    let total_seasons_val = media.seasons_count.map(|s| s as i64);
    let anilist_id_val = media.external_ids.anilist.map(|a| a as i64);
    let tmdb_id_val = media.external_ids.tmdb.map(|t| t as i64);
    let mal_id_val = media.external_ids.mal.map(|m| m as i64);
    let external_id = media.external_ids.imdb.as_deref();
    let genres_json = serde_json::to_string(&media.genres).unwrap_or_else(|_| "[]".to_string());

    // 1. Upsert into media table
    sqlx::query!(
        r#"
        INSERT INTO media (
            id, provider_id, external_id, title, kind, year, description,
            poster_url, rating, genres, total_episodes, total_seasons,
            anilist_id, tmdb_id, mal_id, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, unixepoch())
        ON CONFLICT(id) DO UPDATE SET
            provider_id = excluded.provider_id,
            external_id = coalesce(excluded.external_id, media.external_id),
            title = excluded.title,
            kind = excluded.kind,
            year = coalesce(excluded.year, media.year),
            description = coalesce(excluded.description, media.description),
            poster_url = coalesce(excluded.poster_url, media.poster_url),
            rating = coalesce(excluded.rating, media.rating),
            genres = excluded.genres,
            total_episodes = coalesce(excluded.total_episodes, media.total_episodes),
            total_seasons = coalesce(excluded.total_seasons, media.total_seasons),
            anilist_id = coalesce(excluded.anilist_id, media.anilist_id),
            tmdb_id = coalesce(excluded.tmdb_id, media.tmdb_id),
            mal_id = coalesce(excluded.mal_id, media.mal_id),
            updated_at = unixepoch()
        "#,
        media.id,
        media.provider_id,
        external_id,
        media.title,
        kind_str,
        year_val,
        media.overview,
        media.poster_url,
        rating_val,
        genres_json,
        total_episodes_val,
        total_seasons_val,
        anilist_id_val,
        tmdb_id_val,
        mal_id_val
    )
    .execute(pool)
    .await?;

    // 2. Insert into favorites table
    sqlx::query!(
        r#"
        INSERT OR IGNORE INTO favorites (media_id, added_at)
        VALUES (?1, unixepoch())
        "#,
        media.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Removes a media item from user favorites by its identifier.
pub async fn remove(pool: &SqlitePool, media_id: &str) -> Result<(), SlothError> {
    sqlx::query!(
        r#"
        DELETE FROM favorites WHERE media_id = ?1
        "#,
        media_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Checks whether a media item is currently marked as a favorite.
pub async fn is_favorite(pool: &SqlitePool, media_id: &str) -> Result<bool, SlothError> {
    let row = sqlx::query!(
        r#"
        SELECT media_id FROM favorites WHERE media_id = ?1
        "#,
        media_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

/// Lists all user favorites joined with full media metadata, ordered by added date descending.
pub async fn list(pool: &SqlitePool) -> Result<Vec<Media>, SlothError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            m.id as "id!",
            m.provider_id as "provider_id!",
            m.title as "title!",
            m.kind as "kind!",
            m.year,
            m.description,
            m.poster_url,
            m.rating,
            m.genres,
            m.total_episodes,
            m.total_seasons,
            m.anilist_id,
            m.tmdb_id,
            m.mal_id,
            m.external_id
        FROM favorites f
        JOIN media m ON m.id = f.media_id
        ORDER BY f.added_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let media_items = rows
        .into_iter()
        .map(|r| {
            let genres = r
                .genres
                .and_then(|g| serde_json::from_str(&g).ok())
                .unwrap_or_default();

            Media {
                id: r.id,
                provider_id: Box::leak(r.provider_id.into_boxed_str()),
                media_type: str_to_media_type(&r.kind),
                title: r.title,
                year: r.year.map(|y| y as u32),
                poster_url: r.poster_url,
                backdrop_url: None,
                rating: r.rating.map(|rt| rt as f32),
                duration_secs: None,
                overview: r.description,
                genres,
                episodes_count: r.total_episodes.map(|e| e as u32),
                seasons_count: r.total_seasons.map(|s| s as u32),
                external_ids: ExternalIds {
                    tmdb: r.tmdb_id.map(|t| t as u32),
                    anilist: r.anilist_id.map(|a| a as u32),
                    mal: r.mal_id.map(|m| m as u32),
                    tvdb: None,
                    imdb: r.external_id,
                },
                cast: Vec::new(),
            }
        })
        .collect();

    Ok(media_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_favorites_crud() {
        let temp = NamedTempFile::new().expect("create temp file");
        let pool = open(temp.path()).await.expect("open db");

        let media = Media {
            id: "test:fav1".to_string(),
            provider_id: "test_provider",
            media_type: MediaType::Movie,
            title: "Test Movie".to_string(),
            year: Some(2026),
            poster_url: Some("https://example.com/poster.jpg".to_string()),
            backdrop_url: None,
            rating: Some(8.5),
            duration_secs: None,
            overview: Some("A test movie".to_string()),
            genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
            episodes_count: None,
            seasons_count: None,
            external_ids: ExternalIds {
                imdb: Some("tt1234567".to_string()),
                tmdb: Some(101),
                ..Default::default()
            },
            cast: Vec::new(),
        };

        assert_eq!(is_favorite(&pool, "test:fav1").await.unwrap(), false);

        add(&pool, &media).await.unwrap();
        assert_eq!(is_favorite(&pool, "test:fav1").await.unwrap(), true);

        let favs = list(&pool).await.unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].id, "test:fav1");
        assert_eq!(favs[0].title, "Test Movie");
        assert_eq!(favs[0].genres, vec!["Action", "Sci-Fi"]);

        remove(&pool, "test:fav1").await.unwrap();
        assert_eq!(is_favorite(&pool, "test:fav1").await.unwrap(), false);
        assert_eq!(list(&pool).await.unwrap().len(), 0);
    }
}
