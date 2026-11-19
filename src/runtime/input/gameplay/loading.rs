use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::presentation::ui::app::App;
use crate::runtime::GenerationJob;

pub(in crate::runtime::input) fn handle_loading_input(
    key: KeyEvent,
    app: &mut App,
    job: &mut Option<GenerationJob>,
) {
    match key.code {
        KeyCode::Esc => {
            *job = None;
            app.return_to_menu();
            app.set_status_message("Generation canceled");
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::AppConfig;
    use crate::presentation::ui::app::App;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn escape_on_loading_cancels_request() {
        let mut app = App::new(AppConfig::default());
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut job = Some(receiver);

        app.enter_loading();
        handle_loading_input(key(KeyCode::Esc), &mut app, &mut job);

        assert_eq!(app.state(), crate::presentation::ui::app::AppState::Menu);
        assert_eq!(app.status_message(), Some("Generation canceled"));
        assert!(job.is_none());
        assert!(sender.send(Ok("stale".into())).is_err());
    }
}
