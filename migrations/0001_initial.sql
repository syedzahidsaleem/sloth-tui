-- ============================================================
-- Schema Version
-- ============================================================
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA user_version = 1;

-- ============================================================
-- 1. Media Cache
-- Normalized store for discovered media items.
-- Populated on search/browse, enriched by metadata providers.
-- ============================================================
CREATE TABLE IF NOT EXISTS media (
    id              TEXT PRIMARY KEY,           -- provider-scoped: "moviebox:12345"
    provider_id     TEXT NOT NULL,              -- "moviebox" | "hianime" | "streamed" | "f1"
    external_id     TEXT,                       -- TMDB id, AniList id, etc.
    title           TEXT NOT NULL,
    kind            TEXT NOT NULL CHECK(kind IN (
                        'movie', 'series', 'anime', 'live_sport', 'f1', 'iptv_channel'
                    )),
    year            INTEGER,
    description     TEXT,
    poster_url      TEXT,
    poster_cached   INTEGER DEFAULT 0,          -- 1 if poster is in local cache
    rating          REAL,                       -- 0.0–10.0
    genres          TEXT,                       -- JSON array: ["Action", "Drama"]
    total_episodes  INTEGER,                    -- NULL for movies / live
    total_seasons   INTEGER,                    -- NULL for movies / live
    anilist_id      INTEGER,                    -- AniList media id
    tmdb_id         INTEGER,                    -- TMDB id
    mal_id          INTEGER,                    -- MAL id
    created_at      INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at      INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_media_kind    ON media(kind);
CREATE INDEX IF NOT EXISTS idx_media_title   ON media(title);
CREATE INDEX IF NOT EXISTS idx_media_tmdb    ON media(tmdb_id);
CREATE INDEX IF NOT EXISTS idx_media_anilist ON media(anilist_id);

-- ============================================================
-- 2. Watch History
-- One row per (media_id, season, episode).
-- For movies: season = 0, episode = 0.
-- For live/F1: season = 0, episode = session_id.
-- ============================================================
CREATE TABLE IF NOT EXISTS watch_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    media_id        TEXT NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    season          INTEGER NOT NULL DEFAULT 0,
    episode         INTEGER NOT NULL DEFAULT 0,
    episode_title   TEXT,                       -- episode name if known
    watched_at      INTEGER NOT NULL DEFAULT (unixepoch()),
    resume_position REAL DEFAULT 0.0,           -- seconds from mpv IPC time-pos
    duration        REAL,                       -- total duration in seconds
    completed       INTEGER DEFAULT 0,          -- 1 if watched >= 85% of duration
    source_provider TEXT,                       -- which provider delivered the stream
    quality         TEXT,                       -- "4K" | "1080p" | "720p" etc.
    UNIQUE(media_id, season, episode)
);

CREATE INDEX IF NOT EXISTS idx_history_media     ON watch_history(media_id);
CREATE INDEX IF NOT EXISTS idx_history_watched   ON watch_history(watched_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_completed ON watch_history(completed);

-- ============================================================
-- 3. Favorites / Bookmarks
-- Simple per-media bookmark. No episode granularity.
-- ============================================================
CREATE TABLE IF NOT EXISTS favorites (
    media_id        TEXT PRIMARY KEY REFERENCES media(id) ON DELETE CASCADE,
    added_at        INTEGER NOT NULL DEFAULT (unixepoch()),
    notes           TEXT                        -- optional user note
);

-- ============================================================
-- 4. AniList Tracking
-- Mirrors AniList list entry state locally.
-- Synced bidirectionally: local -> AniList, AniList -> local.
-- ============================================================
CREATE TABLE IF NOT EXISTS anilist_entries (
    anilist_id      INTEGER PRIMARY KEY,        -- AniList media id
    media_id        TEXT REFERENCES media(id),  -- local media id (may be NULL if not searched yet)
    status          TEXT CHECK(status IN (
                        'CURRENT', 'PLANNING', 'COMPLETED',
                        'DROPPED', 'PAUSED', 'REPEATING'
                    )),
    progress        INTEGER DEFAULT 0,          -- episodes watched
    score           REAL,                       -- 0.0–10.0 (user score)
    notes           TEXT,
    synced_at       INTEGER NOT NULL DEFAULT (unixepoch()),
    dirty           INTEGER DEFAULT 0           -- 1 = needs push to AniList
);

-- ============================================================
-- 5. Trakt Tracking
-- Mirrors Trakt watched state locally.
-- ============================================================
CREATE TABLE IF NOT EXISTS trakt_entries (
    trakt_id        TEXT PRIMARY KEY,           -- "movie:123456" or "show:789:s1e2"
    media_id        TEXT REFERENCES media(id),
    kind            TEXT CHECK(kind IN ('movie', 'episode')),
    watched_at      INTEGER,
    synced_at       INTEGER NOT NULL DEFAULT (unixepoch()),
    dirty           INTEGER DEFAULT 0           -- 1 = needs push to Trakt
);

-- ============================================================
-- 6. Provider Session / Tokens
-- OAuth tokens and session data. NOT plaintext where keyring available.
-- Falls back here if keyring unavailable (e.g., headless Termux).
-- ============================================================
CREATE TABLE IF NOT EXISTS auth_tokens (
    provider        TEXT PRIMARY KEY,           -- "anilist" | "trakt"
    access_token    TEXT NOT NULL,
    refresh_token   TEXT,
    expires_at      INTEGER,                    -- unix timestamp
    scope           TEXT,
    created_at      INTEGER NOT NULL DEFAULT (unixepoch())
);

-- ============================================================
-- 7. F1 Calendar Cache
-- Cached race calendar for offline use + countdown display.
-- ============================================================
CREATE TABLE IF NOT EXISTS f1_calendar (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    season          INTEGER NOT NULL,
    round           INTEGER NOT NULL,
    name            TEXT NOT NULL,              -- "Bahrain Grand Prix"
    circuit         TEXT NOT NULL,              -- "Bahrain International Circuit"
    country         TEXT NOT NULL,
    city            TEXT NOT NULL,
    fp1_time        INTEGER,                    -- unix timestamp (UTC)
    fp2_time        INTEGER,
    fp3_time        INTEGER,
    qualifying_time INTEGER,
    sprint_time     INTEGER,                    -- NULL if no sprint
    race_time       INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(season, round)
);

CREATE INDEX IF NOT EXISTS idx_f1_season ON f1_calendar(season);
CREATE INDEX IF NOT EXISTS idx_f1_race   ON f1_calendar(race_time);

-- ============================================================
-- 8. Sports Events Cache
-- Short-lived cache (5 min TTL) of live sports schedule.
-- ============================================================
CREATE TABLE IF NOT EXISTS sports_events (
    id              TEXT PRIMARY KEY,           -- streamed.pk event id
    sport           TEXT NOT NULL,              -- "football" | "f1" | "cricket" etc.
    title           TEXT NOT NULL,
    home_team       TEXT,
    away_team       TEXT,
    competition     TEXT,                       -- "Premier League", "Champions League"
    starts_at       INTEGER,                    -- unix timestamp
    is_live         INTEGER DEFAULT 0,
    is_popular      INTEGER DEFAULT 0,
    streams_json    TEXT,                       -- JSON: [{id, hd_url, quality}]
    fetched_at      INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_sports_sport ON sports_events(sport);
CREATE INDEX IF NOT EXISTS idx_sports_time  ON sports_events(starts_at);
CREATE INDEX IF NOT EXISTS idx_sports_live  ON sports_events(is_live DESC);

-- ============================================================
-- 9. Notification Log
-- Track which notifications have been sent to avoid duplicates.
-- ============================================================
CREATE TABLE IF NOT EXISTS notifications_sent (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    kind            TEXT NOT NULL,              -- "new_episode" | "race_start" | "match_start"
    reference_id    TEXT NOT NULL,              -- media_id or event_id
    sent_at         INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(kind, reference_id)
);

-- ============================================================
-- 10. Schema Migrations Log
-- ============================================================
CREATE TABLE IF NOT EXISTS schema_migrations (
    version         INTEGER PRIMARY KEY,
    applied_at      INTEGER NOT NULL DEFAULT (unixepoch()),
    description     TEXT
);

INSERT OR IGNORE INTO schema_migrations (version, description)
VALUES (1, 'Initial schema — media, history, favorites, tracking, F1, sports');
