use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rodio::{MixerDeviceSink, Source, source::SineWave};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use crate::presentation::ui::app::App;
use crate::runtime::TimerCommand;
use crate::runtime::timer::{cancel_typing_session, finish_typing_session, stop_timer};

pub(in crate::runtime::input) fn handle_typing_input(
    key: KeyEvent,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    audio_sink: &MixerDeviceSink,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
) {
    match key.code {
        KeyCode::Esc => cancel_typing_session(app, timer_command_tx),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            stop_timer(timer_command_tx);
            app.quit();
        }
        KeyCode::Backspace => {
            app.pop_char();
        }
        KeyCode::Char(c) => {
            let is_correct = app.push_char(c);

            if is_correct && app.typing_sound_enabled() {
                let source =
                    SineWave::new(app.frequency()).take_duration(Duration::from_millis(100));
                audio_sink.mixer().add(source);
            }

            if app.is_complete() {
                finish_typing_session(app, timer, timer_command_tx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::usecase::generate_sentence::GenerationSource;

    fn new_app() -> App {
        App::new(
            60,
            30,
            80.0,
            false,
            GenerationSource::Local,
            AppConfig::default(),
        )
    }

    #[test]
    fn escape_cancels_typing_session_and_returns_to_menu() {
        let mut app = new_app();
        let (timer_command_tx, _timer_command_rx) = mpsc::channel();
        app.start_typing();

        cancel_typing_session(&mut app, &timer_command_tx);

        assert_eq!(app.state(), crate::presentation::ui::app::AppState::Menu);
    }

    #[test]
    fn incorrect_input_does_not_advance_cursor() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());

        assert!(!app.push_char('x'));
        assert!(app.input_chars().is_empty());
        assert_eq!(app.current_input_count(), 0);
        assert_eq!(app.typed_count(), 1);
    }
}
