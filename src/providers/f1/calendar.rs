//! Formula 1 calendar provider fetching race schedules from Jolpica / Ergast API
//! and caching them in SQLite with a 24-hour TTL.

use std::time::Duration;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};

use crate::SlothError;
pub use crate::tui::state::{F1Session, F1SessionKind, F1SessionSlot};

const JOLPICA_BASE_URL: &str = "https://api.jolpi.ca/ergast/f1";
const ERGAST_BASE_URL: &str = "https://ergast.com/api/f1";
const CALENDAR_TTL_SECS: i64 = 24 * 3600;

#[derive(Debug, Deserialize)]
struct ErgastResponse {
    #[serde(rename = "MRData")]
    mr_data: MrData,
}

#[derive(Debug, Deserialize)]
struct MrData {
    #[serde(rename = "RaceTable")]
    race_table: Option<RaceTable>,
}

#[derive(Debug, Deserialize)]
struct RaceTable {
    #[serde(rename = "Races")]
    races: Option<Vec<ErgastRace>>,
}

#[derive(Debug, Deserialize)]
struct ErgastRace {
    round: Option<String>,
    #[serde(rename = "raceName")]
    race_name: Option<String>,
    #[serde(rename = "Circuit")]
    circuit: Option<ErgastCircuit>,
    date: Option<String>,
    time: Option<String>,
    #[serde(rename = "FirstPractice")]
    first_practice: Option<ErgastSessionTime>,
    #[serde(rename = "SecondPractice")]
    second_practice: Option<ErgastSessionTime>,
    #[serde(rename = "ThirdPractice")]
    third_practice: Option<ErgastSessionTime>,
    #[serde(rename = "Qualifying")]
    qualifying: Option<ErgastSessionTime>,
    #[serde(rename = "Sprint")]
    sprint: Option<ErgastSessionTime>,
    #[serde(rename = "SprintQualifying", alias = "SprintShootout")]
    sprint_qualifying: Option<ErgastSessionTime>,
}

#[derive(Debug, Deserialize)]
struct ErgastCircuit {
    #[serde(rename = "circuitName")]
    circuit_name: Option<String>,
    #[serde(rename = "Location")]
    location: Option<ErgastLocation>,
}

#[derive(Debug, Deserialize)]
struct ErgastLocation {
    country: Option<String>,
    locality: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErgastSessionTime {
    date: Option<String>,
    time: Option<String>,
}

/// Parses an RFC 3339 or date/time string into a UTC `DateTime`.
fn parse_datetime(date_str: &str, time_str: Option<&str>) -> Option<DateTime<Utc>> {
    let raw_time = time_str.unwrap_or("12:00:00Z");
    let trimmed_time = raw_time.trim();
    let formatted = if trimmed_time.ends_with('Z') || trimmed_time.contains('+') || trimmed_time.contains('-') {
        format!("{}T{}", date_str.trim(), trimmed_time)
    } else {
        format!("{}T{}Z", date_str.trim(), trimmed_time)
    };

    if let Ok(dt) = DateTime::parse_from_rfc3339(&formatted) {
        return Some(dt.with_timezone(&Utc));
    }

    if let Ok(naive_date) = NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
        if let Some(naive_dt) = naive_date.and_hms_opt(12, 0, 0) {
            return Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }
    }
    None
}

/// Parses a raw Ergast / Jolpica JSON response into a list of [`F1Session`] items.
pub fn parse_calendar_json(json: &str) -> Result<Vec<F1Session>, SlothError> {
    let resp: ErgastResponse = serde_json::from_str(json)
        .map_err(|e| SlothError::Provider(format!("Failed to parse F1 calendar JSON: {e}")))?;

    let races = resp
        .mr_data
        .race_table
        .and_then(|rt| rt.races)
        .unwrap_or_default();

    let mut sessions = Vec::with_capacity(races.len());

    for race in races {
        let round = race
            .round
            .as_deref()
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);

        let name = race
            .race_name
            .unwrap_or_else(|| format!("Round {round} Grand Prix"));

        let circuit = race
            .circuit
            .as_ref()
            .and_then(|c| c.circuit_name.clone())
            .unwrap_or_default();

        let country = race
            .circuit
            .as_ref()
            .and_then(|c| c.location.as_ref())
            .and_then(|l| l.country.clone())
            .unwrap_or_default();

        let city = race
            .circuit
            .as_ref()
            .and_then(|c| c.location.as_ref())
            .and_then(|l| l.locality.clone())
            .unwrap_or_default();

        let mut session_slots = Vec::new();

        if let Some(fp1) = &race.first_practice {
            if let Some(d) = &fp1.date {
                if let Some(ts) = parse_datetime(d, fp1.time.as_deref()) {
                    session_slots.push(F1SessionSlot {
                        kind: F1SessionKind::FreePractice1,
                        starts_at: ts,
                        stream_url: None,
                    });
                }
            }
        }

        if let Some(fp2) = &race.second_practice {
            if let Some(d) = &fp2.date {
                if let Some(ts) = parse_datetime(d, fp2.time.as_deref()) {
                    session_slots.push(F1SessionSlot {
                        kind: F1SessionKind::FreePractice2,
                        starts_at: ts,
                        stream_url: None,
                    });
                }
            }
        }

        if let Some(sq) = &race.sprint_qualifying {
            if let Some(d) = &sq.date {
                if let Some(ts) = parse_datetime(d, sq.time.as_deref()) {
                    session_slots.push(F1SessionSlot {
                        kind: F1SessionKind::SprintQualifying,
                        starts_at: ts,
                        stream_url: None,
                    });
                }
            }
        }

        if let Some(fp3) = &race.third_practice {
            if let Some(d) = &fp3.date {
                if let Some(ts) = parse_datetime(d, fp3.time.as_deref()) {
                    session_slots.push(F1SessionSlot {
                        kind: F1SessionKind::FreePractice3,
                        starts_at: ts,
                        stream_url: None,
                    });
                }
            }
        }

        if let Some(sprint) = &race.sprint {
            if let Some(d) = &sprint.date {
                if let Some(ts) = parse_datetime(d, sprint.time.as_deref()) {
                    session_slots.push(F1SessionSlot {
                        kind: F1SessionKind::Sprint,
                        starts_at: ts,
                        stream_url: None,
                    });
                }
            }
        }

        if let Some(quali) = &race.qualifying {
            if let Some(d) = &quali.date {
                if let Some(ts) = parse_datetime(d, quali.time.as_deref()) {
                    session_slots.push(F1SessionSlot {
                        kind: F1SessionKind::Qualifying,
                        starts_at: ts,
                        stream_url: None,
                    });
                }
            }
        }

        if let Some(date) = &race.date {
            if let Some(ts) = parse_datetime(date, race.time.as_deref()) {
                session_slots.push(F1SessionSlot {
                    kind: F1SessionKind::Race,
                    starts_at: ts,
                    stream_url: None,
                });
            }
        }

        session_slots.sort_by_key(|s| s.starts_at);

        sessions.push(F1Session {
            round,
            name,
            circuit,
            country,
            city,
            sessions: session_slots,
        });
    }

    sessions.sort_by_key(|s| s.round);
    Ok(sessions)
}

/// Retrieves the cached F1 calendar from SQLite if it is newer than 24 hours.
pub async fn get_cached_calendar(
    pool: &SqlitePool,
    season: u32,
) -> Result<Option<Vec<F1Session>>, SlothError> {
    let rows = sqlx::query(
        r#"
        SELECT season, round, name, circuit, country, city,
               fp1_time, fp2_time, fp3_time, qualifying_time, sprint_time, race_time, updated_at
        FROM f1_calendar
        WHERE season = ?
        ORDER BY round ASC
        "#,
    )
    .bind(season as i64)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    // Check TTL from first row
    let updated_at: i64 = rows[0].try_get("updated_at").unwrap_or(0);
    let now_ts = Utc::now().timestamp();
    if now_ts - updated_at > CALENDAR_TTL_SECS {
        return Ok(None);
    }

    let mut sessions = Vec::with_capacity(rows.len());
    for row in rows {
        let round: i64 = row.try_get("round").unwrap_or(0);
        let name: String = row.try_get("name").unwrap_or_default();
        let circuit: String = row.try_get("circuit").unwrap_or_default();
        let country: String = row.try_get("country").unwrap_or_default();
        let city: String = row.try_get("city").unwrap_or_default();

        let mut slots = Vec::new();
        let add_slot = |ts: Option<i64>, kind: F1SessionKind, slots: &mut Vec<F1SessionSlot>| {
            if let Some(t) = ts {
                if let Some(dt) = DateTime::from_timestamp(t, 0) {
                    slots.push(F1SessionSlot {
                        kind,
                        starts_at: dt,
                        stream_url: None,
                    });
                }
            }
        };

        add_slot(row.try_get("fp1_time").ok(), F1SessionKind::FreePractice1, &mut slots);
        add_slot(row.try_get("fp2_time").ok(), F1SessionKind::FreePractice2, &mut slots);
        add_slot(row.try_get("fp3_time").ok(), F1SessionKind::FreePractice3, &mut slots);
        add_slot(row.try_get("sprint_time").ok(), F1SessionKind::Sprint, &mut slots);
        add_slot(row.try_get("qualifying_time").ok(), F1SessionKind::Qualifying, &mut slots);
        add_slot(row.try_get("race_time").ok(), F1SessionKind::Race, &mut slots);

        slots.sort_by_key(|s| s.starts_at);

        sessions.push(F1Session {
            round: round as u32,
            name,
            circuit,
            country,
            city,
            sessions: slots,
        });
    }

    Ok(Some(sessions))
}

/// Persists an F1 calendar into SQLite's `f1_calendar` table.
pub async fn save_calendar_to_cache(
    pool: &SqlitePool,
    season: u32,
    sessions: &[F1Session],
) -> Result<(), SlothError> {
    for s in sessions {
        let mut fp1_time: Option<i64> = None;
        let mut fp2_time: Option<i64> = None;
        let mut fp3_time: Option<i64> = None;
        let mut qualifying_time: Option<i64> = None;
        let mut sprint_time: Option<i64> = None;
        let mut race_time: i64 = 0;

        for slot in &s.sessions {
            let ts = slot.starts_at.timestamp();
            match slot.kind {
                F1SessionKind::FreePractice1 => fp1_time = Some(ts),
                F1SessionKind::FreePractice2 => fp2_time = Some(ts),
                F1SessionKind::FreePractice3 => fp3_time = Some(ts),
                F1SessionKind::Sprint | F1SessionKind::SprintQualifying => sprint_time = Some(ts),
                F1SessionKind::Qualifying => qualifying_time = Some(ts),
                F1SessionKind::Race => race_time = ts,
            }
        }

        if race_time == 0 {
            race_time = s.sessions.last().map(|sl| sl.starts_at.timestamp()).unwrap_or(0);
        }

        sqlx::query(
            r#"
            INSERT INTO f1_calendar (
                season, round, name, circuit, country, city,
                fp1_time, fp2_time, fp3_time, qualifying_time, sprint_time, race_time, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())
            ON CONFLICT(season, round) DO UPDATE SET
                name = excluded.name,
                circuit = excluded.circuit,
                country = excluded.country,
                city = excluded.city,
                fp1_time = excluded.fp1_time,
                fp2_time = excluded.fp2_time,
                fp3_time = excluded.fp3_time,
                qualifying_time = excluded.qualifying_time,
                sprint_time = excluded.sprint_time,
                race_time = excluded.race_time,
                updated_at = unixepoch()
            "#,
        )
        .bind(season as i64)
        .bind(s.round as i64)
        .bind(&s.name)
        .bind(&s.circuit)
        .bind(&s.country)
        .bind(&s.city)
        .bind(fp1_time)
        .bind(fp2_time)
        .bind(fp3_time)
        .bind(qualifying_time)
        .bind(sprint_time)
        .bind(race_time)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Fetches the F1 race calendar for a given season, checking SQLite cache first.
pub async fn fetch_calendar(season: u32) -> Result<Vec<F1Session>, SlothError> {
    fetch_calendar_with_pool(season, None).await
}

/// Fetches the F1 race calendar for a given season with an optional database pool for caching.
pub async fn fetch_calendar_with_pool(
    season: u32,
    pool: Option<&SqlitePool>,
) -> Result<Vec<F1Session>, SlothError> {
    if let Some(p) = pool {
        if let Ok(Some(cached)) = get_cached_calendar(p, season).await {
            tracing::info!("Loaded F1 calendar for season {season} from SQLite cache");
            return Ok(cached);
        }
    }

    let client = crate::net::http_client_builder()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap_or_default();

    let jolpica_url = format!("{JOLPICA_BASE_URL}/{season}.json");
    let ergast_url = format!("{ERGAST_BASE_URL}/{season}.json");

    let text_result = match client.get(&jolpica_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
        _ => None,
    };

    let body = match text_result {
        Some(t) => t,
        None => {
            // Fallback to Ergast API
            let resp = client
                .get(&ergast_url)
                .send()
                .await
                .map_err(|e| SlothError::Provider(format!("Failed to reach F1 API: {e}")))?;
            resp.text()
                .await
                .map_err(|e| SlothError::Provider(format!("Failed to read F1 API body: {e}")))?
        }
    };

    let calendar = parse_calendar_json(&body)?;

    if let Some(p) = pool {
        if let Err(e) = save_calendar_to_cache(p, season, &calendar).await {
            tracing::warn!("Failed to cache F1 calendar into SQLite: {e}");
        }
    }

    Ok(calendar)
}

/// Finds the next upcoming or ongoing F1 session slot across the entire calendar.
pub fn next_session(calendar: &[F1Session]) -> Option<(F1Session, F1SessionSlot)> {
    next_session_at(calendar, Utc::now())
}

/// Finds the next session slot relative to a given reference time.
pub fn next_session_at(
    calendar: &[F1Session],
    now: DateTime<Utc>,
) -> Option<(F1Session, F1SessionSlot)> {
    let mut upcoming: Vec<(F1Session, F1SessionSlot)> = Vec::new();
    for session in calendar {
        for slot in &session.sessions {
            // Include if in the future or currently ongoing (within 150 minutes of start)
            if slot.starts_at >= now || slot.starts_at + chrono::Duration::minutes(150) >= now {
                upcoming.push((session.clone(), slot.clone()));
            }
        }
    }
    upcoming.sort_by_key(|(_, slot)| slot.starts_at);
    upcoming.into_iter().next()
}

/// Calculates the duration until an F1 session slot starts.
pub fn time_until(slot: &F1SessionSlot) -> Duration {
    time_until_at(slot, Utc::now())
}

/// Calculates the duration until an F1 session slot starts relative to a reference time.
pub fn time_until_at(slot: &F1SessionSlot, now: DateTime<Utc>) -> Duration {
    if slot.starts_at > now {
        (slot.starts_at - now).to_std().unwrap_or(Duration::ZERO)
    } else {
        Duration::ZERO
    }
}
