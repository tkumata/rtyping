use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::{Arc, Mutex, mpsc};

use crate::presentation::ui::app::{App, MenuItem};
use crate::presentation::ui::render;
use crate::usecase::generate_sentence::GenerationSource;

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
                app.set_generation_source(GenerationSource::Local);
                app.set_practice_mode(false);
                reset_timer(timer);
                spawn_generation_job(app, generation_tx, next_request_id, active_request_id);
            }
            MenuItem::PracticeMode => {
                app.set_generation_source(GenerationSource::Local);
                app.set_practice_mode(true);
                reset_timer(timer);
                spawn_generation_job(app, generation_tx, next_request_id, active_request_id);
            }
            MenuItem::StartGameGoogle => {
                app.set_generation_source(GenerationSource::Google);
                app.set_practice_mode(false);
                reset_timer(timer);
                spawn_generation_job(app, generation_tx, next_request_id, active_request_id);
            }
            MenuItem::StartGameGroq => {
                app.set_generation_source(GenerationSource::Groq);
                app.set_practice_mode(false);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::presentation::ui::app::AppState;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn test_app() -> App {
        App::new(
            60,
            60,
            80.0,
            false,
            GenerationSource::Local,
            AppConfig::default(),
        )
    }

    #[test]
    fn enter_on_google_menu_sets_google_source_and_loading() {
        let mut app = test_app();
        let timer = Arc::new(Mutex::new(0));
        let (generation_tx, _generation_rx) = mpsc::channel();
        let mut next_request_id = 1;
        let mut active_request_id = None;

        app.move_menu_down();
        app.move_menu_down();
        handle_menu_input(
            key(KeyCode::Enter),
            &mut app,
            &timer,
            &generation_tx,
            &mut next_request_id,
            &mut active_request_id,
        );

        assert_eq!(app.generation_source(), GenerationSource::Google);
        assert_eq!(app.state(), AppState::Loading);
        assert_eq!(active_request_id, Some(1));
    }

    #[test]
    fn enter_on_practice_mode_sets_local_source_and_practice_mode() {
        let mut app = test_app();
        let timer = Arc::new(Mutex::new(0));
        let (generation_tx, _generation_rx) = mpsc::channel();
        let mut next_request_id = 10;
        let mut active_request_id = None;

        app.move_menu_down();
        handle_menu_input(
            key(KeyCode::Enter),
            &mut app,
            &timer,
            &generation_tx,
            &mut next_request_id,
            &mut active_request_id,
        );

        assert_eq!(app.generation_source(), GenerationSource::Local);
        assert!(app.is_practice_mode());
        assert_eq!(app.state(), AppState::Loading);
        assert_eq!(active_request_id, Some(10));
    }

    #[test]
    fn enter_on_groq_menu_sets_groq_source_and_loading() {
        let mut app = test_app();
        let timer = Arc::new(Mutex::new(0));
        let (generation_tx, _generation_rx) = mpsc::channel();
        let mut next_request_id = 3;
        let mut active_request_id = None;

        app.move_menu_down();
        app.move_menu_down();
        app.move_menu_down();
        handle_menu_input(
            key(KeyCode::Enter),
            &mut app,
            &timer,
            &generation_tx,
            &mut next_request_id,
            &mut active_request_id,
        );

        assert_eq!(app.generation_source(), GenerationSource::Groq);
        assert_eq!(app.state(), AppState::Loading);
        assert_eq!(active_request_id, Some(3));
    }
}
