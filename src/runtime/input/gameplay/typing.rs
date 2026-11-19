use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::presentation::audio::Audio;
use crate::presentation::ui::app::App;

use super::play_typing_sound;

pub(in crate::runtime::input) fn handle_typing_input(
    key: KeyEvent,
    app: &mut App,
    audio: &mut Audio,
) {
    match key.code {
        KeyCode::Esc => app.return_to_menu(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Backspace => {
            app.pop_char();
        }
        KeyCode::Char(ch) if app.push_char(ch) => play_typing_sound(app, audio),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;

    fn new_app() -> App {
        App::new(AppConfig::default())
    }

    #[test]
    fn escape_cancels_typing_session_and_returns_to_menu() {
        let mut app = new_app();
        app.start_typing();

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handle_typing_input(key, &mut app, &mut Audio::default());

        assert_eq!(app.state(), crate::presentation::ui::app::AppState::Menu);
    }

    #[test]
    fn incorrect_input_does_not_advance_cursor_in_practice_mode() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.set_practice_mode(true);

        assert!(!app.push_char('x'));
        assert_eq!(app.input_chars(), [] as [char; 0]);
        assert_eq!(app.current_input_count(), 0);
        assert_eq!(app.typed_count(), 1);
    }

    #[test]
    fn incorrect_input_advances_cursor_outside_practice_mode() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());

        assert!(!app.push_char('x'));
        assert_eq!(app.input_chars(), &['x']);
        assert_eq!(app.current_input_count(), 1);
        assert_eq!(app.typed_count(), 1);
    }
}
