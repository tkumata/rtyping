use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::presentation::ui::app::App;

use super::common::centered_rect;
use super::history_summary::history_summary_lines;
use super::wpm_graph;

pub fn render_result(frame: &mut Frame, app: &App) {
    if app.is_rhythm_result() {
        render_rhythm_result(frame, app);
        return;
    }

    let area = centered_rect(70, 60, frame.area());
    let elapsed = app.timer().max(1);
    let score = app.result_wpm();
    let accuracy = app.accuracy();
    let [metrics_area, stats_area, graph_area, footer_area] =
        split_result_area(area, app.status_message().is_some());
    let lines = vec![
        Line::from("Typing Finished"),
        Line::from(""),
        Line::from(format!("Typed: {}", app.typed_count())),
        Line::from(format!("Misses: {}", app.incorrects())),
        Line::from(format!("Accuracy: {accuracy:.1}%")),
        Line::from(format!("Time: {elapsed} sec")),
        Line::from(format!("WPM: {score:.1}")),
        Line::from(format!("Source: {}", app.generation_source().label())),
        Line::from(if app.is_practice_mode() {
            "Mode: Practice"
        } else {
            "Mode: Timed"
        }),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Result ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Center),
        metrics_area,
    );

    let stats = app.history_stats();
    frame.render_widget(
        Paragraph::new(history_summary_lines(&stats))
            .block(Block::default().borders(Borders::ALL).title(" History "))
            .alignment(Alignment::Center),
        stats_area,
    );

    wpm_graph::render_wpm_graph(
        frame,
        graph_area,
        app.wpm_history(),
        Block::default()
            .borders(Borders::ALL)
            .title(" Final WPM Trend ")
            .border_style(Style::default().fg(Color::LightYellow)),
    );

    render_footer(frame, footer_area, app);
}

fn render_rhythm_result(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 45, frame.area());
    let stats = app.rhythm_stats();
    let lines = stats.map_or_else(
        || {
            vec![
                Line::from("Rhythm Finished"),
                Line::from(""),
                Line::from("Typed: 0"),
                Line::from("Correct: 0"),
                Line::from("Hit: 0"),
                Line::from("OK: 0"),
                Line::from("Misses: 0"),
                Line::from("Accuracy: 0.0%"),
                Line::from("Mode: Rhythm"),
            ]
        },
        |stats| {
            vec![
                Line::from("Rhythm Finished"),
                Line::from(""),
                Line::from(format!("Typed: {}", stats.typed)),
                Line::from(format!("Correct: {}", stats.correct)),
                Line::from(format!("Hit: {}", stats.hit)),
                Line::from(format!("OK: {}", stats.ok)),
                Line::from(format!("Misses: {}", stats.miss)),
                Line::from(format!("Accuracy: {:.1}%", stats.accuracy)),
                Line::from("Mode: Rhythm"),
            ]
        },
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);
    let [metrics_area, footer_area] = &*chunks else {
        return;
    };

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Result ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Center),
        *metrics_area,
    );
    render_footer(frame, *footer_area, app);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hint = Line::from("Press Enter to return to menu");
    let footer = if let Some(message) = app.status_message() {
        Paragraph::new(vec![Line::from(message), hint]).wrap(Wrap { trim: true })
    } else {
        Paragraph::new(hint).block(Block::default().borders(Borders::ALL))
    };
    frame.render_widget(footer.alignment(Alignment::Center), area);
}

fn split_result_area(area: Rect, show_status: bool) -> [Rect; 4] {
    let stats_height = if area.height >= 24 { 10 } else { 0 };
    let graph_height = if area.height >= 18 { 6 } else { 0 };
    let footer_height = if show_status || area.height >= 12 {
        area.height.min(3)
    } else {
        0
    };
    let metrics_height = if show_status {
        area.height
            .saturating_sub(stats_height + graph_height + footer_height)
            .min(10)
    } else {
        10
    };
    let constraints = [
        Constraint::Min(metrics_height),
        Constraint::Length(stats_height),
        Constraint::Length(graph_height),
        Constraint::Length(footer_height),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    let [metrics_area, stats_area, graph_area, footer_area] = &*chunks else {
        return [area, area, area, area];
    };
    [*metrics_area, *stats_area, *graph_area, *footer_area]
}

#[cfg(test)]
mod tests {
    use super::split_result_area;
    use ratatui::layout::Rect;

    #[test]
    fn save_error_is_visible_on_a_short_result_screen() {
        use crate::domain::config::AppConfig;
        use crate::presentation::ui::app::App;
        use ratatui::{Terminal, backend::TestBackend};

        let Ok(mut terminal) = Terminal::new(TestBackend::new(80, 16));
        let mut app = App::new(AppConfig::default());
        app.set_status_message("Failed to save history: disk error");
        let Ok(_) = terminal.draw(|frame| super::render_result(frame, &app));
        let output: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(output.contains("Failed to save history: disk error"));
    }

    #[test]
    fn split_result_area_reserves_graph_and_footer_when_height_allows() {
        let [metrics_area, stats_area, graph_area, footer_area] =
            split_result_area(Rect::new(0, 0, 60, 20), false);

        assert!(metrics_area.height >= 10);
        assert_eq!(stats_area.height, 0);
        assert_eq!(graph_area.height, 6);
        assert_eq!(footer_area.height, 3);
        assert_eq!(graph_area.y, metrics_area.y + metrics_area.height);
        assert_eq!(footer_area.y, graph_area.y + graph_area.height);
    }

    #[test]
    fn split_result_area_reserves_stats_when_height_allows() {
        let [metrics_area, stats_area, graph_area, footer_area] =
            split_result_area(Rect::new(0, 0, 60, 30), false);

        assert!(metrics_area.height >= 10);
        assert_eq!(stats_area.height, 10);
        assert_eq!(graph_area.height, 6);
        assert_eq!(footer_area.height, 3);
        assert_eq!(stats_area.y, metrics_area.y + metrics_area.height);
        assert_eq!(graph_area.y, stats_area.y + stats_area.height);
    }

    #[test]
    fn split_result_area_hides_graph_when_height_is_small() {
        let [metrics_area, stats_area, graph_area, footer_area] =
            split_result_area(Rect::new(0, 0, 60, 10), false);

        assert_eq!(metrics_area.height, 10);
        assert_eq!(stats_area.height, 0);
        assert_eq!(graph_area.height, 0);
        assert_eq!(footer_area.height, 0);
    }
}
