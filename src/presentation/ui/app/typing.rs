use crate::domain::rhythm::RhythmSession;

use super::{App, AppState, GameMode};

impl App {
    pub fn start_typing(&mut self) {
        self.state = AppState::Typing;
        self.active_game_mode = GameMode::Standard;
        self.time_started = true;
        self.record_wpm_snapshot();
    }

    pub fn start_rhythm_typing(&mut self) {
        self.state = AppState::RhythmTyping;
        self.active_game_mode = GameMode::Rhythm;
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
        self.missed_chars.clear();
        self.wpm_history.clear();
        self.wpm_activity_revision = 0;
        self.last_wpm_activity_timer = None;
        self.last_wpm_sample = None;
        self.timer = 0;
        self.time_started = false;
        self.active_game_mode = GameMode::Standard;
        self.rhythm_session = None;
    }

    pub fn prepare_rhythm_game(&mut self, target: &str) {
        self.prepare_new_game(target.to_string());
        self.active_game_mode = GameMode::Rhythm;
        self.rhythm_session = Some(RhythmSession::new(target, self.rhythm_speed()));
    }

    pub fn update_timer(&mut self, elapsed: i32) {
        self.timer = elapsed;
        self.record_wpm_snapshot();
    }

    pub fn push_char(&mut self, c: char) -> bool {
        let position = self.inputs.len();
        let expected_char = self.target_string.chars().nth(position);
        let is_correct = expected_char == Some(c);
        self.typed_count += 1;
        self.wpm_activity_revision += 1;
        self.last_wpm_activity_timer = Some(self.timer);

        if is_correct || !self.practice_mode {
            self.inputs.push(c);
        }

        if !is_correct {
            self.incorrects += 1;
            if let Some(expected_char) = expected_char {
                self.missed_chars.push(expected_char);
            }
        }
        self.record_wpm_snapshot();
        is_correct
    }

    pub fn pop_char(&mut self) -> Option<char> {
        let removed = self.inputs.pop();
        self.wpm_activity_revision += 1;
        self.last_wpm_activity_timer = Some(self.timer);
        self.record_wpm_snapshot();
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

    fn new_app() -> App {
        App::new(AppConfig::default())
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
    fn practice_mode_does_not_store_incorrect_input() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.set_practice_mode(true);

        assert!(!app.push_char('x'));
        assert!(app.input_chars().is_empty());
        assert_eq!(app.typed_count(), 1);
        assert_eq!(app.incorrects(), 1);
        assert!(app.pop_char().is_none());
    }

    #[test]
    fn normal_mode_stores_incorrect_input_and_advances() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());

        assert!(!app.push_char('x'));
        assert_eq!(app.input_chars(), &['x']);
        assert_eq!(app.missed_chars(), &['a']);
        assert_eq!(app.current_input_count(), 1);
        assert_eq!(app.typed_count(), 1);
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
        assert!(app.missed_chars().is_empty());
    }

    #[test]
    fn build_history_entry_skips_practice_mode() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.set_practice_mode(true);
        app.start_typing();
        app.update_timer(1);
        app.push_char('a');

        assert!(app.build_history_entry().is_none());
    }

    #[test]
    fn build_history_entry_captures_timed_result() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.start_typing();
        app.update_timer(1);
        app.push_char('x');

        let entry = app.build_history_entry();
        assert!(entry.is_some());
        let entry = entry.unwrap_or(crate::domain::history::HistoryEntry {
            wpm: 0.0,
            accuracy: 0.0,
            miss_count: 0,
            elapsed_seconds: 0,
            generation_source: String::new(),
            mode: crate::domain::history::HistoryMode::Timed,
            missed_chars: Vec::new(),
        });

        assert_eq!(entry.miss_count, 1);
        assert_eq!(entry.elapsed_seconds, 1);
        assert_eq!(entry.generation_source, "Local");
        assert_eq!(entry.missed_chars, vec!['a']);
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

    #[test]
    fn wpm_history_keeps_wpm_during_idle_grace_and_then_goes_zero() {
        let mut app = new_app();
        app.prepare_new_game("ab".to_string());
        app.start_typing();
        app.update_timer(1);
        app.push_char('a');
        app.update_timer(2);
        app.update_timer(3);
        app.update_timer(4);

        assert_eq!(app.wpm_history(), &[0, 0, 12, 6, 0, 0]);
    }
}
