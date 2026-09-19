//! Sports tab screen rendering with a three-column layout.
//!
//! Left (20%): Sports category list
//! Middle (45%): Match list filtered by selected sport
//! Right (35%): Stream sources and quality details for selected match

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};

use crate::tui::state::{LiveMatch, MatchStream, SportsColumnFocus, SportsTabState};
use crate::tui::theme::Theme;

/// Renders the complete Sports tab screen view.
pub fn render(frame: &mut Frame, area: Rect, state: &SportsTabState, theme: &Theme) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    // Split into main 3-column panel area and bottom navigation bar
    let vertical_layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let panels_area = vertical_layout[0];
    let bottom_area = vertical_layout[1];

    // Three-column layout: Left (20%), Middle (45%), Right (35%)
    let columns = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(45),
        Constraint::Percentage(35),
    ])
    .split(panels_area);

    let sports_col_area = columns[0];
    let matches_col_area = columns[1];
    let streams_col_area = columns[2];

    render_sports_column(frame, sports_col_area, state, theme);
    render_matches_column(frame, matches_col_area, state, theme);
    render_streams_column(frame, streams_col_area, state, theme);
    render_bottom_bar(frame, bottom_area, theme);
}

/// Renders the left sports category column (20% width).
fn render_sports_column(
    frame: &mut Frame,
    area: Rect,
    state: &SportsTabState,
    theme: &Theme,
) {
    let is_focused = state.focus == SportsColumnFocus::Sports;
    let border_style = if is_focused {
        theme.border_focus
    } else {
        theme.border
    };
    let title_style = if is_focused {
        theme.accent
    } else {
        theme.title
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(" Sports ", title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines = Vec::new();
    let visible_rows = inner.height as usize;
    let total_sports = state.sports.len();

    let scroll_offset = if state.selected_sport_idx >= visible_rows {
        state.selected_sport_idx - visible_rows + 1
    } else {
        0
    };

    for (idx, sport) in state.sports.iter().enumerate().skip(scroll_offset).take(visible_rows) {
        let is_selected = idx == state.selected_sport_idx;
        let prefix = if is_selected { " > " } else { "   " };
        let style = if is_selected {
            if is_focused {
                theme.highlight.add_modifier(Modifier::BOLD)
            } else {
                theme.accent
            }
        } else {
            theme.text
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(sport, style),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    if total_sports > visible_rows {
        let mut scrollbar_state = ScrollbarState::new(total_sports)
            .position(state.selected_sport_idx);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            area,
            &mut scrollbar_state,
        );
    }
}

/// Renders the middle live & upcoming matches column (45% width).
fn render_matches_column(
    frame: &mut Frame,
    area: Rect,
    state: &SportsTabState,
    theme: &Theme,
) {
    let is_focused = state.focus == SportsColumnFocus::Matches;
    let border_style = if is_focused {
        theme.border_focus
    } else {
        theme.border
    };
    let title_style = if is_focused {
        theme.accent
    } else {
        theme.title
    };

    let sport_name = state.selected_sport().unwrap_or("Live Sports");
    let title = format!(" Matches — {sport_name} ");

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(title, title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    if state.matches.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("No {sport_name} matches loaded"),
                theme.text_dim,
            )),
            Line::from(Span::styled(
                "Press Enter on a sport to load matches",
                theme.text_dim,
            )),
            Line::from(Span::styled("Press 'r' to refresh live data", theme.text_dim)),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner);
        return;
    }

    let item_height = 2usize;
    let visible_items = (inner.height as usize / item_height).max(1);
    let total_matches = state.matches.len();

    let scroll_offset = if state.selected_match_idx >= visible_items {
        state.selected_match_idx - visible_items + 1
    } else {
        0
    };

    let mut lines = Vec::new();
    for (idx, m) in state.matches.iter().enumerate().skip(scroll_offset).take(visible_items) {
        let is_selected = idx == state.selected_match_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let status_icon = if m.is_live() {
            Span::styled("● ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("⏰ ", Style::default().fg(Color::Yellow))
        };

        let match_title = format_match_title(m);
        let title_style = if is_selected {
            if is_focused {
                theme.highlight.add_modifier(Modifier::BOLD)
            } else {
                theme.accent.add_modifier(Modifier::BOLD)
            }
        } else {
            theme.text
        };

        // Line 1: cursor + status icon + title
        lines.push(Line::from(vec![
            Span::styled(prefix, title_style),
            status_icon,
            Span::styled(match_title, title_style),
        ]));

        // Line 2: competition name + time badge
        let comp = m.competition.as_deref().unwrap_or("Competition");
        let badge_text = m.time_badge();
        let badge_style = if m.is_live() {
            Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
        } else {
            theme.text_dim
        };

        lines.push(Line::from(vec![
            Span::raw("     "),
            Span::styled(comp, theme.text_dim),
            Span::styled("  ·  ", theme.text_dim),
            Span::styled(badge_text, badge_style),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    if total_matches > visible_items {
        let mut scrollbar_state = ScrollbarState::new(total_matches)
            .position(state.selected_match_idx);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            area,
            &mut scrollbar_state,
        );
    }
}

/// Formats the primary match title, preferring Home vs Away if teams are specified.
fn format_match_title(m: &LiveMatch) -> String {
    if let Some((home, away)) = &m.teams {
        format!("{home} vs {away}")
    } else {
        m.title.clone()
    }
}

/// Renders the right stream sources and details column (35% width).
fn render_streams_column(
    frame: &mut Frame,
    area: Rect,
    state: &SportsTabState,
    theme: &Theme,
) {
    let is_focused = state.focus == SportsColumnFocus::Streams;
    let border_style = if is_focused {
        theme.border_focus
    } else {
        theme.border
    };
    let title_style = if is_focused {
        theme.accent
    } else {
        theme.title
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(" Streams ", title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    if state.streams.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No stream sources selected",
                theme.text_dim,
            )),
            Line::from(Span::styled(
                "Select a match and press Enter",
                theme.text_dim,
            )),
            Line::from(Span::styled(
                "to load available live streams",
                theme.text_dim,
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner);
        return;
    }

    // Split inner area into top stream list and bottom stream details card
    let stream_layout = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(6),
    ])
    .split(inner);

    let list_area = stream_layout[0];
    let details_area = stream_layout[1];

    // Render Stream list
    let visible_rows = list_area.height as usize;
    let total_streams = state.streams.len();
    let scroll_offset = if state.selected_stream_idx >= visible_rows {
        state.selected_stream_idx - visible_rows + 1
    } else {
        0
    };

    let mut lines = Vec::new();
    for (idx, stream) in state.streams.iter().enumerate().skip(scroll_offset).take(visible_rows) {
        let is_selected = idx == state.selected_stream_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let (badge, badge_style) = format_stream_quality_badge(stream, theme);
        let lang = stream.language.as_deref().unwrap_or("English");

        let text_style = if is_selected {
            if is_focused {
                theme.highlight.add_modifier(Modifier::BOLD)
            } else {
                theme.accent.add_modifier(Modifier::BOLD)
            }
        } else {
            theme.text
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, text_style),
            Span::styled(format!("[{badge}] "), badge_style),
            Span::styled(lang, text_style),
        ]));
    }

    let list_paragraph = Paragraph::new(lines);
    frame.render_widget(list_paragraph, list_area);

    if total_streams > visible_rows {
        let mut scrollbar_state = ScrollbarState::new(total_streams)
            .position(state.selected_stream_idx);
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼")),
            list_area,
            &mut scrollbar_state,
        );
    }

    // Render bottom stream details card
    if let Some(selected_stream) = state.selected_stream() {
        render_stream_details_card(frame, details_area, selected_stream, theme);
    }
}

/// Determines the display badge and color style for a stream source.
fn format_stream_quality_badge<'a>(stream: &'a MatchStream, theme: &'a Theme) -> (&'static str, Style) {
    if stream.hd_url.is_some() {
        ("HD", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    } else if stream.sd_url.is_some() {
        ("SD", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        ("Embed", theme.text_dim)
    }
}

/// Renders details, quality bar, and playback action button for the selected stream.
fn render_stream_details_card(
    frame: &mut Frame,
    area: Rect,
    stream: &MatchStream,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(theme.border);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let quality_tier = if stream.hd_url.is_some() {
        "1080p ✓ Live"
    } else if stream.sd_url.is_some() {
        "720p ✓ Live"
    } else {
        "Web Player"
    };

    let quality_bar = if stream.hd_url.is_some() {
        "████████░░"
    } else if stream.sd_url.is_some() {
        "██████░░░░"
    } else {
        "████░░░░░░"
    };

    let details_lines = vec![
        Line::from(vec![
            Span::styled(" Quality: ", theme.text_dim),
            Span::styled(quality_tier, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(quality_bar, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled(" Source:  ", theme.text_dim),
            Span::styled(&stream.id, theme.text),
        ]),
        Line::from(vec![
            Span::styled(" [▶ Enter: Play Stream] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let details_paragraph = Paragraph::new(details_lines)
        .wrap(Wrap { trim: true });
    frame.render_widget(details_paragraph, inner);
}

/// Renders the bottom shortcut hints and navigation bar.
fn render_bottom_bar(frame: &mut Frame, area: Rect, theme: &Theme) {
    let shortcuts = Line::from(vec![
        Span::styled(" h/l ", theme.shortcut),
        Span::styled("Navigate Columns  ", theme.text_dim),
        Span::styled(" j/k ", theme.shortcut),
        Span::styled("Scroll List  ", theme.text_dim),
        Span::styled(" Enter ", theme.shortcut),
        Span::styled("Confirm / Play  ", theme.text_dim),
        Span::styled(" r ", theme.shortcut),
        Span::styled("Refresh Live  ", theme.text_dim),
        Span::styled(" ? ", theme.shortcut),
        Span::styled("Help", theme.text_dim),
    ]);

    let bar = Paragraph::new(shortcuts).alignment(Alignment::Center);
    frame.render_widget(bar, area);
}
