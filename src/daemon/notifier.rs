//! Background notification daemon sending OS desktop notifications for
//! upcoming F1 sessions and AniList episode broadcasts.

use std::time::Duration;
use chrono::{Datelike, Utc};
use sqlx::SqlitePool;

use crate::SlothError;
use crate::config::NotificationsConfig;
use crate::providers::f1::calendar::{get_cached_calendar, F1SessionKind};
use crate::tracking::AniListClient;

/// Dispatches an OS desktop notification with title and body.
pub fn send_notification(title: &str, body: &str) {
    #[cfg(feature = "notifications")]
    {
        // Linux/macOS
        #[cfg(all(not(target_os = "windows"), not(target_os = "android")))]
        {
            let _ = notify_rust::Notification::new()
                .appname("Sloth")
                .summary(title)
                .body(body)
                .icon("media-playback-start")
                .show();
        }

        // Windows: use notify_rust with WinRT toast backend
        #[cfg(target_os = "windows")]
        {
            let _ = notify_rust::Notification::new()
                .appname("Sloth")
                .summary(title)
                .body(body)
                .show();
        }

        #[cfg(target_os = "android")]
        {
            let _ = (title, body);
        }
    }

    #[cfg(not(feature = "notifications"))]
    {
        let _ = (title, body); // suppress unused warnings
    }
}

/// Checks for F1 sessions starting within the specified lead time and notifies if not already sent.
pub async fn check_and_notify_f1(pool: &SqlitePool, lead_time_minutes: u32) -> Result<(), SlothError> {
    let now = Utc::now();
    let season = now.year() as u32;

    let Some(sessions) = get_cached_calendar(pool, season).await? else {
        return Ok(());
    };

    let lead_duration = chrono::Duration::minutes(lead_time_minutes as i64);

    for race in sessions {
        for slot in &race.sessions {
            let diff = slot.starts_at - now;
            // If session starts in the future and within lead time
            if diff > chrono::Duration::zero() && diff <= lead_duration {
                let kind_str = match slot.kind {
                    F1SessionKind::Race => "race_start",
                    _ => "session_start",
                };
                let session_label = slot.kind.label();
                let reference_id = format!("f1_{}_{}_{:?}", season, race.round, slot.kind);

                let already_sent: Option<(i64,)> = sqlx::query_as(
                    "SELECT id FROM notifications_sent WHERE kind = ?1 AND reference_id = ?2",
                )
                .bind(kind_str)
                .bind(&reference_id)
                .fetch_optional(pool)
                .await?;

                if already_sent.is_none() {
                    let minutes_left = (diff.num_seconds() + 59) / 60;
                    let title = format!("F1: {} - {}", race.name, session_label);
                    let body = format!("Starting in ~{} min at {}", minutes_left, race.circuit);

                    send_notification(&title, &body);

                    let _ = sqlx::query(
                        r#"
                        INSERT OR IGNORE INTO notifications_sent (kind, reference_id, sent_at)
                        VALUES (?1, ?2, unixepoch())
                        "#,
                    )
                    .bind(kind_str)
                    .bind(&reference_id)
                    .execute(pool)
                    .await;
                }
            }
        }
    }

    Ok(())
}

/// Checks AniList airing schedule for user's tracked anime broadcasting soon or newly aired.
pub async fn check_and_notify_anilist(pool: &SqlitePool) -> Result<(), SlothError> {
    let Some(client) = AniListClient::authenticate(pool).await? else {
        return Ok(());
    };

    // Query active anime IDs tracked in SQLite
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT anilist_id FROM anilist_entries WHERE status = 'CURRENT' OR status IS NULL",
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(());
    }

    let media_ids: Vec<i64> = rows.into_iter().map(|(id,)| id).collect();

    let now = Utc::now().timestamp();
    // Look for episodes that aired within the last 2 hours or air in next 15 minutes
    let now_min = now - 7200;
    let now_max = now + 900;

    let query = r#"
        query ($mediaIds: [Int], $nowMin: Int, $nowMax: Int) {
            Page(page: 1, perPage: 25) {
                airingSchedules(mediaId_in: $mediaIds, airingAt_greater: $nowMin, airingAt_lesser: $nowMax) {
                    id
                    mediaId
                    episode
                    airingAt
                    media {
                        title {
                            userPreferred
                            english
                            romaji
                        }
                    }
                }
            }
        }
    "#;

    #[derive(serde::Deserialize)]
    struct PageData {
        #[serde(rename = "Page")]
        page: Option<AiringPage>,
    }
    #[derive(serde::Deserialize)]
    struct AiringPage {
        #[serde(rename = "airingSchedules")]
        airing_schedules: Option<Vec<AiringScheduleItem>>,
    }
    #[derive(serde::Deserialize)]
    struct AiringScheduleItem {
        #[serde(rename = "mediaId")]
        media_id: u32,
        episode: u32,
        media: Option<AiringMedia>,
    }
    #[derive(serde::Deserialize)]
    struct AiringMedia {
        title: Option<AiringTitle>,
    }
    #[derive(serde::Deserialize)]
    struct AiringTitle {
        #[serde(rename = "userPreferred")]
        user_preferred: Option<String>,
        english: Option<String>,
    }

    let variables = serde_json::json!({
        "mediaIds": media_ids,
        "nowMin": now_min,
        "nowMax": now_max,
    });

    let res: Result<PageData, _> = client.gql(query, variables).await;
    if let Ok(data) = res {
        if let Some(page) = data.page {
            if let Some(schedules) = page.airing_schedules {
                for item in schedules {
                    let ref_id = format!("anime_{}_{}", item.media_id, item.episode);
                    let already_sent: Option<(i64,)> = sqlx::query_as(
                        "SELECT id FROM notifications_sent WHERE kind = 'new_episode' AND reference_id = ?1",
                    )
                    .bind(&ref_id)
                    .fetch_optional(pool)
                    .await?;

                    if already_sent.is_none() {
                        let anime_name = item
                            .media
                            .as_ref()
                            .and_then(|m| m.title.as_ref())
                            .and_then(|t| t.user_preferred.as_ref().or(t.english.as_ref()))
                            .cloned()
                            .unwrap_or_else(|| format!("Anime #{}", item.media_id));

                        let title = format!("New Episode: {}", anime_name);
                        let body = format!("Episode {} is now available!", item.episode);

                        send_notification(&title, &body);

                        let _ = sqlx::query(
                            r#"
                            INSERT OR IGNORE INTO notifications_sent (kind, reference_id, sent_at)
                            VALUES ('new_episode', ?1, unixepoch())
                            "#,
                        )
                        .bind(&ref_id)
                        .execute(pool)
                        .await;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Purges notification records older than 7 days per docs/SCHEMA.md.
pub async fn purge_old_notifications(pool: &SqlitePool) -> Result<u64, SlothError> {
    let res = sqlx::query("DELETE FROM notifications_sent WHERE sent_at < (unixepoch() - 604800)")
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

/// Main notification daemon loop. Checks F1 calendar and AniList airing schedule periodically.
pub async fn run_notifier(pool: SqlitePool, config: NotificationsConfig) {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        if !config.enabled {
            continue;
        }

        if let Err(err) = check_and_notify_f1(&pool, config.f1_lead_time_minutes).await {
            log::debug!("Notification daemon F1 check: {err}");
        }

        if config.anime_alerts {
            if let Err(err) = check_and_notify_anilist(&pool).await {
                log::debug!("Notification daemon AniList check: {err}");
            }
        }

        let _ = purge_old_notifications(&pool).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::f1::calendar::{save_calendar_to_cache, F1Session, F1SessionSlot};
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    async fn setup_test_db() -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("Failed to connect to in-memory sqlite db");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_f1_notification_sent_and_deduplicated() {
        let pool = setup_test_db().await;

        let now = Utc::now();
        let session = F1Session {
            round: 1,
            name: "Bahrain Grand Prix".to_string(),
            circuit: "Sakhir".to_string(),
            country: "Bahrain".to_string(),
            city: "Sakhir".to_string(),
            sessions: vec![F1SessionSlot {
                kind: F1SessionKind::Race,
                starts_at: now + chrono::Duration::minutes(10),
                stream_url: None,
            }],
        };

        save_calendar_to_cache(&pool, now.year() as u32, &[session]).await.unwrap();

        // First check: should insert 1 notification
        check_and_notify_f1(&pool, 15).await.unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications_sent")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);

        // Second check: duplicate should not insert again
        check_and_notify_f1(&pool, 15).await.unwrap();
        let count_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications_sent")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count_after.0, 1);
    }

    #[tokio::test]
    async fn test_purge_old_notifications() {
        let pool = setup_test_db().await;

        sqlx::query(
            "INSERT INTO notifications_sent (kind, reference_id, sent_at) VALUES ('race_start', 'old_race', unixepoch() - 800000)"
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO notifications_sent (kind, reference_id, sent_at) VALUES ('race_start', 'new_race', unixepoch())"
        )
        .execute(&pool)
        .await
        .unwrap();

        let purged = purge_old_notifications(&pool).await.unwrap();
        assert_eq!(purged, 1);

        let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications_sent")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining.0, 1);
    }

    #[test]
    fn test_send_notification_no_panic() {
        send_notification("Test Notification", "Testing daemon notification delivery");
    }
}
