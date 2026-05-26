mod config_editor;
mod menu;
mod typing;

use crate::domain::config::AppConfig;
use crate::domain::history::{HistoryEntry, HistoryMode};
use crate::domain::rhythm::{RhythmJudgement, RhythmSession, RhythmStats};
use crate::usecase::accuracy;
use crate::usecase::generate_sentence::GenerationSource;
use crate::usecase::history_stats::{self, HistoryStats};
use crate::usecase::wpm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Config,
    Stats,
    Loading,
    Typing,
    RhythmTyping,
    Result,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    StartGame,
    PracticeMode,
    StartGameRhythm,
    StartGameGoogle,
    StartGameGroq,
    Stats,
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
    GameTimeout,
    GameTextScale,
    GameRhythmSpeed,
    GameFreq,
    GameSoundEnabled,
}

impl ConfigField {
    pub const ALL: [ConfigField; 11] = [
        ConfigField::GoogleApiUrl,
        ConfigField::GoogleApiKey,
        ConfigField::GoogleModel,
        ConfigField::GroqApiUrl,
        ConfigField::GroqApiKey,
        ConfigField::GroqModel,
        ConfigField::GameTimeout,
        ConfigField::GameTextScale,
        ConfigField::GameRhythmSpeed,
        ConfigField::GameFreq,
        ConfigField::GameSoundEnabled,
    ];

    pub fn accepts_text(self) -> bool {
        self != ConfigField::GameSoundEnabled
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Standard,
    Rhythm,
}

#[expect(clippy::struct_excessive_bools)]
pub struct App {
    state: AppState,
    target_string: String,
    inputs: Vec<char>,
    typed_count: usize,
    incorrects: usize,
    missed_chars: Vec<char>,
    wpm_history: Vec<u64>,
    wpm_activity_revision: u64,
    last_wpm_activity_timer: Option<i32>,
    last_wpm_sample: Option<(i32, usize, usize, u64)>,
    timer: i32,
    practice_mode: bool,
    should_quit: bool,
    time_started: bool,
    show_help: bool,
    help_scroll: u16,
    menu_selected: MenuItem,
    config_field: ConfigField,
    config_cursor_index: usize,
    config: AppConfig,
    status_message: Option<String>,
    generation_source: GenerationSource,
    history_entries: Vec<HistoryEntry>,
    next_game_mode: GameMode,
    active_game_mode: GameMode,
    rhythm_session: Option<RhythmSession>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            state: AppState::Menu,
            target_string: String::new(),
            inputs: Vec::new(),
            typed_count: 0,
            incorrects: 0,
            missed_chars: Vec::new(),
            wpm_history: Vec::new(),
            wpm_activity_revision: 0,
            last_wpm_activity_timer: None,
            last_wpm_sample: None,
            timer: 0,
            practice_mode: false,
            should_quit: false,
            time_started: false,
            show_help: false,
            help_scroll: 0,
            menu_selected: MenuItem::StartGame,
            config_field: ConfigField::GoogleApiUrl,
            config_cursor_index: 0,
            config,
            status_message: None,
            generation_source: GenerationSource::Local,
            history_entries: Vec::new(),
            next_game_mode: GameMode::Standard,
            active_game_mode: GameMode::Standard,
            rhythm_session: None,
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
        self.move_config_cursor_to_end();
    }

    pub fn open_stats(&mut self) {
        self.state = AppState::Stats;
        self.clear_status_message();
    }

    pub fn return_to_menu(&mut self) {
        self.state = AppState::Menu;
        self.hide_help();
    }

    pub fn return_to_menu_with_start_selected(&mut self) {
        self.return_to_menu();
        self.set_practice_mode(false);
        self.set_next_game_mode(GameMode::Standard);
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

    #[cfg(test)]
    pub fn missed_chars(&self) -> &[char] {
        &self.missed_chars
    }

    pub fn wpm_history(&self) -> &[u64] {
        &self.wpm_history
    }

    pub fn timer(&self) -> i32 {
        self.timer
    }

    pub fn timeout(&self) -> i32 {
        if self.practice_mode {
            0
        } else {
            self.config.game.timeout_value()
        }
    }

    pub fn set_practice_mode(&mut self, practice_mode: bool) {
        self.practice_mode = practice_mode;
    }

    pub fn is_practice_mode(&self) -> bool {
        self.practice_mode
    }

    pub fn is_quit_requested(&self) -> bool {
        self.should_quit
    }

    pub fn typing_sound_enabled(&self) -> bool {
        self.config.game.sound_enabled_value()
    }

    pub fn frequency(&self) -> f32 {
        self.config.game.freq_value()
    }

    pub fn rhythm_speed(&self) -> u8 {
        self.config.game.rhythm_speed_value()
    }

    pub fn generation_source(&self) -> GenerationSource {
        self.generation_source
    }

    pub fn set_generation_source(&mut self, source: GenerationSource) {
        self.generation_source = source;
    }

    pub fn next_game_mode(&self) -> GameMode {
        self.next_game_mode
    }

    pub fn set_next_game_mode(&mut self, mode: GameMode) {
        self.next_game_mode = mode;
    }

    pub fn is_rhythm_result(&self) -> bool {
        self.state == AppState::Result && self.active_game_mode == GameMode::Rhythm
    }

    pub fn rhythm_stats(&self) -> Option<RhythmStats> {
        self.rhythm_session.as_ref().map(RhythmSession::stats)
    }

    pub fn rhythm_visible_chars(&self, width: u16) -> Vec<(u16, char)> {
        self.rhythm_session
            .as_ref()
            .map_or_else(Vec::new, |session| session.visible_chars(width))
    }

    pub fn push_rhythm_char(&mut self, ch: char) -> RhythmJudgement {
        self.rhythm_session
            .as_mut()
            .map_or(RhythmJudgement::Miss, |session| session.push_char(ch))
    }

    pub fn rhythm_last_judgement(&self) -> Option<RhythmJudgement> {
        self.rhythm_session
            .as_ref()
            .and_then(RhythmSession::last_judgement)
    }

    pub fn rhythm_combo(&self) -> usize {
        self.rhythm_session.as_ref().map_or(0, RhythmSession::combo)
    }

    pub fn update_rhythm_elapsed_seconds(&mut self, elapsed_seconds: f64) {
        if let Some(session) = &mut self.rhythm_session {
            session.set_elapsed_seconds(elapsed_seconds);
        }
    }

    pub fn is_rhythm_complete(&self) -> bool {
        self.rhythm_session
            .as_ref()
            .is_some_and(RhythmSession::is_complete)
    }

    pub fn generation_settings(&self) -> (usize, GenerationSource, AppConfig) {
        (
            self.config.game.text_scale_value(),
            self.generation_source,
            self.config.clone(),
        )
    }

    pub fn set_history_entries(&mut self, entries: Vec<HistoryEntry>) {
        self.history_entries = entries;
    }

    pub fn history_entries(&self) -> &[HistoryEntry] {
        &self.history_entries
    }

    pub fn history_stats(&self) -> HistoryStats {
        history_stats::summarize(&self.history_entries)
    }

    pub fn build_history_entry(&self) -> Option<HistoryEntry> {
        if self.practice_mode {
            return None;
        }

        let elapsed = self.timer.max(1);
        Some(HistoryEntry {
            wpm: wpm::calc_wpm(
                self.typed_count(),
                elapsed,
                i32::try_from(self.incorrects()).unwrap_or(i32::MAX),
            ),
            accuracy: accuracy::calc_accuracy(self.typed_count(), self.incorrects()),
            miss_count: self.incorrects(),
            elapsed_seconds: elapsed,
            generation_source: self.generation_source.label().into(),
            mode: HistoryMode::Timed,
            missed_chars: self.missed_chars.clone(),
        })
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

    pub fn config_cursor_index(&self) -> usize {
        self.config_cursor_index
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
            crate::usecase::wpm::calc_wpm(
                self.typed_count,
                self.timer,
                i32::try_from(self.incorrects).unwrap_or(i32::MAX),
            )
            .max(0.0)
        }
    }

    pub fn record_wpm_snapshot(&mut self) {
        const MAX_WPM_HISTORY: usize = 120;
        const WPM_IDLE_GRACE_SECONDS: i32 = 2;

        let sample_key = (
            self.timer,
            self.typed_count,
            self.incorrects,
            self.wpm_activity_revision,
        );
        if self.last_wpm_sample == Some(sample_key) {
            return;
        }

        let should_zero = self
            .last_wpm_activity_timer
            .is_some_and(|last_activity_timer| {
                self.timer.saturating_sub(last_activity_timer) >= WPM_IDLE_GRACE_SECONDS
            });
        let sample = if should_zero {
            0
        } else {
            rounded_wpm_sample(self.current_wpm())
        };
        self.wpm_history.push(sample);
        if self.wpm_history.len() > MAX_WPM_HISTORY {
            let overflow = self.wpm_history.len() - MAX_WPM_HISTORY;
            self.wpm_history.drain(0..overflow);
        }
        self.last_wpm_sample = Some(sample_key);
    }
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rounded_wpm_sample(current_wpm: f64) -> u64 {
    current_wpm.round().clamp(0.0, f64::from(u32::MAX)) as u64
}
