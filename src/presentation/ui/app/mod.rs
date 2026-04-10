mod config_editor;
mod menu;
mod typing;

use crate::domain::config::AppConfig;
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
    PracticeMode,
    StartGameGoogle,
    StartGameGroq,
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
    state: AppState,
    target_string: String,
    inputs: Vec<char>,
    typed_count: usize,
    incorrects: usize,
    wpm_history: Vec<u64>,
    last_wpm_sample: Option<(i32, usize, usize)>,
    timer: i32,
    timeout: i32,
    practice_mode: bool,
    text_scale: usize,
    should_quit: bool,
    freq: f32,
    sound_enabled: bool,
    time_started: bool,
    show_help: bool,
    help_scroll: u16,
    menu_selected: MenuItem,
    config_field: ConfigField,
    config: AppConfig,
    status_message: Option<String>,
    generation_source: GenerationSource,
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
            typed_count: 0,
            incorrects: 0,
            wpm_history: Vec::new(),
            last_wpm_sample: None,
            timer: 0,
            timeout,
            practice_mode: false,
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
        self.hide_help();
    }

    pub fn return_to_menu_with_start_selected(&mut self) {
        self.return_to_menu();
        self.set_practice_mode(false);
        self.select_start_game();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn target_string(&self) -> &str {
        &self.target_string
    }

    pub fn input_chars(&self) -> &[char] {
        &self.inputs
    }

    pub fn incorrects(&self) -> usize {
        self.incorrects
    }

    pub fn wpm_history(&self) -> &[u64] {
        &self.wpm_history
    }

    pub fn timer(&self) -> i32 {
        self.timer
    }

    pub fn timeout(&self) -> i32 {
        if self.practice_mode { 0 } else { self.timeout }
    }

    pub fn set_practice_mode(&mut self, practice_mode: bool) {
        self.practice_mode = practice_mode;
    }

    #[cfg(test)]
    pub fn is_practice_mode(&self) -> bool {
        self.practice_mode
    }

    pub fn is_quit_requested(&self) -> bool {
        self.should_quit
    }

    pub fn typing_sound_enabled(&self) -> bool {
        self.sound_enabled
    }

    pub fn frequency(&self) -> f32 {
        self.freq
    }

    pub fn generation_source(&self) -> GenerationSource {
        self.generation_source
    }

    pub fn set_generation_source(&mut self, source: GenerationSource) {
        self.generation_source = source;
    }

    pub fn generation_settings(&self) -> (usize, GenerationSource, AppConfig) {
        (self.text_scale, self.generation_source, self.config.clone())
    }

    pub fn is_help_visible(&self) -> bool {
        self.show_help
    }

    pub fn hide_help(&mut self) {
        self.show_help = false;
        self.help_scroll = 0;
    }

    pub fn help_scroll(&self) -> u16 {
        self.help_scroll
    }

    pub fn menu_selected(&self) -> MenuItem {
        self.menu_selected
    }

    pub fn config_field(&self) -> ConfigField {
        self.config_field
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn current_wpm(&self) -> f64 {
        if self.timer <= 0 {
            0.0
        } else {
            crate::usecase::wpm::calc_wpm(self.typed_count, self.timer, self.incorrects as i32)
                .max(0.0)
        }
    }

    pub fn record_wpm_snapshot(&mut self) {
        const MAX_WPM_HISTORY: usize = 120;

        let sample_key = (self.timer, self.typed_count, self.incorrects);
        if self.last_wpm_sample == Some(sample_key) {
            return;
        }

        let sample = self.current_wpm().round().clamp(0.0, u64::MAX as f64) as u64;
        self.wpm_history.push(sample);
        if self.wpm_history.len() > MAX_WPM_HISTORY {
            let overflow = self.wpm_history.len() - MAX_WPM_HISTORY;
            self.wpm_history.drain(0..overflow);
        }
        self.last_wpm_sample = Some(sample_key);
    }
}
