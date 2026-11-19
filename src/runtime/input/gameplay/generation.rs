use std::sync::mpsc;
use std::thread;

use crate::domain::config::{AppConfig, ProviderConfig};
use crate::presentation::ui::app::{App, AppState, GameMode};
use crate::usecase::generate_sentence::{self, GenerationSource};

use crate::runtime::GenerationJob;

pub(in crate::runtime::input) fn spawn_generation_job(
    app: &mut App,
    job: &mut Option<GenerationJob>,
) {
    let (text_scale, source, config) = app.generation_settings();
    let provider = provider_config_for_source(source, &config);
    app.enter_loading();

    let (sender, receiver) = mpsc::channel();
    let worker = thread::Builder::new().spawn(move || {
        let result = generate_sentence::generate(text_scale, source, provider)
            .map_err(|err| err.to_string());
        let _ = sender.send(result);
    });
    match worker {
        Ok(_) => *job = Some(receiver),
        Err(err) => {
            app.return_to_menu();
            app.set_status_message(format!("Failed to start generation: {err}"));
        }
    }
}

pub(in crate::runtime::input) fn apply_generation_result(
    app: &mut App,
    result: Result<String, String>,
) {
    if app.state() != AppState::Loading {
        return;
    }
    match result {
        Ok(contents) => match app.next_game_mode() {
            GameMode::Standard => {
                app.prepare_new_game(contents);
                app.start_typing();
            }
            GameMode::Rhythm => {
                app.prepare_rhythm_game(&contents);
                app.start_rhythm_typing();
            }
        },
        Err(message) => {
            app.return_to_menu();
            app.set_status_message(message);
        }
    }
}

pub(in crate::runtime::input) fn provider_config_for_source(
    source: GenerationSource,
    config: &AppConfig,
) -> Option<ProviderConfig> {
    match source {
        GenerationSource::Local => None,
        GenerationSource::Google => Some(config.google.clone()),
        GenerationSource::Groq => Some(config.groq.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::ui::app::App;

    fn test_app() -> App {
        App::new(AppConfig::default())
    }

    fn app_config() -> AppConfig {
        AppConfig {
            google: ProviderConfig {
                api_url: "https://google.example".into(),
                api_key: "google-secret".into(),
                model: "google-model".into(),
            },
            groq: ProviderConfig {
                api_url: "https://groq.example".into(),
                api_key: "groq-secret".into(),
                model: "groq-model".into(),
            },
            game: AppConfig::default().game,
        }
    }

    #[test]
    fn provider_config_for_local_generation_is_none() {
        assert_eq!(
            provider_config_for_source(GenerationSource::Local, &app_config()),
            None
        );
    }

    #[test]
    fn provider_config_for_google_generation_uses_google_config() {
        let config = app_config();
        assert_eq!(
            provider_config_for_source(GenerationSource::Google, &config),
            Some(config.google)
        );
    }

    #[test]
    fn provider_config_for_groq_generation_uses_groq_config() {
        let config = app_config();
        assert_eq!(
            provider_config_for_source(GenerationSource::Groq, &config),
            Some(config.groq)
        );
    }

    #[test]
    fn matching_generation_result_starts_typing() {
        let mut app = test_app();

        app.enter_loading();
        apply_generation_result(&mut app, Ok("typing text".into()));

        assert_eq!(app.state(), AppState::Typing);
        assert_eq!(app.target_string(), "typing text");
    }

    #[test]
    fn matching_generation_result_starts_rhythm() {
        let mut app = test_app();

        app.set_next_game_mode(GameMode::Rhythm);
        app.enter_loading();
        apply_generation_result(&mut app, Ok("rhythm text".into()));

        assert_eq!(app.state(), AppState::RhythmTyping);
        assert_eq!(app.target_string(), "rhythm text");
    }

    #[test]
    fn generation_result_outside_loading_is_ignored() {
        let mut app = test_app();

        apply_generation_result(&mut app, Ok("stale".into()));

        assert_eq!(app.state(), AppState::Menu);
    }

    #[test]
    fn generation_error_returns_to_menu_with_status() {
        let mut app = test_app();

        app.enter_loading();
        apply_generation_result(&mut app, Err("request failed".into()));

        assert_eq!(app.state(), AppState::Menu);
        assert_eq!(app.status_message(), Some("request failed"));
    }
}
