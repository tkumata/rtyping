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
use rodio::{OutputStreamBuilder, Source, source::SineWave};
use std::io::{self, stdout};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use presentation::bgm_handler::BgmHandler;
use presentation::sentence_handler::SentenceHandler;
use presentation::ui::app::{App, AppState};
use presentation::ui::render;
use presentation::ui::ui_handler::UiHandler;

fn main() -> io::Result<()> {
    let args = UiHandler::parse_args();

    let stream = OutputStreamBuilder::open_default_stream().unwrap();

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

    let mut app = App::new(args.timeout, args.level, args.freq, args.sound);

    // Timer thread communication channels:
    // - timer_start: signals the timer thread to begin counting
    // - timer_stop: signals the timer thread to stop (on completion or early exit)
    // - timeout: notifies the main thread when time runs out
    let timer = Arc::new(Mutex::new(0i32));
    let timeout = args.timeout;

    let (timer_stop_tx, timer_stop_rx) = mpsc::channel::<()>();
    let (timeout_tx, timeout_rx) = mpsc::channel::<()>();
    let (timer_start_tx, timer_start_rx) = mpsc::channel::<()>();

    let timer_clone = Arc::clone(&timer);
    let timer_thread = thread::spawn(move || {
        // Wait for game start signal
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
        &stream,
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

/// Runs the main event loop, handling user input and screen transitions.
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    timer: &Arc<Mutex<i32>>,
    stream: &rodio::OutputStream,
    timer_stop_tx: mpsc::Sender<()>,
    timeout_rx: mpsc::Receiver<()>,
    timer_start_tx: mpsc::Sender<()>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| render::render(f, app))?;

        if app.should_quit {
            break;
        }

        // Check for timeout notification from timer thread
        if app.state == AppState::Typing && timeout_rx.try_recv().is_ok() {
            let current_timer = *timer.lock().unwrap();
            app.update_timer(current_timer);
            app.finish_typing();
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match app.state {
                AppState::Intro => {
                    if key.code == KeyCode::Char('h') {
                        app.toggle_help();
                    } else if app.show_help {
                        // ヘルプ表示中のキー操作
                        match key.code {
                            KeyCode::Up => app.scroll_help_up(),
                            KeyCode::Down => {
                                let max_scroll = render::help_line_count().saturating_sub(5);
                                app.scroll_help_down(max_scroll);
                            }
                            KeyCode::Esc => app.show_help = false,
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.quit();
                            }
                            _ => {}
                        }
                    } else if key.code == KeyCode::Enter {
                        match SentenceHandler::print_sentence(app.level) {
                            Ok(contents) => {
                                app.set_target_string(contents);
                                app.start_typing();
                                timer_start_tx.send(()).ok();
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    } else if key.code == KeyCode::Esc
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                    {
                        app.quit();
                    }
                }
                AppState::Typing => {
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

                            // Play feedback sound on correct input
                            if is_correct {
                                let source = SineWave::new(app.freq)
                                    .take_duration(Duration::from_millis(100));
                                stream.mixer().add(source);
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
