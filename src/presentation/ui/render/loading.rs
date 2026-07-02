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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::usecase::generate_sentence::GenerationSource;
    use ratatui::{Terminal, backend::TestBackend};
    use std::convert::Infallible;

    fn value_from_infallible<T>(result: Result<T, Infallible>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => match err {},
        }
    }

    fn render_to_text(app: &App) -> String {
        let backend = TestBackend::new(80, 40);
        let mut terminal = value_from_infallible(Terminal::new(backend));
        value_from_infallible(terminal.draw(|frame| render_loading(frame, app)));

        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    }

    #[test]
    fn loading_render_includes_provider_and_wait_message() {
        let mut app = App::new(AppConfig::default());
        app.set_generation_source(GenerationSource::Groq);

        let text = render_to_text(&app);

        assert!(text.contains("Loading"));
        assert!(text.contains("Generating text with Groq"));
        assert!(text.contains("Please wait..."));
    }
}
