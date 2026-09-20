use chrono::{Datelike, Utc};
use sloth_tui::config::NotificationsConfig;
use sloth_tui::daemon::notifier::{
    check_and_notify_f1, purge_old_notifications, send_notification,
};
use sloth_tui::providers::f1::calendar::{save_calendar_to_cache, F1Session, F1SessionKind, F1SessionSlot};
use sloth_tui::tui::state::{AppState, SettingsCategory};

#[test]
fn test_send_notification_smoke() {
    // Should execute safely without panic regardless of feature flags or platform
    send_notification("Sloth Test", "Smoke test notification");
}

#[tokio::test]
async fn test_f1_session_notification_and_deduplication() {
    let pool = sloth_tui::db::open(std::path::Path::new(":memory:"))
        .await
        .expect("in-memory db must open and migrate");

    let now = Utc::now();
    let current_year = now.year() as u32;

    let test_sessions = vec![F1Session {
        round: 1,
        name: "Monaco Grand Prix".to_string(),
        circuit: "Circuit de Monaco".to_string(),
        country: "Monaco".to_string(),
        city: "Monte Carlo".to_string(),
        sessions: vec![
            F1SessionSlot {
                kind: F1SessionKind::Qualifying,
                starts_at: now + chrono::Duration::minutes(10),
                stream_url: None,
            },
            F1SessionSlot {
                kind: F1SessionKind::Race,
                starts_at: now + chrono::Duration::hours(24),
                stream_url: None,
            },
        ],
    }];

    save_calendar_to_cache(&pool, current_year, &test_sessions)
        .await
        .expect("must cache test F1 calendar");

    // First run with 15 min lead time: Qualifying is in 10 min, so it triggers 1 notification
    check_and_notify_f1(&pool, 15)
        .await
        .expect("F1 notifier check must succeed");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications_sent WHERE kind = 'session_start'")
        .fetch_one(&pool)
        .await
        .expect("query count");
    assert_eq!(count.0, 1, "exactly one notification must be recorded");

    // Second run: deduplication should ensure count remains 1
    check_and_notify_f1(&pool, 15)
        .await
        .expect("second F1 check must succeed");

    let count2: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications_sent WHERE kind = 'session_start'")
        .fetch_one(&pool)
        .await
        .expect("query count");
    assert_eq!(count2.0, 1, "notification must not be duplicated");
}

#[tokio::test]
async fn test_notification_retention_purge() {
    let pool = sloth_tui::db::open(std::path::Path::new(":memory:"))
        .await
        .expect("in-memory db must open and migrate");

    // Old entry: 8 days old (> 7 days)
    sqlx::query(
        "INSERT INTO notifications_sent (kind, reference_id, sent_at) VALUES ('race_start', 'old_event', unixepoch() - 700000)"
    )
    .execute(&pool)
    .await
    .expect("insert old notification");

    // Recent entry: 1 day old (< 7 days)
    sqlx::query(
        "INSERT INTO notifications_sent (kind, reference_id, sent_at) VALUES ('race_start', 'recent_event', unixepoch() - 86400)"
    )
    .execute(&pool)
    .await
    .expect("insert recent notification");

    let purged = purge_old_notifications(&pool).await.expect("purge must succeed");
    assert_eq!(purged, 1, "must purge exactly 1 expired notification");

    let remaining: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM notifications_sent")
        .fetch_one(&pool)
        .await
        .expect("query remaining count");
    assert_eq!(remaining.0, 1, "recent notification must be retained");
}

#[test]
fn test_notification_config_defaults() {
    let cfg = NotificationsConfig::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.f1_lead_time_minutes, 15);
    assert!(cfg.anime_alerts);
}

#[test]
fn test_settings_state_lead_time_cycle() {
    let mut state = AppState::default();
    assert_eq!(state.f1_lead_time_minutes, 15);

    // Forward cycle: 15 -> 30 -> 60 -> 5 -> 10 -> 15
    state.cycle_f1_lead_time(true);
    assert_eq!(state.f1_lead_time_minutes, 30);
    state.cycle_f1_lead_time(true);
    assert_eq!(state.f1_lead_time_minutes, 60);
    state.cycle_f1_lead_time(true);
    assert_eq!(state.f1_lead_time_minutes, 5);
    state.cycle_f1_lead_time(true);
    assert_eq!(state.f1_lead_time_minutes, 10);
    state.cycle_f1_lead_time(true);
    assert_eq!(state.f1_lead_time_minutes, 15);

    // Backward cycle: 15 -> 10 -> 5 -> 60
    state.cycle_f1_lead_time(false);
    assert_eq!(state.f1_lead_time_minutes, 10);
    state.cycle_f1_lead_time(false);
    assert_eq!(state.f1_lead_time_minutes, 5);
    state.cycle_f1_lead_time(false);
    assert_eq!(state.f1_lead_time_minutes, 60);
}
