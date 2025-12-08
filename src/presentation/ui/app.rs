#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Intro,
    Typing,
    Result,
}

pub struct App {
    pub state: AppState,
    pub target_string: String,
    pub inputs: Vec<char>,
    pub incorrects: usize,
    pub timer: i32,
    pub timeout: i32,
    pub level: usize,
    pub should_quit: bool,
    pub freq: f32,
    pub sound_enabled: bool,
    pub time_started: bool,
    pub show_help: bool,
    pub help_scroll: u16,
}

impl App {
    pub fn new(timeout: i32, level: usize, freq: f32, sound_enabled: bool) -> Self {
        Self {
            state: AppState::Intro,
            target_string: String::new(),
            inputs: Vec::new(),
            incorrects: 0,
            timer: 0,
            timeout,
            level,
            should_quit: false,
            freq,
            sound_enabled,
            time_started: false,
            show_help: false,
            help_scroll: 0,
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if !self.show_help {
            self.help_scroll = 0;
        }
    }

    pub fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    pub fn scroll_help_down(&mut self, max_scroll: u16) {
        if self.help_scroll < max_scroll {
            self.help_scroll += 1;
        }
    }

    pub fn set_target_string(&mut self, target: String) {
        self.target_string = target;
    }

    pub fn start_typing(&mut self) {
        self.state = AppState::Typing;
        self.time_started = true;
    }

    pub fn finish_typing(&mut self) {
        self.state = AppState::Result;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
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
