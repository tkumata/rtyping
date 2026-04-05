//! R-Typing: A terminal-based typing practice application.
//!
//! This application provides a typing game with countdown timer,
//! real-time WPM calculation, and optional background music.

mod config;
mod domain;
mod presentation;
mod usecase;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Source, source::SineWave};
use std::io::{self, stdout};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use config::AppConfig;
use presentation::bgm_handler::BgmHandler;
use presentation::sentence_handler::SentenceHandler;
use presentation::ui::app::{App, AppState, MenuItem};
use presentation::ui::render;
use presentation::ui::ui_handler::UiHandler;

fn main() -> io::Result<()> {
    let args = UiHandler::parse_args();
    let (loaded_config, config_message) = match config::load_config() {
        Ok(config) => (config, None),
        Err(err) => (
            AppConfig::default(),
            Some(format!("Failed to load config: {err}")),
        ),
    };

    let audio_sink = DeviceSinkBuilder::open_default_sink()
        .map_err(|err| io::Error::other(format!("failed to open audio device: {err}")))?;
    let (snd_sender, snd_receiver) = mpsc::channel();

    if args.sound {
        let bgm_handler = BgmHandler::new(snd_receiver);
        bgm_handler.start();
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        args.timeout,
        args.level,
        args.freq,
        args.sound,
        args.source,
        loaded_config,
    );
    if let Some(message) = config_message {
        app.set_status_message(message);
    }

    let timer = Arc::new(Mutex::new(0i32));
    let timeout = args.timeout;

    let (timer_stop_tx, timer_stop_rx) = mpsc::channel::<()>();
    let (timeout_tx, timeout_rx) = mpsc::channel::<()>();
    let (timer_start_tx, timer_start_rx) = mpsc::channel::<()>();

    let timer_clone = Arc::clone(&timer);
    let timer_thread = thread::spawn(move || {
        if timer_start_rx.recv().is_err() {
            return;
        }

        loop {
            if timer_stop_rx.try_recv().is_ok() {
                break;
            }
            thread::sleep(Duration::from_secs(1));
            let mut t = timer_clone.lock().unwrap();
            *t += 1;
            if *t >= timeout {
                timeout_tx.send(()).ok();
                break;
            }
        }
    });

    let res = run_app(
        &mut terminal,
        &mut app,
        &timer,
        &audio_sink,
        timer_stop_tx,
        timeout_rx,
        timer_start_tx,
    );

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    timer_thread.join().unwrap();
    snd_sender.send(()).ok();

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    audio_sink: &MixerDeviceSink,
    timer_stop_tx: mpsc::Sender<()>,
    timeout_rx: mpsc::Receiver<()>,
    timer_start_tx: mpsc::Sender<()>,
) -> io::Result<()> {
    let (generation_tx, generation_rx) = mpsc::channel::<Result<String, String>>();
    let mut timer_started = false;

    loop {
        terminal.draw(|f| render::render(f, app))?;

        if app.should_quit {
            break;
        }

        if app.state == AppState::Loading {
            match generation_rx.try_recv() {
                Ok(Ok(contents)) => {
                    *timer.lock().unwrap() = 0;
                    app.prepare_new_game(contents);
                    app.start_typing();
                    if !timer_started {
                        timer_start_tx.send(()).ok();
                        timer_started = true;
                    }
                }
                Ok(Err(message)) => {
                    app.return_to_menu();
                    app.set_status_message(message);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.return_to_menu();
                    app.set_status_message("Generation worker disconnected");
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        if app.state == AppState::Typing && timeout_rx.try_recv().is_ok() {
            let current_timer = *timer.lock().unwrap();
            app.update_timer(current_timer);
            app.finish_typing();
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match app.state {
                AppState::Menu => handle_menu_input(key, app, timer, &generation_tx),
                AppState::Config => handle_config_input(key, app)?,
                AppState::Loading => handle_loading_input(key, app),
                AppState::Typing => {
                    handle_typing_input(key, app, timer, audio_sink, &timer_stop_tx)
                }
                AppState::Result => {
                    if key.code == KeyCode::Enter {
                        app.quit();
                    }
                }
            }
        }

        if app.state == AppState::Typing {
            let current_timer = *timer.lock().unwrap();
            app.update_timer(current_timer);
        }
    }

    Ok(())
}

fn handle_menu_input(
    key: crossterm::event::KeyEvent,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    generation_tx: &mpsc::Sender<Result<String, String>>,
) {
    if app.status_message.is_some() && !app.show_help {
        match key.code {
            KeyCode::Enter
            | KeyCode::Esc
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Tab
            | KeyCode::Backspace
            | KeyCode::Char(_) => {
                app.clear_status_message();
            }
            _ => {}
        }
        return;
    }

    if key.code == KeyCode::Char('h') {
        app.toggle_help();
        return;
    }

    if app.show_help {
        match key.code {
            KeyCode::Up => app.scroll_help_up(),
            KeyCode::Down => {
                let max_scroll = render::help_line_count().saturating_sub(5);
                app.scroll_help_down(max_scroll);
            }
            KeyCode::Esc => app.show_help = false,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Up => app.move_menu_up(),
        KeyCode::Down => app.move_menu_down(),
        KeyCode::Enter => match app.menu_selected {
            MenuItem::StartGame => {
                *timer.lock().unwrap() = 0;
                app.enter_loading();
                let sender = generation_tx.clone();
                let source = app.generation_source;
                let level = app.level;
                let provider = match source {
                    usecase::generate_sentence::GenerationSource::Google => {
                        Some(app.config.google.clone())
                    }
                    usecase::generate_sentence::GenerationSource::Groq => {
                        Some(app.config.groq.clone())
                    }
                    usecase::generate_sentence::GenerationSource::Local => None,
                };

                thread::spawn(move || {
                    let result = SentenceHandler::print_sentence(level, source, provider)
                        .map_err(|err| err.to_string());
                    sender.send(result).ok();
                });
            }
            MenuItem::Config => {
                app.clear_status_message();
                app.open_config();
            }
        },
        KeyCode::Esc => app.quit(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        _ => {}
    }
}

fn handle_config_input(key: crossterm::event::KeyEvent, app: &mut App) -> io::Result<()> {
    match key.code {
        KeyCode::Up => app.move_config_up(),
        KeyCode::Down | KeyCode::Tab => app.move_config_down(),
        KeyCode::Backspace => app.pop_config_char(),
        KeyCode::Enter => {
            config::save_config(&app.config)?;
            app.return_to_menu();
            app.select_start_game();
            app.set_status_message("Configuration saved");
        }
        KeyCode::Esc => {
            app.return_to_menu();
            app.set_status_message("Configuration changes discarded");
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Char(ch) => {
            if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                app.edit_config_char(ch);
            }
        }
        _ => {}
    }

    Ok(())
}

fn handle_loading_input(key: crossterm::event::KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            app.return_to_menu();
            app.set_status_message("Generation continues in background until completion");
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        _ => {}
    }
}

fn handle_typing_input(
    key: crossterm::event::KeyEvent,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    audio_sink: &MixerDeviceSink,
    timer_stop_tx: &mpsc::Sender<()>,
) {
    match key.code {
        KeyCode::Esc => {
            timer_stop_tx.send(()).ok();
            app.update_timer(*timer.lock().unwrap());
            app.finish_typing();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            timer_stop_tx.send(()).ok();
            app.quit();
        }
        KeyCode::Backspace => {
            app.pop_char();
        }
        KeyCode::Char(c) => {
            let is_correct = app.push_char(c);

            if is_correct && app.sound_enabled {
                let source = SineWave::new(app.freq).take_duration(Duration::from_millis(100));
                audio_sink.mixer().add(source);
            }

            if app.is_complete() {
                timer_stop_tx.send(()).ok();
                app.update_timer(*timer.lock().unwrap());
                app.finish_typing();
            }
        }
        _ => {}
    }
}
