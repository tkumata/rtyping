use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::presentation::ui::app::{App, GameMode, MenuItem};
use crate::presentation::ui::render;
use crate::usecase::generate_sentence::GenerationSource;

use super::gameplay::spawn_generation_job;
use crate::runtime::GenerationJob;

pub(super) fn handle_menu_input(key: KeyEvent, app: &mut App, job: &mut Option<GenerationJob>) {
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
                app.set_next_game_mode(GameMode::Standard);
                app.set_practice_mode(false);
                spawn_generation_job(app, job);
            }
            MenuItem::PracticeMode => {
                app.set_generation_source(GenerationSource::Local);
                app.set_next_game_mode(GameMode::Standard);
                app.set_practice_mode(true);
                spawn_generation_job(app, job);
            }
            MenuItem::StartGameRhythm => {
                app.set_generation_source(GenerationSource::Local);
                app.set_next_game_mode(GameMode::Rhythm);
                app.set_practice_mode(false);
                spawn_generation_job(app, job);
            }
            MenuItem::StartGameGoogle => {
                app.set_generation_source(GenerationSource::Google);
                app.set_next_game_mode(GameMode::Standard);
                app.set_practice_mode(false);
                spawn_generation_job(app, job);
            }
            MenuItem::StartGameGroq => {
                app.set_generation_source(GenerationSource::Groq);
                app.set_next_game_mode(GameMode::Standard);
                app.set_practice_mode(false);
                spawn_generation_job(app, job);
            }
            MenuItem::Stats => {
                app.open_stats();
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
    use crate::domain::config::{AppConfig, ProviderConfig};
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
        App::new(AppConfig::default())
    }

    fn test_app_with_provider_config() -> App {
        App::new(AppConfig {
            google: ProviderConfig {
                api_url: "https://google.example".to_string(),
                api_key: "google-key".to_string(),
                model: "google-model".to_string(),
            },
            groq: ProviderConfig {
                api_url: "https://groq.example".to_string(),
                api_key: "groq-key".to_string(),
                model: "groq-model".to_string(),
            },
            ..AppConfig::default()
        })
    }

    #[test]
    fn enter_on_google_menu_sets_google_source_and_loading() {
        let mut app = test_app_with_provider_config();
        let mut job = None;

        app.move_menu_down();
        app.move_menu_down();
        app.move_menu_down();
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.generation_source(), GenerationSource::Google);
        assert_eq!(app.state(), AppState::Loading);
        assert!(job.is_some());
    }

    #[test]
    fn enter_on_practice_mode_sets_local_source_and_practice_mode() {
        let mut app = test_app_with_provider_config();
        let mut job = None;

        app.move_menu_down();
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.generation_source(), GenerationSource::Local);
        assert!(app.is_practice_mode());
        assert_eq!(app.state(), AppState::Loading);
        assert!(job.is_some());
    }

    #[test]
    fn enter_on_rhythm_menu_sets_local_source_and_loading() {
        let mut app = test_app();
        let mut job = None;

        app.move_menu_down();
        app.move_menu_down();
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.generation_source(), GenerationSource::Local);
        assert_eq!(app.next_game_mode(), GameMode::Rhythm);
        assert_eq!(app.state(), AppState::Loading);
        assert!(job.is_some());
    }

    #[test]
    fn enter_on_groq_menu_sets_groq_source_and_loading() {
        let mut app = test_app_with_provider_config();
        let mut job = None;

        app.move_menu_down();
        app.move_menu_down();
        app.move_menu_down();
        app.move_menu_down();
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.generation_source(), GenerationSource::Groq);
        assert_eq!(app.state(), AppState::Loading);
        assert!(job.is_some());
    }

    #[test]
    fn enter_on_stats_menu_opens_stats_without_generation() {
        let mut app = test_app();
        let mut job = None;

        app.move_menu_down();
        app.move_menu_down();
        app.move_menu_down();
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.state(), AppState::Stats);
        assert!(job.is_none());
    }

    #[test]
    fn help_visible_blocks_menu_confirmation_until_escape_closes_help() {
        let mut app = test_app();
        let mut job = None;

        handle_menu_input(key(KeyCode::Char('h')), &mut app, &mut job);
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.state(), AppState::Menu);
        assert!(app.is_help_visible());
        assert!(job.is_none());

        handle_menu_input(key(KeyCode::Esc), &mut app, &mut job);

        assert_eq!(app.state(), AppState::Menu);
        assert!(!app.is_help_visible());
        assert!(!app.is_quit_requested());
    }

    #[test]
    fn enter_on_config_menu_opens_config_without_generation() {
        let mut app = test_app();
        let mut job = None;

        app.move_menu_up();
        handle_menu_input(key(KeyCode::Enter), &mut app, &mut job);

        assert_eq!(app.state(), AppState::Config);
        assert!(job.is_none());
    }
}
