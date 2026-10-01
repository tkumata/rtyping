use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::presentation::ui::app::App;
use crate::usecase::history_stats::{HistoryStats, MistakeCount};

use super::common::centered_rect;

pub fn render_stats(frame: &mut Frame, app: &App) {
    let area = centered_rect(78, 70, frame.area());
    let stats = app.history_stats();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Stats ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if stats.count == 0 {
        render_empty_stats(frame, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(1),
        ])
        .split(inner);
    let [kpi_area, trend_area, misses_area, footer_area] = &*chunks else {
        return;
    };

    frame.render_widget(Paragraph::new(kpi_lines(&stats)), *kpi_area);
    frame.render_widget(Paragraph::new(recent_wpm_lines(&stats)), *trend_area);
    frame.render_widget(Paragraph::new(miss_lines(&stats)), *misses_area);
    frame.render_widget(
        Paragraph::new("Enter / Esc: Back").alignment(Alignment::Center),
        *footer_area,
    );
}

fn render_empty_stats(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
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
    ]
}

fn recent_wpm_lines(stats: &HistoryStats) -> Vec<Line<'static>> {
    vec![
        section_title("Recent 10 WPM"),
        Line::from(format_recent_values(stats)),
        Line::from(recent_graph(&stats.recent_wpm)),
    ]
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

fn format_recent_values(stats: &HistoryStats) -> String {
    if stats.recent_wpm.is_empty() {
        return "-".into();
    }

    stats
        .recent_wpm
        .iter()
        .map(|wpm| format!("{wpm:.0}"))
        .collect::<Vec<_>>()
        .join("  ")
}

fn recent_graph(values: &[f64]) -> String {
    if values.is_empty() {
        return "-".into();
    }

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        return vec!["▄"; values.len()].join("  ");
    }

    values
        .iter()
        .map(|value| {
            let ratio = ((*value - min) / (max - min)).clamp(0.0, 1.0);
            graph_block(ratio)
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn graph_block(ratio: f64) -> &'static str {
    if ratio < 0.125 {
        "▁"
    } else if ratio < 0.250 {
        "▂"
    } else if ratio < 0.375 {
        "▃"
    } else if ratio < 0.500 {
        "▄"
    } else if ratio < 0.625 {
        "▅"
    } else if ratio < 0.750 {
        "▆"
    } else if ratio < 0.875 {
        "▇"
    } else {
        "█"
    }
}

fn format_mistake_bar(mistake: &MistakeCount, max_count: usize) -> String {
    const BAR_WIDTH: usize = 24;

    let width = ((mistake.count * BAR_WIDTH).div_ceil(max_count)).max(1);
    format!(
        "{} {:>3}  {}",
        mistake.character,
        mistake.count,
        "■".repeat(width)
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
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
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
        assert!(text.contains("Recent 10 WPM"));
        assert!(text.contains("40  60  72"));
        assert!(text.contains("Frequent Misses"));
        assert!(text.contains("e  24  ■■■■■■■■■■■■■■■■■■■■■■■■ "));
        assert!(text.contains("t  12  ■■■■■■■■■■■■ "));
        assert!(text.contains("a   1  ■ "));
        assert!(text.contains("Enter / Esc: Back"));
    }

    #[test]
    fn recent_graph_uses_same_height_for_flat_values() {
        assert_eq!(recent_graph(&[50.0, 50.0, 50.0]), "▄  ▄  ▄");
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
