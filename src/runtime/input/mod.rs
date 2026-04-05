mod config_screen;
mod gameplay;
mod menu;

use crossterm::event::KeyEvent;
use std::sync::mpsc;

use crate::presentation::ui::app::{App, AppState};

use super::{GenerationJobResult, RuntimeContext, TimerCommand};

pub(super) fn handle_key_event(key: KeyEvent, app: &mut App, context: &mut RuntimeContext<'_>) {
    match app.state() {
        AppState::Menu => menu::handle_menu_input(
            key,
            app,
            context.timer,
            context.generation_tx,
            context.next_request_id,
            context.active_request_id,
        ),
        AppState::Config => config_screen::handle_config_input(key, app),
        AppState::Loading => gameplay::handle_loading_input(key, app, context.active_request_id),
        AppState::Typing => gameplay::handle_typing_input(
            key,
            app,
            context.timer,
            context.audio_sink,
            context.timer_command_tx,
        ),
        AppState::Result => gameplay::handle_result_input(key, app, context.timer),
    }
}

pub(super) fn drain_generation_results(
    generation_rx: &mpsc::Receiver<GenerationJobResult>,
    app: &mut App,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
    active_request_id: &mut Option<u64>,
) {
    loop {
        match generation_rx.try_recv() {
            Ok(job) => {
                gameplay::apply_generation_result(app, timer_command_tx, active_request_id, job)
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                if app.state() == AppState::Loading {
                    *active_request_id = None;
                    app.return_to_menu();
                    app.set_status_message("Generation worker disconnected");
                }
                break;
            }
        }
    }
}
