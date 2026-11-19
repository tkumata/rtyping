use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config;
use crate::presentation::ui::app::{App, ConfigField};

pub(super) fn handle_config_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Up => app.move_config_up(),
        KeyCode::Down | KeyCode::Tab => app.move_config_down(),
        KeyCode::Left => app.move_config_cursor_left(),
        KeyCode::Right => app.move_config_cursor_right(),
        KeyCode::Backspace => app.pop_config_char(),
        KeyCode::Char(' ') if app.config_field() == ConfigField::GameSoundEnabled => {
            app.toggle_sound_enabled();
        }
        KeyCode::Enter => match config::save_config(app.config_draft()) {
            Ok(()) => {
                app.apply_config_draft();
                app.return_to_menu_with_start_selected();
                app.set_status_message("Configuration saved");
            }
            Err(err) => {
                app.set_status_message(format!("Failed to save configuration: {err}"));
            }
        },
        KeyCode::Esc => {
            app.return_to_menu();
            app.set_status_message("Configuration changes discarded");
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Char(ch) if accepts_config_char(key.modifiers) => {
            app.edit_config_char(ch);
        }
        _ => {}
    }
}

fn accepts_config_char(modifiers: KeyModifiers) -> bool {
    modifiers.is_empty() || modifiers == KeyModifiers::SHIFT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_discards_the_draft() {
        let mut app = App::new(crate::domain::config::AppConfig::default());
        app.open_config();
        app.edit_config_char('x');
        handle_config_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut app);
        app.open_config();
        assert_eq!(app.config_draft().google.api_url, "");
    }

    #[test]
    fn config_char_input_accepts_plain_and_shift_only() {
        assert!(accepts_config_char(KeyModifiers::empty()));
        assert!(accepts_config_char(KeyModifiers::SHIFT));
        assert!(!accepts_config_char(KeyModifiers::CONTROL));
    }
}
