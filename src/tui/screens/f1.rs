//! Formula 1 tab screen rendering the next session countdown and season race calendar.

use chrono::Utc;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState};

use crate::providers::f1::calendar::next_session;
use crate::tui::state::AppState;
use crate::tui::theme::Theme;
use crate::tui::widgets::CountdownWidget;

/// Renders the Formula 1 tab screen.
pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Next Session & Countdown Box
            Constraint::Min(8),    // Race Calendar Table
            Constraint::Length(1), // Shortcut Hints Bar
        ])
        .split(area);

    render_next_session_box(f, chunks[0], state, theme);
    render_calendar_table(f, chunks[1], state, theme);
    render_footer(f, chunks[2], theme);
}

/// Renders the top NEXT SESSION hero box with event metadata and countdown timer.
fn render_next_session_box(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_focus))
        .title(" 🏎 Formula 1 — 2026 Season ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.f1_tab.loading {
        let spinner = crate::tui::widgets::loading_spinner(state.tick_count, false);
        let p = Paragraph::new(format!("{spinner} Fetching F1 calendar & session schedule..."))
            .style(Style::default().fg(theme.info))
            .alignment(Alignment::Center);
        f.render_widget(p, inner);
        return;
    }

    let next = next_session(&state.f1_tab.calendar);
    match next {
        Some((session, slot)) => {
            let hero_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Session Header + Circuit Info
                    Constraint::Length(3), // Countdown Timer Widget
                    Constraint::Length(2), // Action Badges / Controls
                ])
                .split(inner);

            // 1. Session & Circuit Lines
            let header_line = Line::from(vec![
                Span::styled("⏰ NEXT: ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{} {} — {}", slot.kind.icon(), session.name.to_uppercase(), slot.kind.label()),
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ),
            ]);

            let loc_str = if !session.city.is_empty() && !session.country.is_empty() {
                format!("{} · {}, {}", session.circuit, session.city, session.country)
            } else if !session.country.is_empty() {
                format!("{} · {}", session.circuit, session.country)
            } else {
                session.circuit.clone()
            };

            let circuit_line = Line::from(vec![
                Span::raw("   "),
                Span::styled(loc_str, Style::default().fg(theme.text_dim)),
            ]);

            f.render_widget(Paragraph::new(vec![header_line, circuit_line]), hero_chunks[0]);

            // 2. Countdown Widget
            let countdown_label = format!("STARTS IN ({} UTC)", slot.starts_at.format("%a, %b %d @ %H:%M"));
            let widget = CountdownWidget::new(slot.starts_at, countdown_label);
            widget.render_with_theme(hero_chunks[1], f.buffer_mut(), theme);

            // 3. Action Buttons
            let buttons_line = Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    " [▶ Watch Live (w)] ",
                    Style::default()
                        .fg(theme.bg)
                        .bg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    " [📋 View Sessions (Enter)] ",
                    Style::default()
                        .fg(theme.text)
                        .bg(theme.bg_elevated),
                ),
            ]);
            f.render_widget(Paragraph::new(vec![Line::raw(""), buttons_line]), hero_chunks[2]);
        }
        None => {
            let msg = if state.f1_tab.calendar.is_empty() {
                "No F1 calendar data loaded. Press 'r' to fetch the 2026 race calendar."
            } else {
                "Season complete or no upcoming sessions found."
            };
            let p = Paragraph::new(msg)
                .style(Style::default().fg(theme.text_muted))
                .alignment(Alignment::Center);
            f.render_widget(p, inner);
        }
    }
}

/// Renders the scrollable Grand Prix race calendar table.
fn render_calendar_table(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(" Race Calendar ")
        .title_style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD));

    if state.f1_tab.calendar.is_empty() {
        let inner = block.inner(area);
        f.render_widget(block, area);
        let p = Paragraph::new("No Grand Prix races scheduled.").style(Style::default().fg(theme.text_dim));
        f.render_widget(p, inner);
        return;
    }

    let now = Utc::now();
    let next_round = next_session(&state.f1_tab.calendar).map(|(s, _)| s.round);

    let header_cells = ["Round", "Grand Prix Event", "Circuit / Location", "Date", "Status"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.bg_elevated))
        .height(1)
        .bottom_margin(1);

    let rows = state.f1_tab.calendar.iter().map(|race| {
        let is_next = Some(race.round) == next_round;
        let is_completed = race
            .sessions
            .last()
            .map(|s| s.starts_at + chrono::Duration::minutes(150) < now)
            .unwrap_or(false);

        let round_str = format!("R{:02}", race.round);
        let event_str = race.name.clone();
        let loc_str = if !race.country.is_empty() {
            format!("{} ({})", race.circuit, race.country)
        } else {
            race.circuit.clone()
        };

        // Format dates
        let date_str = match (race.sessions.first(), race.sessions.last()) {
            (Some(first), Some(last)) => {
                let first_fmt = first.starts_at.format("%b %d").to_string();
                let last_fmt = last.starts_at.format("%b %d").to_string();
                if first_fmt == last_fmt {
                    first_fmt
                } else if first.starts_at.format("%b").to_string() == last.starts_at.format("%b").to_string() {
                    format!("{}-{}", first_fmt, last.starts_at.format("%d"))
                } else {
                    format!("{first_fmt} - {last_fmt}")
                }
            }
            _ => "TBD".to_string(),
        };

        // Status badge
        let (status_text, status_style) = if is_next {
            ("🏁 NEXT RACE", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        } else if is_completed {
            ("✓ Complete", Style::default().fg(theme.text_dim))
        } else {
            ("Upcoming", Style::default().fg(theme.text))
        };

        let row_style = if is_next {
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
        } else if is_completed {
            Style::default().fg(theme.text_dim)
        } else {
            Style::default().fg(theme.text)
        };

        Row::new(vec![
            Cell::from(round_str),
            Cell::from(event_str),
            Cell::from(loc_str),
            Cell::from(date_str),
            Cell::from(status_text).style(status_style),
        ])
        .style(row_style)
    });

    let widths = [
        Constraint::Length(7),  // Round
        Constraint::Min(22),    // Grand Prix Event
        Constraint::Min(22),    // Circuit / Location
        Constraint::Length(14), // Date
        Constraint::Length(14), // Status
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme.selection_bg)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut table_state = TableState::default();
    table_state.select(Some(state.f1_tab.selected_session_idx));

    f.render_stateful_widget(table, area, &mut table_state);
}

/// Renders contextual shortcuts at the bottom of the screen.
fn render_footer(f: &mut Frame, area: Rect, theme: &Theme) {
    let hints = Line::from(vec![
        Span::styled(" j/k / ↑↓", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" Navigate  ", Style::default().fg(theme.text_dim)),
        Span::styled("Enter", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" Play/Sessions  ", Style::default().fg(theme.text_dim)),
        Span::styled("w", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" Watch Live  ", Style::default().fg(theme.text_dim)),
        Span::styled("r", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" Refresh  ", Style::default().fg(theme.text_dim)),
        Span::styled("?", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" Help", Style::default().fg(theme.text_dim)),
    ]);

    let p = Paragraph::new(hints).style(Style::default().bg(theme.bg));
    f.render_widget(p, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use crate::providers::f1::calendar::{F1Session, F1SessionKind, F1SessionSlot};

    #[test]
    fn test_f1_screen_renders_without_panic() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState::default();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                render(f, f.area(), &state, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_f1_screen_with_sample_data() {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = AppState::default();
        let theme = Theme::default();

        let future_time = Utc::now() + chrono::Duration::days(3);
        state.f1_tab.calendar = vec![F1Session {
            round: 1,
            name: "Bahrain Grand Prix".into(),
            circuit: "Bahrain International Circuit".into(),
            country: "Bahrain".into(),
            city: "Sakhir".into(),
            sessions: vec![
                F1SessionSlot {
                    kind: F1SessionKind::FreePractice1,
                    starts_at: future_time,
                    stream_url: None,
                },
                F1SessionSlot {
                    kind: F1SessionKind::Race,
                    starts_at: future_time + chrono::Duration::days(2),
                    stream_url: None,
                },
            ],
        }];

        terminal
            .draw(|f| {
                render(f, f.area(), &state, &theme);
            })
            .unwrap();
    }
}
