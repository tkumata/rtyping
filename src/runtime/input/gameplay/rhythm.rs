use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rodio::{MixerDeviceSink, Source, source::SineWave};
use std::sync::mpsc;
use std::time::Duration;

use crate::domain::rhythm::RhythmJudgement;
use crate::presentation::ui::app::App;
use crate::runtime::TimerCommand;
use crate::runtime::timer::{cancel_typing_session, stop_timer};

pub(in crate::runtime::input) fn handle_rhythm_input(
    key: KeyEvent,
    app: &mut App,
    audio_sink: &MixerDeviceSink,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
) {
    match key.code {
        KeyCode::Esc => cancel_typing_session(app, timer_command_tx),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            stop_timer(timer_command_tx);
            app.quit();
        }
        KeyCode::Char(c) => {
            let judgement = app.push_rhythm_char(c);
            if matches!(judgement, RhythmJudgement::Hit | RhythmJudgement::Ok)
                && app.typing_sound_enabled()
            {
                let source =
                    SineWave::new(app.frequency()).take_duration(Duration::from_millis(100));
                audio_sink.mixer().add(source);
            }
            if app.is_rhythm_complete() {
                app.finish_typing();
            }
        }
        _ => {}
    }
}
