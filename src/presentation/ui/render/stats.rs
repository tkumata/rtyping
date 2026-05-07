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
