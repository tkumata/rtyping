use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config;
use crate::presentation::ui::app::App;

pub(super) fn handle_config_input(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Up => app.move_config_up(),
        KeyCode::Down | KeyCode::Tab => app.move_config_down(),
        KeyCode::Backspace => app.pop_config_char(),
        KeyCode::Enter => match config::save_config(app.config()) {
            Ok(()) => {
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
        KeyCode::Char(ch) => {
            if accepts_config_char(key.modifiers) {
                app.edit_config_char(ch);
            }
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
    fn config_char_input_accepts_plain_and_shift_only() {
        assert!(accepts_config_char(KeyModifiers::empty()));
        assert!(accepts_config_char(KeyModifiers::SHIFT));
        assert!(!accepts_config_char(KeyModifiers::CONTROL));
    }
}
