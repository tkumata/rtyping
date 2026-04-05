use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::presentation::ui::app::App;

use super::common::centered_rect;

pub fn render_loading(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);
    let provider = match app.generation_source() {
        crate::usecase::generate_sentence::GenerationSource::Local => "Local",
        crate::usecase::generate_sentence::GenerationSource::Google => "Google AI Studio",
        crate::usecase::generate_sentence::GenerationSource::Groq => "Groq",
    };
    let text = vec![
        Line::from(format!("Generating text with {provider}")),
        Line::from(""),
        Line::from("Please wait..."),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Loading ")
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center),
        area,
    );
}
