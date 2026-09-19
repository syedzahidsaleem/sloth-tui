//! Media metadata providers (TMDB, AniList, etc.).

pub mod anilist;
pub mod tmdb;

pub use tmdb::TmdbClient;
