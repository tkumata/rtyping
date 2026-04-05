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
        self.inputs.len()
    }
}
