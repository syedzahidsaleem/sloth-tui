//! Integration tests for Formula 1 calendar parsing and next session computation.

use chrono::{DateTime, Utc};
use sloth_tui::providers::f1::calendar::{
    F1SessionKind, next_session_at, parse_calendar_json, time_until_at,
};

const FIXTURE_JSON: &str = include_str!("fixtures/f1/calendar_2026.json");

#[test]
fn test_calendar_parses_correctly_from_fixture() {
    let calendar = parse_calendar_json(FIXTURE_JSON).expect("Failed to parse calendar JSON");

    assert_eq!(calendar.len(), 3, "Expected 3 Grand Prix races");

    // Round 1: Bahrain
    let r1 = &calendar[0];
    assert_eq!(r1.round, 1);
    assert_eq!(r1.name, "Bahrain Grand Prix");
    assert_eq!(r1.circuit, "Bahrain International Circuit");
    assert_eq!(r1.country, "Bahrain");
    assert_eq!(r1.city, "Sakhir");
    assert_eq!(
        r1.sessions.len(),
        5,
        "Expected FP1, FP2, FP3, Qualifying, Race"
    );

    assert_eq!(r1.sessions[0].kind, F1SessionKind::FreePractice1);
    assert_eq!(r1.sessions[1].kind, F1SessionKind::FreePractice2);
    assert_eq!(r1.sessions[2].kind, F1SessionKind::FreePractice3);
    assert_eq!(r1.sessions[3].kind, F1SessionKind::Qualifying);
    assert_eq!(r1.sessions[4].kind, F1SessionKind::Race);

    // Verify session order is strictly chronological
    for i in 1..r1.sessions.len() {
        assert!(
            r1.sessions[i].starts_at >= r1.sessions[i - 1].starts_at,
            "Sessions should be sorted chronologically"
        );
    }

    // Round 2: Saudi Arabia (Sprint weekend)
    let r2 = &calendar[1];
    assert_eq!(r2.round, 2);
    assert_eq!(r2.name, "Saudi Arabian Grand Prix");
    assert_eq!(r2.circuit, "Jeddah Corniche Circuit");
    assert_eq!(r2.country, "Saudi Arabia");
    assert_eq!(r2.city, "Jeddah");
    assert_eq!(r2.sessions.len(), 5);

    let kinds: Vec<_> = r2.sessions.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        vec![
            F1SessionKind::FreePractice1,
            F1SessionKind::SprintQualifying,
            F1SessionKind::Sprint,
            F1SessionKind::Qualifying,
            F1SessionKind::Race,
        ]
    );

    // Round 3: Australia
    let r3 = &calendar[2];
    assert_eq!(r3.round, 3);
    assert_eq!(r3.name, "Australian Grand Prix");
    assert_eq!(r3.city, "Melbourne");
    assert_eq!(r3.country, "Australia");
}

#[test]
fn test_next_session_returns_correct_next_event() {
    let calendar = parse_calendar_json(FIXTURE_JSON).expect("Failed to parse calendar JSON");

    // Case 1: Before the start of the season
    let before_season: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-02-20T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let (session, slot) = next_session_at(&calendar, before_season)
        .expect("Should find upcoming session before season start");
    assert_eq!(session.round, 1);
    assert_eq!(slot.kind, F1SessionKind::FreePractice1);

    // Case 2: Right before Bahrain Race on March 1, 2026
    let before_bahrain_race: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-03-01T14:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let (session, slot) = next_session_at(&calendar, before_bahrain_race)
        .expect("Should find Bahrain Race as next session");
    assert_eq!(session.round, 1);
    assert_eq!(slot.kind, F1SessionKind::Race);

    // Case 3: Between Bahrain and Saudi Arabia GP
    let mid_march: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-03-03T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let (session, slot) = next_session_at(&calendar, mid_march)
        .expect("Should find Saudi Arabia FP1 as next session");
    assert_eq!(session.round, 2);
    assert_eq!(slot.kind, F1SessionKind::FreePractice1);

    // Case 4: Right before Saudi Arabia Sprint on March 7, 2026
    let before_sprint: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-03-07T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let (session, slot) = next_session_at(&calendar, before_sprint)
        .expect("Should find Saudi Arabia Sprint as next session");
    assert_eq!(session.round, 2);
    assert_eq!(slot.kind, F1SessionKind::Sprint);

    // Case 5: Far in the future after all races
    let future: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-12-31T23:59:59Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(
        next_session_at(&calendar, future).is_none(),
        "Should return None when all sessions are in the past"
    );
}

#[test]
fn test_time_until_calculation() {
    let calendar = parse_calendar_json(FIXTURE_JSON).expect("Failed to parse calendar JSON");
    let slot = &calendar[0].sessions[0]; // Bahrain FP1 at 2026-02-27T11:30:00Z

    // 1 hour before
    let one_hour_before: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-02-27T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = time_until_at(slot, one_hour_before);
    assert_eq!(duration.as_secs(), 3600);

    // After start
    let after_start: DateTime<Utc> = DateTime::parse_from_rfc3339("2026-02-27T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let duration = time_until_at(slot, after_start);
    assert_eq!(duration, std::time::Duration::ZERO);
}
