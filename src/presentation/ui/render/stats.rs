use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::presentation::ui::app::App;
use crate::usecase::history_stats::{HistoryStats, MistakeCount};

use super::common::centered_rect;

const BAR_GLYPH: char = '■';

pub fn render_stats(frame: &mut Frame, app: &App) {
    let area = centered_rect(78, 100, frame.area());
    let stats = app.history_stats();
    if stats.count == 0 {
        render_empty_stats(frame, centered_rect(100, 70, area));
        return;
    }

    let kpi = kpi_lines(&stats);
    let misses = miss_lines(&stats);
    let kpi_height = u16::try_from(kpi.len()).unwrap_or(u16::MAX);
    let misses_height = u16::try_from(misses.len()).unwrap_or(u16::MAX);
    let trend_height = u16::try_from(stats.recent_wpm.len() + 1).unwrap_or(u16::MAX);
    let fixed_height = kpi_height + misses_height + 1;
    let trend_height = trend_height.min(area.height.saturating_sub(fixed_height));
    let [area] = Layout::vertical([Constraint::Length(fixed_height + trend_height)])
        .flex(Flex::Center)
        .areas(area);
    let [kpi_area, trend_area, misses_area, footer_area] = Layout::vertical([
        Constraint::Max(kpi_height),
        Constraint::Length(trend_height),
        Constraint::Max(misses_height),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(Paragraph::new(kpi), kpi_area);
    frame.render_widget(
        Paragraph::new(recent_wpm_lines(&stats, trend_area)),
        trend_area,
    );
    frame.render_widget(Paragraph::new(misses), misses_area);
    frame.render_widget(
        Paragraph::new("Enter / Esc: Back").alignment(Alignment::Center),
        footer_area,
    );
}

fn render_empty_stats(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(vec![
            section_title("Stats"),
            Line::from(Span::styled(
                "No timed history yet",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Finish a timed session to build your stats."),
            Line::from(""),
            Line::from("Enter / Esc: Back"),
        ])
        .alignment(Alignment::Center),
        area,
    );
}

fn kpi_lines(stats: &HistoryStats) -> Vec<Line<'static>> {
    vec![
        section_title("Stats"),
        section_title("Performance"),
        Line::from(""),
        Line::from(vec![
            label_span("Best WPM        "),
            label_span("Avg WPM         "),
            label_span("Avg Accuracy    "),
            label_span("Runs"),
        ]),
        Line::from(vec![
            value_span(format!("{:<16}", format_optional(stats.best_wpm))),
            value_span(format!("{:<16}", format_optional(stats.average_wpm))),
            value_span(format!("{:<16}", format_accuracy(stats.average_accuracy))),
            value_span(stats.count.to_string()),
        ]),
        Line::default(),
    ]
}

fn recent_wpm_lines(stats: &HistoryStats, area: Rect) -> Vec<Line<'static>> {
    let mut lines = vec![section_title("Recent 10 WPM")];
    let visible_count = usize::from(area.height.saturating_sub(1));
    let visible_values = stats
        .recent_wpm
        .iter()
        .skip(stats.recent_wpm.len().saturating_sub(visible_count));
    let labels: Vec<_> = visible_values
        .clone()
        .map(|&wpm| recent_wpm_label(wpm))
        .collect();
    let label_width = labels.iter().map(String::len).max().unwrap_or(0);
    let bar_width = area
        .width
        .saturating_sub(u16::try_from(label_width + 2).unwrap_or(u16::MAX));

    lines.extend(visible_values.zip(labels).map(|(wpm, label)| {
        let squares = (wpm / 10.0).round();
        let bar: String = (1..=bar_width)
            .take_while(|index| f64::from(*index) <= squares)
            .map(|_| BAR_GLYPH)
            .collect();
        Line::from(format!("{label:>label_width$}  {bar}"))
    }));
    lines
}

fn recent_wpm_label(wpm: f64) -> String {
    let whole_wpm = wpm.round_ties_even();
    if (whole_wpm / 10.0).round() > (wpm / 10.0).round() {
        // Keep the fraction when rounding would put the label on the next bar threshold.
        wpm.to_string()
    } else if whole_wpm.abs() >= 1_000_000.0 {
        format!("{whole_wpm:e}")
    } else {
        format!("{whole_wpm:.0}")
    }
}

fn miss_lines(stats: &HistoryStats) -> Vec<Line<'static>> {
    let mut lines = vec![section_title("Frequent Misses")];
    if stats.frequent_mistakes.is_empty() {
        lines.push(Line::from("-"));
        return lines;
    }

    let max_count = stats
        .frequent_mistakes
        .iter()
        .map(|mistake| mistake.count)
        .max()
        .unwrap_or(1);

    lines.extend(
        stats
            .frequent_mistakes
            .iter()
            .map(|mistake| Line::from(format_mistake_bar(mistake, max_count))),
    );
    lines
}

fn section_title(title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn label_span(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::DarkGray))
}

fn value_span(value: String) -> Span<'static> {
    Span::styled(
        value,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )
}

fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "-".into(), |value| format!("{value:.1}"))
}

fn format_accuracy(value: Option<f64>) -> String {
    value.map_or_else(|| "-".into(), |value| format!("{value:.1}%"))
}

fn format_mistake_bar(mistake: &MistakeCount, max_count: usize) -> String {
    const BAR_WIDTH: usize = 24;

    let width = ((mistake.count * BAR_WIDTH).div_ceil(max_count)).max(1);
    format!(
        "{} {:>3}  {}",
        mistake.character,
        mistake.count,
        BAR_GLYPH.to_string().repeat(width)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::domain::history::{HistoryEntry, HistoryMode};
    use ratatui::{Terminal, backend::TestBackend};
    use std::convert::Infallible;

    fn value_from_infallible<T>(result: Result<T, Infallible>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => match err {},
        }
    }

    fn render_to_text(app: &App) -> String {
        let backend = TestBackend::new(100, 30);
        let mut terminal = value_from_infallible(Terminal::new(backend));
        value_from_infallible(terminal.draw(|frame| render_stats(frame, app)));

        terminal
            .backend()
            .buffer()
            .content()
            .chunks(100)
            .map(|row| {
                row.iter()
                    .map(ratatui::buffer::Cell::symbol)
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn stats_render_includes_empty_history_and_return_hint() {
        let app = App::new(AppConfig::default());

        let text = render_to_text(&app);

        assert!(text.contains("Stats"));
        assert!(text.contains("No timed history yet"));
        assert!(text.contains("Finish a timed session to build your stats."));
        assert!(text.contains("Enter / Esc: Back"));
    }

    #[test]
    fn stats_render_includes_dashboard_sections() {
        let mut app = App::new(AppConfig::default());
        app.set_history_entries(vec![
            history_entry(999.0, 100.0, vec![]),
            history_entry(4.0, 100.0, vec![]),
            history_entry(5.0, 100.0, vec![]),
            history_entry(10.0, 100.0, vec![]),
            history_entry(14.99, 100.0, vec![]),
            history_entry(15.0, 100.0, vec![]),
            history_entry(21.0, 100.0, vec![]),
            history_entry(25.0, 100.0, vec![]),
            history_entry(40.0, 90.0, vec!['e'; 24]),
            history_entry(60.0, 95.0, vec!['t'; 12]),
            history_entry(72.0, 100.0, vec!['a']),
        ]);

        let text = render_to_text(&app);

        assert!(text.contains("Performance"));
        assert!(text.contains("Best WPM"));
        assert!(text.contains("Avg WPM"));
        assert!(text.contains("Avg Accuracy"));
        assert!(text.contains("Runs"));
        assert!(!text.contains('┌'));
        let mut trend = text
            .lines()
            .skip_while(|row| !row.contains("Recent 10 WPM"));
        let heading = trend.next().unwrap_or_default();
        assert_eq!(heading.trim(), "Recent 10 WPM");
        let margin = heading.len() - heading.trim_start().len();
        let rows: Vec<_> = trend
            .take(10)
            .map(|row| row.split_at(margin).1.trim_end())
            .collect();
        assert_eq!(
            rows,
            [
                "    4",
                "    5  ■",
                "   10  ■",
                "14.99  ■",
                "   15  ■■",
                "   21  ■■",
                "   25  ■■■",
                "   40  ■■■■",
                "   60  ■■■■■■",
                "   72  ■■■■■■■",
            ]
        );
        assert!(text.contains("Frequent Misses"));
        assert!(text.contains("e  24  ■■■■■■■■■■■■■■■■■■■■■■■■ "));
        assert!(text.contains("t  12  ■■■■■■■■■■■■ "));
        assert!(text.contains("a   1  ■ "));
        assert!(text.contains("Enter / Esc: Back"));
    }

    fn history_entry(wpm: f64, accuracy: f64, missed_chars: Vec<char>) -> HistoryEntry {
        HistoryEntry {
            wpm,
            accuracy,
            miss_count: missed_chars.len(),
            elapsed_seconds: 60,
            generation_source: "Local".into(),
            mode: HistoryMode::Timed,
            missed_chars,
        }
    }
}
