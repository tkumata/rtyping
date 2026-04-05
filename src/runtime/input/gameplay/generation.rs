use std::sync::mpsc;
use std::thread;

use crate::domain::config::{AppConfig, ProviderConfig};
use crate::presentation::ui::app::{App, AppState};
use crate::usecase::generate_sentence::{self, GenerationSource};

use crate::runtime::{GenerationJobResult, TimerCommand};

pub(in crate::runtime::input) fn spawn_generation_job(
    app: &mut App,
    generation_tx: &mpsc::Sender<GenerationJobResult>,
    next_request_id: &mut u64,
    active_request_id: &mut Option<u64>,
) {
    let (text_scale, source, config) = app.generation_settings();
    let provider = provider_config_for_source(source, &config);
    app.enter_loading();

    let sender = generation_tx.clone();
    let request_id = *next_request_id;
    *next_request_id += 1;
    *active_request_id = Some(request_id);

    thread::spawn(move || {
        let result = generate_sentence::generate(text_scale, source, provider)
            .map_err(|err| err.to_string());
        sender.send(GenerationJobResult { request_id, result }).ok();
    });
}

pub(in crate::runtime::input) fn apply_generation_result(
    app: &mut App,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
    active_request_id: &mut Option<u64>,
    job: GenerationJobResult,
) {
    if Some(job.request_id) != *active_request_id {
        return;
    }

    *active_request_id = None;
    match job.result {
        Ok(contents) if app.state() == AppState::Loading => {
            app.prepare_new_game(contents);
            app.start_typing();
            timer_command_tx.send(TimerCommand::Start).ok();
        }
        Ok(_) => {}
        Err(message) if app.state() == AppState::Loading => {
            app.return_to_menu();
            app.set_status_message(message);
        }
        Err(_) => {}
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
        App::new(
            60,
            30,
            80.0,
            false,
            GenerationSource::Local,
            AppConfig::default(),
        )
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
    fn matching_generation_result_starts_typing_and_timer() {
        let mut app = test_app();
        let (timer_tx, timer_rx) = mpsc::channel();
        let mut active_request_id = Some(4);

        app.enter_loading();
        apply_generation_result(
            &mut app,
            &timer_tx,
            &mut active_request_id,
            GenerationJobResult {
                request_id: 4,
                result: Ok("typing text".into()),
            },
        );

        assert_eq!(app.state(), AppState::Typing);
        assert_eq!(app.target_string(), "typing text");
        assert_eq!(active_request_id, None);
        assert!(matches!(timer_rx.try_recv(), Ok(TimerCommand::Start)));
    }

    #[test]
    fn stale_generation_result_is_ignored() {
        let mut app = test_app();
        let (timer_tx, timer_rx) = mpsc::channel();
        let mut active_request_id = Some(5);

        app.enter_loading();
        apply_generation_result(
            &mut app,
            &timer_tx,
            &mut active_request_id,
            GenerationJobResult {
                request_id: 6,
                result: Ok("stale".into()),
            },
        );

        assert_eq!(app.state(), AppState::Loading);
        assert_eq!(active_request_id, Some(5));
        assert!(timer_rx.try_recv().is_err());
    }

    #[test]
    fn generation_error_returns_to_menu_with_status() {
        let mut app = test_app();
        let (timer_tx, _) = mpsc::channel();
        let mut active_request_id = Some(7);

        app.enter_loading();
        apply_generation_result(
            &mut app,
            &timer_tx,
            &mut active_request_id,
            GenerationJobResult {
                request_id: 7,
                result: Err("request failed".into()),
            },
        );

        assert_eq!(app.state(), AppState::Menu);
        assert_eq!(app.status_message(), Some("request failed"));
        assert_eq!(active_request_id, None);
    }
}
