use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::{Arc, Mutex, mpsc};

use crate::presentation::ui::app::{App, MenuItem};
use crate::presentation::ui::render;

use super::gameplay::spawn_generation_job;
use crate::runtime::GenerationJobResult;
use crate::runtime::timer::reset_timer;

pub(super) fn handle_menu_input(
    key: KeyEvent,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    generation_tx: &mpsc::Sender<GenerationJobResult>,
    next_request_id: &mut u64,
    active_request_id: &mut Option<u64>,
) {
    if app.status_message().is_some() && !app.is_help_visible() {
        if should_clear_status_message(&key) {
            app.clear_status_message();
        }
        return;
    }

    if key.code == KeyCode::Char('h') {
        app.toggle_help();
        return;
    }

    if app.is_help_visible() {
        match key.code {
            KeyCode::Up => app.scroll_help_up(),
            KeyCode::Down => {
                let max_scroll = render::help_line_count().saturating_sub(5);
                app.scroll_help_down(max_scroll);
            }
            KeyCode::Esc => app.hide_help(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Up => app.move_menu_up(),
        KeyCode::Down => app.move_menu_down(),
        KeyCode::Enter => match app.menu_selected() {
            MenuItem::StartGame => {
                reset_timer(timer);
                spawn_generation_job(app, generation_tx, next_request_id, active_request_id);
            }
            MenuItem::Config => {
                app.clear_status_message();
                app.open_config();
            }
        },
        KeyCode::Esc => app.quit(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        _ => {}
    }
}

fn should_clear_status_message(key: &KeyEvent) -> bool {
    matches!(
        key.code,
        KeyCode::Enter
            | KeyCode::Esc
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Tab
            | KeyCode::Backspace
            | KeyCode::Char(_)
    )
}

