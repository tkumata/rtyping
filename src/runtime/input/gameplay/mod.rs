mod generation;
mod loading;
mod result;
mod rhythm;
mod typing;

pub(super) use generation::{apply_generation_result, spawn_generation_job};
pub(super) use loading::handle_loading_input;
pub(super) use result::handle_result_input;
pub(super) use rhythm::handle_rhythm_input;
pub(super) use typing::handle_typing_input;

use crate::presentation::audio::Audio;
use crate::presentation::ui::app::App;

fn play_typing_sound(app: &mut App, audio: &mut Audio) {
    if app.typing_sound_enabled()
        && let Err(err) = audio.play_typing_sound(app.frequency())
    {
        app.set_status_message(format!("Failed to play typing sound: {err}"));
    }
}
