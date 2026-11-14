//! R-Typing: A terminal-based typing practice application.
//!
//! This application provides a typing game with countdown timer,
//! real-time WPM calculation, and optional background music.

mod config;
mod domain;
mod presentation;
mod runtime;
mod usecase;

use crossterm::{
    cursor::{SetCursorStyle, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use rodio::DeviceSinkBuilder;
use std::io::{self, Write, stdout};
use std::mem::DropGuard;
use std::sync::{Arc, Mutex, mpsc};

use domain::config::AppConfig;
use presentation::bgm_handler::BgmHandler;
use presentation::ui::app::App;

fn main() -> io::Result<()> {
    let (loaded_config, config_message) = match config::load_config() {
        Ok(report) => {
            let message = if report.warnings.is_empty() {
                None
            } else {
                Some(format!("Config warning: {}", report.warnings.join(" / ")))
            };
            (report.config, message)
        }
        Err(err) => (
            AppConfig::default(),
            Some(format!("Failed to load config: {err}")),
        ),
    };
    let (history_entries, history_message) = match config::load_history() {
        Ok(report) => {
            let message = if report.warnings.is_empty() {
                None
            } else {
                Some(format!("History warning: {}", report.warnings.join(" / ")))
            };
            (report.entries, message)
        }
        Err(err) => (Vec::new(), Some(format!("Failed to load history: {err}"))),
    };

    let mut audio_sink = DeviceSinkBuilder::open_default_sink()
        .map_err(|err| io::Error::other(format!("failed to open audio device: {err}")))?;
    audio_sink.log_on_drop(false);
    let (snd_sender, snd_receiver) = mpsc::channel();

    if loaded_config.game.sound_enabled_value() {
        let bgm_handler = BgmHandler::new(snd_receiver);
        bgm_handler.start();
    }

    enable_raw_mode()?;
    let terminal_restore = DropGuard::new((), |()| {
        if let Err(err) = restore_terminal(&mut stdout()) {
            let _ = writeln!(io::stderr(), "Failed to restore terminal: {err}");
        }
    });
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(loaded_config);
    app.set_history_entries(history_entries);
    let startup_messages = [config_message, history_message]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if !startup_messages.is_empty() {
        app.set_status_message(startup_messages.join(" / "));
    }

    let timer = Arc::new(Mutex::new(0i32));
    let (timer_command_tx, timer_command_rx) = mpsc::channel::<runtime::TimerCommand>();
    let (timeout_tx, timeout_rx) = mpsc::channel::<()>();
    let timer_thread =
        runtime::spawn_timer_thread(Arc::clone(&timer), 0, timer_command_rx, timeout_tx);

    let res = runtime::run_app(
        &mut terminal,
        &mut app,
        &timer,
        &audio_sink,
        &timer_command_tx,
        &timeout_rx,
    );

    restore_terminal(terminal.backend_mut())?;
    DropGuard::dismiss(terminal_restore);

    timer_command_tx.send(runtime::TimerCommand::Shutdown).ok();
    timer_thread.join().ok();
    snd_sender.send(()).ok();

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}

fn restore_terminal(output: &mut impl Write) -> io::Result<()> {
    let raw_mode_result = disable_raw_mode();
    let screen_result = execute!(
        output,
        SetCursorStyle::DefaultUserShape,
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    );
    raw_mode_result.and(screen_result)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn restore_terminal_resets_screen_mouse_and_cursor() {
        let mut output = Vec::new();

        assert!(restore_terminal(&mut output).is_ok());

        assert_eq!(
            output,
            b"\x1b[0 q\x1b[?1049l\x1b[?1006l\x1b[?1015l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?25h"
        );
    }

    #[test]
    fn restore_terminal_returns_output_error() {
        let mut output = io::Cursor::new([]);

        let result = restore_terminal(&mut output);

        assert_eq!(
            result.err().map(|err| err.kind()),
            Some(io::ErrorKind::WriteZero)
        );
    }
}
