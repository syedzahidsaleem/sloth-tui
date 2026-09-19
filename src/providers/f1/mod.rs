//! Formula 1 streaming providers and calendar module.

pub mod calendar;
pub mod streams;

pub use calendar::{
    F1Session, F1SessionKind, F1SessionSlot, fetch_calendar, fetch_calendar_with_pool,
    next_session, parse_calendar_json, time_until,
};
pub use streams::{F1StreamsProvider, fetch_f1_streams, is_f1_channel};
