use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event},
    execute,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use rodio::MixerDeviceSink;
use std::io;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use crate::presentation::ui::app::{App, AppState};
use crate::presentation::ui::render;

use super::input::{drain_generation_results, handle_key_event};
use super::timer::current_timer;
use super::{GenerationJobResult, RuntimeContext, TimerCommand};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    audio_sink: &MixerDeviceSink,
    timer_command_tx: &mpsc::Sender<TimerCommand>,
    timeout_rx: &mpsc::Receiver<()>,
) -> io::Result<()> {
    let (generation_tx, generation_rx) = mpsc::channel::<GenerationJobResult>();
    let mut next_request_id = 1_u64;
    let mut active_request_id: Option<u64> = None;
    let mut using_typing_cursor_style = false;

    loop {
        let is_typing = app.state() == AppState::Typing;
        if is_typing && !using_typing_cursor_style {
            execute!(terminal.backend_mut(), SetCursorStyle::SteadyBar)?;
            using_typing_cursor_style = true;
        } else if !is_typing && using_typing_cursor_style {
            execute!(terminal.backend_mut(), SetCursorStyle::DefaultUserShape)?;
            using_typing_cursor_style = false;
        }

        terminal.draw(|frame| render::render(frame, app))?;

        if app.is_quit_requested() {
            break;
        }

        drain_generation_results(
            &generation_rx,
            app,
            timer_command_tx,
            &mut active_request_id,
        );

        if app.state() == AppState::Typing && app.timeout() > 0 && timeout_rx.try_recv().is_ok() {
            app.update_timer(current_timer(timer));
            app.finish_typing();
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            let mut context = RuntimeContext {
                timer,
                generation_tx: &generation_tx,
                next_request_id: &mut next_request_id,
                active_request_id: &mut active_request_id,
                audio_sink,
                timer_command_tx,
            };
            handle_key_event(key, app, &mut context);
        }

        if app.state() == AppState::Typing {
            app.update_timer(current_timer(timer));
        }
    }

    Ok(())
}
