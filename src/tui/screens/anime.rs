//! Anime tab screen rendering and two-panel layout.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
};

use crate::providers::models::{EpisodeRef, Media};
use crate::tui::state::{AnimePanelFocus, AnimeTabState};
use crate::tui::theme::Theme;

/// Renders the complete Anime tab screen view.
pub fn render(frame: &mut Frame, area: Rect, state: &AnimeTabState, theme: &Theme) {
    if area.width < 10 || area.height < 5 {
        return;
    }

    // Split into main panel area and bottom control bar
    let vertical_layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let panels_area = vertical_layout[0];
    let bottom_area = vertical_layout[1];

    // Two-panel layout: Left (35% results) and Right (65% details/episodes)
    let panel_columns = Layout::horizontal([
        Constraint::Percentage(35),
        Constraint::Percentage(65),
    ])
    .split(panels_area);

    let left_area = panel_columns[0];
    let right_area = panel_columns[1];

    render_left_panel(frame, left_area, state, theme);
    render_right_panel(frame, right_area, state, theme);
    render_bottom_bar(frame, bottom_area, state, theme);
}

/// Renders the left search results panel (35% width).
fn render_left_panel(frame: &mut Frame, area: Rect, state: &AnimeTabState, theme: &Theme) {
    let is_focused = state.focus == AnimePanelFocus::Results;
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
        .title(Span::styled(" Search Results ", title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.results.is_empty() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Search for anime above",
                theme.text_dim,
            )),
            Line::from(Span::styled(
                "Type '/' or click search bar",
                theme.text_dim,
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner);
        return;
    }

    let item_height = 2;
    let visible_items = (inner.height as usize / item_height).max(1);
    let total_items = state.results.len();

    // Scroll window calculation
    let scroll_offset = if state.selected_idx >= visible_items {
        state.selected_idx - visible_items + 1
    } else {
        0
    };

    let mut lines = Vec::new();
    let end_idx = (scroll_offset + visible_items).min(total_items);

    for idx in scroll_offset..end_idx {
        let anime = &state.results[idx];
        let is_selected = idx == state.selected_idx;

        let title_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(theme.accent.fg.unwrap_or(Color::Cyan))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.text.fg.unwrap_or(Color::White))
                .add_modifier(Modifier::BOLD)
        };

        // Line 1: Title + prefix indicator
        let prefix = if is_selected { "> " } else { "  " };
        let mut title_spans = vec![
            Span::styled(prefix, if is_selected { title_style } else { theme.accent }),
            Span::styled(&anime.title, title_style),
        ];

        // Line 2: Rating star + sub/dub badges
        let rating_str = anime
            .rating
            .map(|r| format!("★ {r:.1}"))
            .unwrap_or_else(|| "★ N/A".to_string());

        let ep_count = anime.episodes_count.unwrap_or(0);
        let badge_text = format!(" [Sub: {ep_count} | Dub: {ep_count}]");

        let meta_spans = vec![
            Span::raw("    "),
            Span::styled(rating_str, theme.rating),
            Span::styled(badge_text, theme.lavender),
        ];

        lines.push(Line::from(title_spans));
        lines.push(Line::from(meta_spans));
    }

    let list_paragraph = Paragraph::new(lines);
    frame.render_widget(list_paragraph, inner);

    // Render scrollbar if results exceed visible area
    if total_items > visible_items {
        let mut scrollbar_state =
            ScrollbarState::new(total_items.saturating_sub(visible_items))
                .position(scroll_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

/// Renders the right panel (65% width) with episode list and details.
fn render_right_panel(frame: &mut Frame, area: Rect, state: &AnimeTabState, theme: &Theme) {
    let is_focused = state.focus == AnimePanelFocus::Episodes;
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
        .title(Span::styled(" Episode List / Details ", title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let selected_anime = state.selected_anime();
    if selected_anime.is_none() {
        let placeholder = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Search for anime above",
                theme.text_dim,
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(placeholder, inner);
        return;
    }

    let anime = selected_anime.unwrap();

    // Split inner area into top metadata and bottom episode list
    let content_split = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    let meta_area = content_split[0];
    let sep_area = content_split[1];
    let episodes_area = content_split[2];

    // Metadata header
    let rating_str = anime
        .rating
        .map(|r| format!("★ {r:.1}"))
        .unwrap_or_else(|| "★ N/A".to_string());

    let genres_str = if !anime.genres.is_empty() {
        anime.genres.join(", ")
    } else {
        "Action, Adventure".to_string()
    };

    let meta_lines = vec![
        Line::from(vec![
            Span::styled(
                anime.title.to_uppercase(),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(theme.accent.fg.unwrap_or(Color::Cyan)),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ⛩ AniList: ", theme.lavender),
            Span::styled("Completed  ", theme.teal),
            Span::styled(format!("{rating_str}  "), theme.rating),
            Span::styled(format!("•  Genres: {genres_str}"), theme.text_dim),
        ]),
        Line::from(vec![
            Span::styled(
                format!(
                    "  {}",
                    anime
                        .overview
                        .as_deref()
                        .unwrap_or("A legendary anime series following epic adventures.")
                ),
                theme.text_dim,
            ),
        ]),
    ];

    let meta_p = Paragraph::new(meta_lines).wrap(Wrap { trim: true });
    frame.render_widget(meta_p, meta_area);

    // Separator line
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(sep_area.width as usize),
        theme.border,
    )));
    frame.render_widget(sep, sep_area);

    // Episode list rendering
    let fallback_count = anime.episodes_count.unwrap_or(0);
    let total_episodes = if !state.episodes.is_empty() {
        state.episodes.len()
    } else {
        fallback_count as usize
    };

    if total_episodes == 0 {
        let no_ep = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No episodes found for this series",
                theme.text_dim,
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(no_ep, episodes_area);
        return;
    }

    let visible_episodes = episodes_area.height as usize;
    let selected_ep_idx = state.selected_episode_idx;

    let ep_scroll_offset = if selected_ep_idx >= visible_episodes {
        selected_ep_idx - visible_episodes + 1
    } else {
        0
    };

    let ep_end_idx = (ep_scroll_offset + visible_episodes).min(total_episodes);
    let mut ep_lines = Vec::new();

    for idx in ep_scroll_offset..ep_end_idx {
        let is_current_select = idx == selected_ep_idx && is_focused;
        let is_resume = idx == 0; // First or active resume episode
        let is_watched = idx > 0 && idx < 3; // Mock watched state for earlier episodes

        let ep_num = idx + 1;
        let ep_title = if !state.episodes.is_empty() && idx < state.episodes.len() {
            state.episodes[idx]
                .title
                .clone()
                .unwrap_or_else(|| format!("Episode {ep_num}"))
        } else {
            format!("Episode {ep_num}")
        };

        let label = format!("S01E{ep_num:02} — {ep_title}");

        let prefix_span = if is_resume {
            Span::styled("  ▶ ", theme.accent)
        } else if is_current_select {
            Span::styled("  > ", theme.accent)
        } else {
            Span::raw("    ")
        };

        let content_style = if is_current_select {
            Style::default()
                .fg(Color::Black)
                .bg(theme.accent.fg.unwrap_or(Color::Cyan))
                .add_modifier(Modifier::BOLD)
        } else if is_watched {
            theme.text_dim
        } else {
            theme.text
        };

        let mut spans = vec![prefix_span, Span::styled(label, content_style)];

        if is_watched {
            spans.push(Span::styled(" ✓", theme.text_dim));
        }

        ep_lines.push(Line::from(spans));
    }

    let ep_paragraph = Paragraph::new(ep_lines);
    frame.render_widget(ep_paragraph, episodes_area);

    // Scrollbar for episode list
    if total_episodes > visible_episodes {
        let mut scrollbar_state =
            ScrollbarState::new(total_episodes.saturating_sub(visible_episodes))
                .position(ep_scroll_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        frame.render_stateful_widget(scrollbar, episodes_area, &mut scrollbar_state);
    }
}

/// Renders the bottom control and status bar (2 rows).
fn render_bottom_bar(frame: &mut Frame, area: Rect, state: &AnimeTabState, theme: &Theme) {
    let sub_active = !state.is_dub;
    let dub_active = state.is_dub;

    let sub_style = if sub_active {
        Style::default()
            .fg(Color::Black)
            .bg(theme.accent.fg.unwrap_or(Color::Cyan))
            .add_modifier(Modifier::BOLD)
    } else {
        theme.text_dim
    };

    let dub_style = if dub_active {
        Style::default()
            .fg(Color::Black)
            .bg(theme.accent.fg.unwrap_or(Color::Cyan))
            .add_modifier(Modifier::BOLD)
    } else {
        theme.text_dim
    };

    let row1 = Line::from(vec![
        Span::raw(" "),
        Span::styled("[Sub]", sub_style),
        Span::raw(" "),
        Span::styled("[Dub]", dub_style),
        Span::styled("  (Press 't' to toggle)", theme.text_dim),
        Span::styled("   ⛩ AniList: Synced", theme.teal),
    ]);

    let row2 = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "[j/k: Navigate  l: Episodes  h: Results  Enter: Select / Play  /: Search]",
            theme.shortcut,
        ),
    ]);

    let p = Paragraph::new(vec![row1, row2]);
    frame.render_widget(p, area);
}
