//! TheMovieDatabase (TMDB) API v3 metadata provider and enrichment service.

use crate::SlothError;
use crate::providers::models::{CastMember, EpisodeRef, Media, MediaType};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::sync::Arc;

pub const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
pub const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/w500";

/// Client for TheMovieDatabase (TMDB) API v3.
#[derive(Clone)]
pub struct TmdbClient {
    http: Arc<reqwest::Client>,
    api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TmdbMovieResult {
    pub id: u32,
    pub title: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TmdbTvResult {
    pub id: u32,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbSearchResponse<T> {
    #[serde(default)]
    pub results: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbGenre {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbCastMember {
    pub id: u32,
    pub name: String,
    pub character: Option<String>,
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbCredits {
    pub cast: Option<Vec<TmdbCastMember>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbMovie {
    pub id: u32,
    pub title: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f32>,
    pub runtime: Option<u32>,
    pub genres: Option<Vec<TmdbGenre>>,
    pub credits: Option<TmdbCredits>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbTv {
    pub id: u32,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f32>,
    pub number_of_seasons: Option<u32>,
    pub number_of_episodes: Option<u32>,
    pub genres: Option<Vec<TmdbGenre>>,
    pub credits: Option<TmdbCredits>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbSeasonEpisode {
    pub episode_number: u32,
    pub season_number: u32,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub runtime: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TmdbSeason {
    pub season_number: u32,
    pub episodes: Option<Vec<TmdbSeasonEpisode>>,
}

impl TmdbClient {
    /// Constructs a new TMDB client.
    /// If no API key is provided, falls back to `TMDB_API_KEY` environment variable.
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            http: Arc::new(reqwest::Client::new()),
            api_key: api_key.or_else(|| std::env::var("TMDB_API_KEY").ok()),
        }
    }

    /// Constructs a TMDB client using a specific reqwest client and API key.
    pub fn with_http(http: Arc<reqwest::Client>, api_key: Option<String>) -> Self {
        Self { http, api_key }
    }

    /// Returns the active API key, if configured.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Computes full image URL given a TMDB poster or backdrop relative path.
    pub fn poster_url(path: &str) -> String {
        let clean = path.strip_prefix('/').unwrap_or(path);
        format!("{TMDB_IMAGE_BASE}/{clean}")
    }

    /// Searches for a movie by title and optional release year.
    pub async fn search_movie(
        &self,
        query: &str,
        year: Option<u32>,
    ) -> Result<Option<TmdbMovieResult>, SlothError> {
        let key = match &self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Ok(None),
        };

        let url = format!("{TMDB_BASE_URL}/search/movie");
        let mut req = self
            .http
            .get(&url)
            .query(&[("api_key", key.as_str()), ("query", query)]);

        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            req = req.query(&[("year", year_str.as_str())]);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let data: TmdbSearchResponse<TmdbMovieResult> = resp
            .json()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        Ok(data.results.into_iter().next())
    }

    /// Searches for a TV series by title and optional first air year.
    pub async fn search_tv(
        &self,
        query: &str,
        year: Option<u32>,
    ) -> Result<Option<TmdbTvResult>, SlothError> {
        let key = match &self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Ok(None),
        };

        let url = format!("{TMDB_BASE_URL}/search/tv");
        let mut req = self
            .http
            .get(&url)
            .query(&[("api_key", key.as_str()), ("query", query)]);

        let year_str;
        if let Some(y) = year {
            year_str = y.to_string();
            req = req.query(&[("first_air_date_year", year_str.as_str())]);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let data: TmdbSearchResponse<TmdbTvResult> = resp
            .json()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        Ok(data.results.into_iter().next())
    }

    /// Fetches full movie details including credits and recommendations.
    pub async fn movie_details(&self, id: u32) -> Result<Option<TmdbMovie>, SlothError> {
        let key = match &self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Ok(None),
        };

        let url = format!("{TMDB_BASE_URL}/movie/{id}");
        let resp = self
            .http
            .get(&url)
            .query(&[
                ("api_key", key.as_str()),
                ("append_to_response", "credits,recommendations"),
            ])
            .send()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let movie: TmdbMovie = resp
            .json()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        Ok(Some(movie))
    }

    /// Fetches full TV show details including credits and recommendations.
    pub async fn tv_details(&self, id: u32) -> Result<Option<TmdbTv>, SlothError> {
        let key = match &self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Ok(None),
        };

        let url = format!("{TMDB_BASE_URL}/tv/{id}");
        let resp = self
            .http
            .get(&url)
            .query(&[
                ("api_key", key.as_str()),
                ("append_to_response", "credits,recommendations"),
            ])
            .send()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let tv: TmdbTv = resp
            .json()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        Ok(Some(tv))
    }

    /// Fetches episode references for a specific TV show season.
    pub async fn tv_season_episodes(
        &self,
        id: u32,
        season: u32,
    ) -> Result<Vec<EpisodeRef>, SlothError> {
        let key = match &self.api_key {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Ok(Vec::new()),
        };

        let url = format!("{TMDB_BASE_URL}/tv/{id}/season/{season}");
        let resp = self
            .http
            .get(&url)
            .query(&[("api_key", key.as_str())])
            .send()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let season_data: TmdbSeason = resp
            .json()
            .await
            .map_err(|e| SlothError::Io(std::io::Error::other(e)))?;

        let episodes = season_data
            .episodes
            .unwrap_or_default()
            .into_iter()
            .map(|ep| EpisodeRef {
                season: ep.season_number,
                episode: ep.episode_number,
                title: ep.name,
                duration_secs: ep.runtime.map(|r| (r as f64) * 60.0),
            })
            .collect();

        Ok(episodes)
    }

    /// Enriches a media item with metadata from TMDB and updates the SQLite database.
    pub async fn enrich_media(&self, media: &mut Media) -> Result<(), SlothError> {
        if self.api_key.is_none() {
            return Ok(());
        }

        match media.media_type {
            MediaType::Movie => {
                if let Some(res) = self.search_movie(&media.title, media.year).await? {
                    if let Some(details) = self.movie_details(res.id).await? {
                        media.external_ids.tmdb = Some(details.id);

                        if let Some(path) = details.poster_path {
                            media.poster_url = Some(Self::poster_url(&path));
                        }
                        if let Some(path) = details.backdrop_path {
                            media.backdrop_url = Some(Self::poster_url(&path));
                        }
                        if let Some(desc) = details.overview {
                            if !desc.trim().is_empty() {
                                media.overview = Some(desc);
                            }
                        }
                        if let Some(vote) = details.vote_average {
                            if vote > 0.0 {
                                media.rating = Some(vote);
                            }
                        }
                        if let Some(runtime) = details.runtime {
                            if runtime > 0 {
                                media.duration_secs = Some((runtime as f64) * 60.0);
                            }
                        }
                        if let Some(genres) = details.genres {
                            for g in genres {
                                if !media
                                    .genres
                                    .iter()
                                    .any(|existing| existing.eq_ignore_ascii_case(&g.name))
                                {
                                    media.genres.push(g.name);
                                }
                            }
                        }
                        if let Some(credits) = details.credits {
                            if let Some(cast) = credits.cast {
                                media.cast = cast
                                    .into_iter()
                                    .take(10)
                                    .map(|c| CastMember {
                                        name: c.name,
                                        character: c.character,
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
            MediaType::Series => {
                if let Some(res) = self.search_tv(&media.title, media.year).await? {
                    if let Some(details) = self.tv_details(res.id).await? {
                        media.external_ids.tmdb = Some(details.id);

                        if let Some(path) = details.poster_path {
                            media.poster_url = Some(Self::poster_url(&path));
                        }
                        if let Some(path) = details.backdrop_path {
                            media.backdrop_url = Some(Self::poster_url(&path));
                        }
                        if let Some(desc) = details.overview {
                            if !desc.trim().is_empty() {
                                media.overview = Some(desc);
                            }
                        }
                        if let Some(vote) = details.vote_average {
                            if vote > 0.0 {
                                media.rating = Some(vote);
                            }
                        }
                        if let Some(s) = details.number_of_seasons {
                            media.seasons_count = Some(s);
                        }
                        if let Some(e) = details.number_of_episodes {
                            media.episodes_count = Some(e);
                        }
                        if let Some(genres) = details.genres {
                            for g in genres {
                                if !media
                                    .genres
                                    .iter()
                                    .any(|existing| existing.eq_ignore_ascii_case(&g.name))
                                {
                                    media.genres.push(g.name);
                                }
                            }
                        }
                        if let Some(credits) = details.credits {
                            if let Some(cast) = credits.cast {
                                media.cast = cast
                                    .into_iter()
                                    .take(10)
                                    .map(|c| CastMember {
                                        name: c.name,
                                        character: c.character,
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Persist enriched media record into SQLite
        let db_path = crate::config::db_path();
        if let Ok(pool) = crate::db::open(&db_path).await {
            let _ = Self::upsert_media(&pool, media).await;
        }

        Ok(())
    }

    /// Upserts a media record into the SQLite `media` table.
    pub async fn upsert_media(pool: &SqlitePool, media: &Media) -> Result<(), SlothError> {
        let kind_str = match media.media_type {
            MediaType::Movie => "movie",
            MediaType::Series => "series",
            MediaType::Anime => "anime",
            MediaType::LiveSport => "live_sport",
            MediaType::F1 => "f1",
            MediaType::IptvChannel => "iptv_channel",
        };

        let genres_json = serde_json::to_string(&media.genres).unwrap_or_else(|_| "[]".to_string());
        let external_id = media.external_ids.imdb.as_deref();
        let year_val = media.year.map(|y| y as i64);
        let rating_val = media.rating.map(|r| r as f64);
        let total_episodes_val = media.episodes_count.map(|e| e as i64);
        let total_seasons_val = media.seasons_count.map(|s| s as i64);
        let anilist_id_val = media.external_ids.anilist.map(|a| a as i64);
        let tmdb_id_val = media.external_ids.tmdb.map(|t| t as i64);
        let mal_id_val = media.external_ids.mal.map(|m| m as i64);

        sqlx::query(
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
        )
        .bind(&media.id)
        .bind(media.provider_id)
        .bind(external_id)
        .bind(&media.title)
        .bind(kind_str)
        .bind(year_val)
        .bind(&media.overview)
        .bind(&media.poster_url)
        .bind(rating_val)
        .bind(genres_json)
        .bind(total_episodes_val)
        .bind(total_seasons_val)
        .bind(anilist_id_val)
        .bind(tmdb_id_val)
        .bind(mal_id_val)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ExternalIds;

    #[test]
    fn test_poster_url_generation() {
        let url = TmdbClient::poster_url("/q6y0Go1tsGEsmtFryDOJo3dEmqu.jpg");
        assert_eq!(
            url,
            "https://image.tmdb.org/t/p/w500/q6y0Go1tsGEsmtFryDOJo3dEmqu.jpg"
        );
        let url_no_slash = TmdbClient::poster_url("q6y0Go1tsGEsmtFryDOJo3dEmqu.jpg");
        assert_eq!(
            url_no_slash,
            "https://image.tmdb.org/t/p/w500/q6y0Go1tsGEsmtFryDOJo3dEmqu.jpg"
        );
    }

    #[tokio::test]
    async fn test_graceful_skip_without_api_key() {
        let client = TmdbClient::new(None);
        let mut media = Media {
            id: "test:1".to_string(),
            title: "Inception".to_string(),
            media_type: MediaType::Movie,
            year: Some(2010),
            overview: None,
            poster_url: None,
            backdrop_url: None,
            genres: vec![],
            rating: None,
            duration_secs: None,
            seasons_count: None,
            episodes_count: None,
            provider_id: "test",
            external_ids: ExternalIds::default(),
            cast: vec![],
        };

        let res = client.enrich_media(&mut media).await;
        assert!(res.is_ok());
        assert!(media.external_ids.tmdb.is_none());
    }
}
