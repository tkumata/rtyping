use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::presentation::ui::app::App;

use super::common::centered_rect;
use super::history_summary::history_summary_lines;

pub fn render_stats(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, frame.area());
    let stats = app.history_stats();
    let mut lines = history_summary_lines(&stats);
    lines.push("".into());
    lines.push("Press Enter or Esc to return to menu".into());

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Stats ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
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
        assert!(text.contains("History Stats"));
        assert!(text.contains("No timed history yet"));
        assert!(text.contains("Press Enter or Esc to return to menu"));
    }
}
