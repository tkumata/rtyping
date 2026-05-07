use crossterm::event::{KeyCode, KeyEvent};

use crate::presentation::ui::app::App;

pub(super) fn handle_stats_input(key: KeyEvent, app: &mut App) {
    if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
        app.return_to_menu();
    }
}
