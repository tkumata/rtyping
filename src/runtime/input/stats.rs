use crossterm::event::{KeyCode, KeyEvent};

use crate::presentation::ui::app::App;

pub(super) fn handle_stats_input(key: KeyEvent, app: &mut App) {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        app.return_to_menu();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::presentation::ui::app::AppState;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn stats_app() -> App {
        let mut app = App::new(AppConfig::default());
        app.open_stats();
        app
    }

    #[test]
    fn enter_returns_from_stats_to_menu() {
        let mut app = stats_app();

        handle_stats_input(key(KeyCode::Enter), &mut app);

        assert_eq!(app.state(), AppState::Menu);
    }

    #[test]
    fn escape_returns_from_stats_to_menu() {
        let mut app = stats_app();

        handle_stats_input(key(KeyCode::Esc), &mut app);

        assert_eq!(app.state(), AppState::Menu);
    }
}
