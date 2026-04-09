use super::{App, AppState};

impl App {
    pub fn start_typing(&mut self) {
        self.state = AppState::Typing;
        self.time_started = true;
    }

    pub fn finish_typing(&mut self) {
        self.state = AppState::Result;
    }

    pub fn prepare_new_game(&mut self, target: String) {
        self.target_string = target;
        self.inputs.clear();
        self.typed_count = 0;
        self.incorrects = 0;
        self.timer = 0;
        self.time_started = false;
    }

    pub fn update_timer(&mut self, elapsed: i32) {
        self.timer = elapsed;
    }

    pub fn push_char(&mut self, c: char) -> bool {
        let position = self.inputs.len();
        let is_correct = self.target_string.chars().nth(position) == Some(c);
        self.typed_count += 1;

        if !is_correct {
            self.incorrects += 1;
        }

        self.inputs.push(c);
        is_correct
    }

    pub fn pop_char(&mut self) -> Option<char> {
        self.inputs.pop()
    }

    pub fn is_complete(&self) -> bool {
        self.inputs.len() >= self.target_string.len()
    }

    pub fn typed_count(&self) -> usize {
        self.typed_count
    }

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
}
