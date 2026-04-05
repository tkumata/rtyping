use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::presentation::ui::app::App;

pub(in crate::runtime::input) fn handle_loading_input(key: KeyEvent, app: &mut App, active_request_id: &mut Option<u64>) {
    match key.code {
        KeyCode::Esc => {
            *active_request_id = None;
            app.return_to_menu();
            app.set_status_message("Generation canceled");
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::presentation::ui::app::App;
    use crate::usecase::generate_sentence::GenerationSource;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn escape_on_loading_cancels_request() {
        let mut app = App::new(60, 30, 80.0, false, GenerationSource::Local, AppConfig::default());
        let mut active_request_id = Some(8);

        app.enter_loading();
        handle_loading_input(key(KeyCode::Esc), &mut app, &mut active_request_id);

        assert_eq!(app.state(), crate::presentation::ui::app::AppState::Menu);
        assert_eq!(app.status_message(), Some("Generation canceled"));
        assert_eq!(active_request_id, None);
    }
}
