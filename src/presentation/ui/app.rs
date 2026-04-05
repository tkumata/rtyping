use crate::config::AppConfig;
use crate::usecase::generate_sentence::GenerationSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Config,
    Loading,
    Typing,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    StartGame,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    GoogleApiUrl,
    GoogleApiKey,
    GoogleModel,
    GroqApiUrl,
    GroqApiKey,
    GroqModel,
}

impl ConfigField {
    pub const ALL: [ConfigField; 6] = [
        ConfigField::GoogleApiUrl,
        ConfigField::GoogleApiKey,
        ConfigField::GoogleModel,
        ConfigField::GroqApiUrl,
        ConfigField::GroqApiKey,
        ConfigField::GroqModel,
    ];
}

pub struct App {
    pub state: AppState,
    pub target_string: String,
    pub inputs: Vec<char>,
    pub incorrects: usize,
    pub timer: i32,
    pub timeout: i32,
    pub text_scale: usize,
    pub should_quit: bool,
    pub freq: f32,
    pub sound_enabled: bool,
    pub time_started: bool,
    pub show_help: bool,
    pub help_scroll: u16,
    pub menu_selected: MenuItem,
    pub config_field: ConfigField,
    pub config: AppConfig,
    pub status_message: Option<String>,
    pub generation_source: GenerationSource,
}

impl App {
    pub fn new(
        timeout: i32,
        text_scale: usize,
        freq: f32,
        sound_enabled: bool,
        generation_source: GenerationSource,
        config: AppConfig,
    ) -> Self {
        Self {
            state: AppState::Menu,
            target_string: String::new(),
            inputs: Vec::new(),
            incorrects: 0,
            timer: 0,
            timeout,
            text_scale,
            should_quit: false,
            freq,
            sound_enabled,
            time_started: false,
            show_help: false,
            help_scroll: 0,
            menu_selected: MenuItem::StartGame,
            config_field: ConfigField::GoogleApiUrl,
            config,
            status_message: None,
            generation_source,
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

    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    pub fn enter_loading(&mut self) {
        self.state = AppState::Loading;
        self.clear_status_message();
    }

    pub fn open_config(&mut self) {
        self.state = AppState::Config;
    }

    pub fn return_to_menu(&mut self) {
        self.state = AppState::Menu;
        self.show_help = false;
    }

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

    pub fn move_menu_up(&mut self) {
        self.menu_selected = match self.menu_selected {
            MenuItem::StartGame => MenuItem::Config,
            MenuItem::Config => MenuItem::StartGame,
        };
    }

    pub fn move_menu_down(&mut self) {
        self.menu_selected = match self.menu_selected {
            MenuItem::StartGame => MenuItem::Config,
            MenuItem::Config => MenuItem::StartGame,
        };
    }

    pub fn select_start_game(&mut self) {
        self.menu_selected = MenuItem::StartGame;
    }

    pub fn move_config_up(&mut self) {
        let idx = self.config_field_index();
        let next = if idx == 0 {
            ConfigField::ALL.len() - 1
        } else {
            idx - 1
        };
        self.config_field = ConfigField::ALL[next];
    }

    pub fn move_config_down(&mut self) {
        let idx = self.config_field_index();
        let next = (idx + 1) % ConfigField::ALL.len();
        self.config_field = ConfigField::ALL[next];
    }

    pub fn edit_config_char(&mut self, ch: char) {
        self.selected_config_field_mut().push(ch);
    }

    pub fn pop_config_char(&mut self) {
        self.selected_config_field_mut().pop();
    }

    fn selected_config_field_mut(&mut self) -> &mut String {
        match self.config_field {
            ConfigField::GoogleApiUrl => &mut self.config.google.api_url,
            ConfigField::GoogleApiKey => &mut self.config.google.api_key,
            ConfigField::GoogleModel => &mut self.config.google.model,
            ConfigField::GroqApiUrl => &mut self.config.groq.api_url,
            ConfigField::GroqApiKey => &mut self.config.groq.api_key,
            ConfigField::GroqModel => &mut self.config.groq.model,
        }
    }

    fn config_field_index(&self) -> usize {
        ConfigField::ALL
            .iter()
            .position(|field| *field == self.config_field)
            .expect("ConfigField::ALL must contain all variants")
    }
}
