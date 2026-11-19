mod config_screen;
mod gameplay;
mod menu;
mod stats;

use crossterm::event::KeyEvent;
use std::sync::mpsc;

use crate::presentation::ui::app::{App, AppState};

use crate::presentation::audio::Audio;

use super::GenerationJob;

pub(super) fn handle_key_event(
    key: KeyEvent,
    app: &mut App,
    job: &mut Option<GenerationJob>,
    audio: &mut Audio,
) {
    match app.state() {
        AppState::Menu => menu::handle_menu_input(key, app, job),
        AppState::Config => config_screen::handle_config_input(key, app),
        AppState::Stats => stats::handle_stats_input(key, app),
        AppState::Loading => gameplay::handle_loading_input(key, app, job),
        AppState::Typing => gameplay::handle_typing_input(key, app, audio),
        AppState::RhythmTyping => gameplay::handle_rhythm_input(key, app, audio),
        AppState::Result => gameplay::handle_result_input(key, app),
    }
}

pub(super) fn receive_generation_result(app: &mut App, job: &mut Option<GenerationJob>) {
    let Some(receiver) = job else {
        return;
    };
    let result = match receiver.try_recv() {
        Ok(result) => result,
        Err(mpsc::TryRecvError::Empty) => return,
        Err(mpsc::TryRecvError::Disconnected) => Err("Generation worker disconnected".into()),
    };
    *job = None;
    gameplay::apply_generation_result(app, result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_reports_a_disconnected_generation_worker() {
        let mut app = App::new(crate::domain::config::AppConfig::default());
        app.enter_loading();
        let (sender, receiver) = mpsc::channel();
        let mut job = Some(receiver);
        drop(sender);

        receive_generation_result(&mut app, &mut job);

        assert_eq!(app.state(), AppState::Menu);
        assert_eq!(app.status_message(), Some("Generation worker disconnected"));
        assert!(job.is_none());
    }
}
