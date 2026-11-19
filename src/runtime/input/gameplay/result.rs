use crossterm::event::{KeyCode, KeyEvent};

use crate::presentation::ui::app::App;

pub(in crate::runtime::input) fn handle_result_input(key: KeyEvent, app: &mut App) {
    if key.code == KeyCode::Enter {
        app.return_to_menu_with_start_selected();
    }
}
