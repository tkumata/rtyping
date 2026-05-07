use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::presentation::ui::app::App;
use crate::usecase::accuracy;
use crate::usecase::wpm;

use super::common::centered_rect;
use super::history_summary::history_summary_lines;
use super::wpm_graph;

pub fn render_result(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, frame.area());
    let elapsed = app.timer().max(1);
    let score = wpm::calc_wpm(
        app.typed_count(),
        elapsed,
        i32::try_from(app.incorrects()).unwrap_or(i32::MAX),
    );
    let accuracy = accuracy::calc_accuracy(app.typed_count(), app.incorrects());
    let [metrics_area, stats_area, graph_area, footer_area] = split_result_area(area);
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

    wpm_graph::render_wpm_graph(frame, graph_area, app.wpm_history(), " Final WPM Trend ");

    frame.render_widget(
        Paragraph::new(vec![Line::from("Press Enter to return to menu")])
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        footer_area,
    );
}

fn split_result_area(area: Rect) -> [Rect; 4] {
    let stats_height = if area.height >= 24 { 10 } else { 0 };
    let graph_height = if area.height >= 18 { 6 } else { 0 };
    let footer_height = if area.height >= 12 { 3 } else { 0 };
    let constraints = if stats_height > 0 && graph_height > 0 && footer_height > 0 {
        [
            Constraint::Min(10),
            Constraint::Length(stats_height),
            Constraint::Length(graph_height),
            Constraint::Length(footer_height),
        ]
    } else if graph_height > 0 && footer_height > 0 {
        [
            Constraint::Min(10),
            Constraint::Length(0),
            Constraint::Length(graph_height),
            Constraint::Length(footer_height),
        ]
    } else if footer_height > 0 {
        [
            Constraint::Min(10),
            Constraint::Length(0),
            Constraint::Length(0),
            Constraint::Length(footer_height),
        ]
    } else {
        [
            Constraint::Min(10),
            Constraint::Length(0),
            Constraint::Length(0),
            Constraint::Length(0),
        ]
    };

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
    fn split_result_area_reserves_graph_and_footer_when_height_allows() {
        let [metrics_area, stats_area, graph_area, footer_area] =
            split_result_area(Rect::new(0, 0, 60, 20));

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
            split_result_area(Rect::new(0, 0, 60, 30));

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
            split_result_area(Rect::new(0, 0, 60, 10));

        assert_eq!(metrics_area.height, 10);
        assert_eq!(stats_area.height, 0);
        assert_eq!(graph_area.height, 0);
        assert_eq!(footer_area.height, 0);
    }
}
