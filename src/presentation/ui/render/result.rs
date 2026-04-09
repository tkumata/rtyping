use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::presentation::ui::app::App;
use crate::usecase::accuracy;
use crate::usecase::wpm;

use super::common::centered_rect;

pub fn render_result(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 35, frame.area());
    let elapsed = app.timer().max(1);
    let score = wpm::calc_wpm(app.current_input_count(), elapsed, app.incorrects() as i32);
    let accuracy = accuracy::calc_accuracy(app.typed_count(), app.incorrects());
    let lines = vec![
        Line::from("Typing Finished"),
        Line::from(""),
        Line::from(format!("Typed: {}", app.typed_count())),
        Line::from(format!("Misses: {}", app.incorrects())),
        Line::from(format!("Accuracy: {:.1}%", accuracy)),
        Line::from(format!("Time: {} sec", elapsed)),
        Line::from(format!("WPM: {:.1}", score)),
        Line::from(""),
        Line::from("Press Enter to return to menu"),
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
        area,
    );
}
