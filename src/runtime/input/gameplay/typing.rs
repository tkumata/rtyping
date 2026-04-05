use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rodio::{MixerDeviceSink, Source, source::SineWave};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use crate::presentation::ui::app::App;
use crate::runtime::TimerCommand;
use crate::runtime::timer::{finish_typing_session, stop_timer};

pub(in crate::runtime::input) fn handle_typing_input(
    key: KeyEvent,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    audio_sink: &MixerDeviceSink,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
) {
    match key.code {
        KeyCode::Esc => finish_typing_session(app, timer, timer_command_tx),
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
