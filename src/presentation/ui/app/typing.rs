use super::{App, AppState};

impl App {
    pub fn start_typing(&mut self) {
        self.state = AppState::Typing;
        self.time_started = true;
        self.record_wpm_snapshot();
    }

    pub fn finish_typing(&mut self) {
        self.state = AppState::Result;
    }

    pub fn prepare_new_game(&mut self, target: String) {
        self.target_string = target;
        self.inputs.clear();
        self.typed_count = 0;
        self.incorrects = 0;
        self.wpm_history.clear();
        self.last_wpm_sample = None;
        self.timer = 0;
        self.time_started = false;
    }

    pub fn update_timer(&mut self, elapsed: i32) {
        self.timer = elapsed;
        self.record_wpm_snapshot();
    }

    pub fn push_char(&mut self, c: char) -> bool {
        let position = self.inputs.len();
        let is_correct = self.target_string.chars().nth(position) == Some(c);
        self.typed_count += 1;

        if is_correct {
            self.inputs.push(c);
        } else {
            self.incorrects += 1;
        }
        self.record_wpm_snapshot();
        is_correct
    }

    pub fn pop_char(&mut self) -> Option<char> {
        let removed = self.inputs.pop();
        if removed.is_some() {
            self.record_wpm_snapshot();
        }
        removed
    }

    pub fn is_complete(&self) -> bool {
        self.inputs.len() >= self.target_string.len()
    }

    pub fn typed_count(&self) -> usize {
        self.typed_count
    }

    #[cfg(test)]
    pub fn current_input_count(&self) -> usize {
        self.inputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::domain::config::AppConfig;
    use crate::usecase::generate_sentence::GenerationSource;

    fn new_app() -> App {
        App::new(
            60,
            30,
            80.0,
            false,
            GenerationSource::Local,
            AppConfig::default(),
        )
    }

    #[test]
    fn typed_count_tracks_total_keypresses_after_backspace() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());

        app.push_char('x');
        app.pop_char();
        app.push_char('a');

        assert_eq!(app.input_chars(), &['a']);
        assert_eq!(app.typed_count(), 2);
        assert_eq!(app.incorrects(), 1);
    }

    #[test]
    fn strict_mode_does_not_store_incorrect_input() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());

        assert!(!app.push_char('x'));
        assert!(app.input_chars().is_empty());
        assert_eq!(app.typed_count(), 1);
        assert_eq!(app.incorrects(), 1);
        assert!(app.pop_char().is_none());
    }

    #[test]
    fn prepare_new_game_resets_total_typed_count() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.push_char('a');
        app.push_char('b');

        app.prepare_new_game("cd".to_string());

        assert_eq!(app.typed_count(), 0);
        assert!(app.input_chars().is_empty());
        assert_eq!(app.incorrects(), 0);
    }

    #[test]
    fn prepare_new_game_resets_wpm_history() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.start_typing();
        app.update_timer(1);
        app.push_char('a');

        assert!(!app.wpm_history().is_empty());

        app.prepare_new_game("cd".to_string());

        assert!(app.wpm_history().is_empty());
    }

    #[test]
    fn wpm_history_avoids_duplicate_samples() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.start_typing();

        assert_eq!(app.wpm_history(), &[0]);

        app.update_timer(0);
        assert_eq!(app.wpm_history(), &[0]);

        app.update_timer(1);
        assert_eq!(app.wpm_history().len(), 2);

        app.update_timer(1);
        assert_eq!(app.wpm_history().len(), 2);
    }
}
