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
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use rodio::DeviceSinkBuilder;
use std::io::{self, stdout};
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

    let mut audio_sink = DeviceSinkBuilder::open_default_sink()
        .map_err(|err| io::Error::other(format!("failed to open audio device: {err}")))?;
    audio_sink.log_on_drop(false);
    let (snd_sender, snd_receiver) = mpsc::channel();

    if loaded_config.game.sound_enabled_value() {
        let bgm_handler = BgmHandler::new(snd_receiver);
        bgm_handler.start();
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(loaded_config);
    if let Some(message) = config_message {
        app.set_status_message(message);
    }

    let timer = Arc::new(Mutex::new(0i32));
    let (timer_command_tx, timer_command_rx) = mpsc::channel::<runtime::TimerCommand>();
    let (timeout_tx, timeout_rx) = mpsc::channel::<()>();
    let timer_thread = runtime::spawn_timer_thread(
        Arc::clone(&timer),
        0,
        timer_command_rx,
        timeout_tx,
    );

    let res = runtime::run_app(
        &mut terminal,
        &mut app,
        &timer,
        &audio_sink,
        &timer_command_tx,
        &timeout_rx,
    );

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    timer_command_tx.send(runtime::TimerCommand::Shutdown).ok();
    timer_thread.join().ok();
    snd_sender.send(()).ok();

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}
