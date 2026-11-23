use crossterm::{
    cursor::SetCursorStyle,
    event::{self, Event},
    execute,
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::io;
use std::time::{Duration, Instant};

use crate::config;
use crate::presentation::audio::Audio;
use crate::presentation::ui::app::{App, AppState};
use crate::presentation::ui::render;

use super::input::{handle_key_event, receive_generation_result};

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    audio: &mut Audio,
) -> io::Result<()> {
    let mut job = None;
    let mut using_typing_cursor_style = false;
    let mut started_at = None;
    let mut previous_screen = app.state();

    while !app.is_quit_requested() {
        receive_generation_result(app, &mut job);
        if started_at.is_none() && matches!(app.state(), AppState::Typing | AppState::RhythmTyping)
        {
            audio.start_round();
        }
        tick_game(app, &mut started_at, Instant::now());

        let is_typing = app.state() == AppState::Typing;
        if is_typing && !using_typing_cursor_style {
            execute!(terminal.backend_mut(), SetCursorStyle::SteadyBar)?;
            using_typing_cursor_style = true;
        } else if !is_typing && using_typing_cursor_style {
            execute!(terminal.backend_mut(), SetCursorStyle::DefaultUserShape)?;
            using_typing_cursor_style = false;
        }
        draw_screen(terminal, app, &mut previous_screen)?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if tick_game(app, &mut started_at, Instant::now()) {
                continue;
            }
            handle_key_event(key, app, &mut job, audio);
            if (app.state() == AppState::Typing && app.is_complete())
                || (app.state() == AppState::RhythmTyping && app.is_rhythm_complete())
            {
                finish_game(app);
            }
            if !matches!(app.state(), AppState::Typing | AppState::RhythmTyping) {
                started_at = None;
            }
        }
    }
    Ok(())
}

fn draw_screen<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &App,
    previous_screen: &mut AppState,
) -> Result<(), B::Error> {
    let screen = app.state();
    if screen != *previous_screen {
        // Clear both the fullscreen display and the previous frame used for diffing.
        terminal.backend_mut().clear()?;
        terminal.swap_buffers();
    }
    terminal.draw(|frame| render::render(frame, app))?;
    *previous_screen = screen;
    Ok(())
}

fn tick_game(app: &mut App, started_at: &mut Option<Instant>, now: Instant) -> bool {
    if update_game_time(app, started_at, now) {
        finish_game(app);
        *started_at = None;
        return true;
    }
    false
}

fn update_game_time(app: &mut App, started_at: &mut Option<Instant>, now: Instant) -> bool {
    match app.state() {
        AppState::Typing => {
            let elapsed = now.duration_since(*started_at.get_or_insert(now));
            let seconds = i32::try_from(elapsed.as_secs()).unwrap_or(i32::MAX);
            let timeout = app.timeout();
            if timeout > 0 && seconds >= timeout {
                app.update_timer(timeout);
                true
            } else {
                app.update_timer(seconds);
                false
            }
        }
        AppState::RhythmTyping => {
            let elapsed = now.duration_since(*started_at.get_or_insert(now));
            app.update_rhythm_elapsed_seconds(elapsed.as_secs_f64());
            app.is_rhythm_complete()
        }
        _ => {
            *started_at = None;
            false
        }
    }
}

fn finish_game(app: &mut App) {
    if app.state() == AppState::Typing
        && let Some(entry) = app.build_history_entry()
    {
        match config::append_history(entry) {
            Ok(entries) => app.set_history_entries(entries),
            Err(err) => app.set_status_message(format!("Failed to save history: {err}")),
        }
    }
    app.finish_typing();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{AppConfig, GameSettings};
    use ratatui::{backend::TestBackend, buffer::Cell, style::Color};

    #[test]
    fn screen_transition_erases_background_left_in_terminal() {
        let mut terminal = Terminal::new(TestBackend::new(100, 34)).expect("create test terminal");
        let mut app = App::new(AppConfig::default());
        let mut previous_screen = app.state();
        draw_screen(&mut terminal, &app, &mut previous_screen).expect("draw title screen");
        app.open_stats();
        draw_screen(&mut terminal, &app, &mut previous_screen).expect("draw stats screen");

        let mut stale_cell = Cell::EMPTY;
        stale_cell.bg = Color::Red;
        terminal
            .backend_mut()
            .draw(std::iter::once((0, 0, &stale_cell)))
            .expect("simulate stale background in terminal");

        draw_screen(&mut terminal, &app, &mut previous_screen).expect("redraw stats screen");
        assert_eq!(
            terminal.backend().buffer().cell((0, 0)).map(|cell| cell.bg),
            Some(Color::Red)
        );

        app.return_to_menu();
        draw_screen(&mut terminal, &app, &mut previous_screen).expect("return to title screen");
        assert_eq!(
            terminal.backend().buffer().cell((0, 0)).map(|cell| cell.bg),
            Some(Color::Reset)
        );
    }

    #[test]
    fn each_round_gets_a_new_deadline() {
        let mut app = App::new(AppConfig {
            game: GameSettings {
                timeout: "1".into(),
                ..GameSettings::default()
            },
            ..AppConfig::default()
        });
        let mut started_at = None;
        let start = Instant::now();
        app.prepare_new_game("ab".into());
        app.start_typing();
        assert!(!update_game_time(&mut app, &mut started_at, start));
        assert!(!update_game_time(
            &mut app,
            &mut started_at,
            start + Duration::from_millis(999)
        ));
        assert!(update_game_time(
            &mut app,
            &mut started_at,
            start + Duration::from_secs(1)
        ));
        assert_eq!(app.timer(), 1);

        app.return_to_menu();
        let restart = start + Duration::from_secs(3);
        assert!(!update_game_time(&mut app, &mut started_at, restart));
        app.prepare_new_game("ab".into());
        app.start_typing();
        assert!(!update_game_time(&mut app, &mut started_at, restart));
        assert_eq!(app.timer(), 0);
    }

    #[test]
    fn practice_clock_advances_without_a_deadline() {
        let mut app = App::new(AppConfig::default());
        let mut started_at = None;
        let start = Instant::now();
        app.set_practice_mode(true);
        app.start_typing();
        assert!(!update_game_time(&mut app, &mut started_at, start));
        assert!(!update_game_time(
            &mut app,
            &mut started_at,
            start + Duration::from_secs(90)
        ));
        assert_eq!(app.timer(), 90);
        assert_eq!(app.state(), AppState::Typing);
    }
}
