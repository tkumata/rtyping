use crossterm::event::{KeyCode, KeyEvent};
use std::sync::{Arc, Mutex};

use crate::presentation::ui::app::App;
use crate::runtime::timer::reset_timer;

pub(in crate::runtime::input) fn handle_result_input(
    key: KeyEvent,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
) {
    if key.code == KeyCode::Enter {
        reset_timer(timer);
        app.return_to_menu_with_start_selected();
    }
}
