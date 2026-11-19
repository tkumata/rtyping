use crate::domain::rhythm::RhythmJudgement;
use crate::presentation::audio::Audio;
use crate::presentation::ui::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::play_typing_sound;

pub(in crate::runtime::input) fn handle_rhythm_input(
    key: KeyEvent,
    app: &mut App,
    audio: &mut Audio,
) {
    match key.code {
        KeyCode::Esc => app.return_to_menu(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Char(ch) => {
            if matches!(
                app.push_rhythm_char(ch),
                RhythmJudgement::Hit | RhythmJudgement::Ok
            ) {
                play_typing_sound(app, audio);
            }
        }
        _ => {}
    }
}
