//! Countdown widget displaying time remaining until a target date/time.

use chrono::{DateTime, Utc};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::theme::Theme;

/// A widget that prominently displays a formatted countdown timer.
#[derive(Debug, Clone)]
pub struct CountdownWidget {
    /// Target date and time in UTC.
    pub target: DateTime<Utc>,
    /// Header or category label above the countdown.
    pub label: String,
}

impl CountdownWidget {
    /// Creates a new countdown widget for the given target and label.
    pub fn new(target: DateTime<Utc>, label: impl Into<String>) -> Self {
        Self {
            target,
            label: label.into(),
        }
    }

    /// Formats the duration between `now` and `target` as `"02d 14h 23m 07s"` or `"● LIVE NOW"`.
    pub fn format_duration(target: DateTime<Utc>, now: DateTime<Utc>) -> String {
        if target > now {
            let total_secs = (target - now).num_seconds().max(0);
            let days = total_secs / 86400;
            let hours = (total_secs % 86400) / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;
            format!("{days:02}d {hours:02}h {minutes:02}m {seconds:02}s")
        } else {
            let elapsed_secs = (now - target).num_seconds();
            if elapsed_secs <= 150 * 60 {
                "● LIVE NOW".to_string()
            } else {
                "00d 00h 00m 00s (Complete)".to_string()
            }
        }
    }

    /// Renders the countdown widget using theme colors.
    pub fn render_with_theme(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        if area.width < 10 || area.height < 1 {
            return;
        }

        let now = Utc::now();
        let formatted = Self::format_duration(self.target, now);

        let style = if formatted.starts_with('●') {
            theme.error.add_modifier(Modifier::BOLD)
        } else if self.target - now < chrono::Duration::hours(24) {
            theme.rating.add_modifier(Modifier::BOLD)
        } else {
            theme.accent.add_modifier(Modifier::BOLD)
        };

        if area.height == 1 {
            let p = Paragraph::new(formatted).style(style).alignment(Alignment::Center);
            p.render(area, buf);
            return;
        }

        let lines = if area.height >= 3 && !self.label.is_empty() {
            vec![
                ratatui::text::Line::from(ratatui::text::Span::styled(&self.label, theme.text_dim)),
                ratatui::text::Line::from(ratatui::text::Span::styled(formatted, style)),
                ratatui::text::Line::from(ratatui::text::Span::styled(
                    "████████████████████░░░░░░░░░░░",
                    theme.accent,
                )),
            ]
        } else if !self.label.is_empty() {
            vec![
                ratatui::text::Line::from(ratatui::text::Span::styled(&self.label, theme.text_dim)),
                ratatui::text::Line::from(ratatui::text::Span::styled(formatted, style)),
            ]
        } else {
            vec![ratatui::text::Line::from(ratatui::text::Span::styled(formatted, style))]
        };

        let p = Paragraph::new(lines).alignment(Alignment::Center);
        p.render(area, buf);
    }
}

impl Widget for CountdownWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = Theme::default();
        self.render_with_theme(area, buf, &theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countdown_format_future() {
        let now = DateTime::parse_from_rfc3339("2026-03-01T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let target = DateTime::parse_from_rfc3339("2026-03-03T12:23:07Z")
            .unwrap()
            .with_timezone(&Utc);

        let formatted = CountdownWidget::format_duration(target, now);
        assert_eq!(formatted, "02d 02h 23m 07s");
    }

    #[test]
    fn test_countdown_format_live() {
        let target = DateTime::parse_from_rfc3339("2026-03-01T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let now = DateTime::parse_from_rfc3339("2026-03-01T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let formatted = CountdownWidget::format_duration(target, now);
        assert_eq!(formatted, "● LIVE NOW");
    }
}
