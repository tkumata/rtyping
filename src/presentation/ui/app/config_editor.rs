use super::{App, ConfigField};

impl App {
    pub fn move_config_up(&mut self) {
        let idx = self.config_field_index();
        let next = if idx == 0 {
            ConfigField::ALL.len() - 1
        } else {
            idx - 1
        };
        self.config_field = ConfigField::ALL
            .get(next)
            .copied()
            .unwrap_or(self.config_field);
        self.move_config_cursor_to_end();
    }

    pub fn move_config_down(&mut self) {
        let idx = self.config_field_index();
        let next = (idx + 1) % ConfigField::ALL.len();
        self.config_field = ConfigField::ALL
            .get(next)
            .copied()
            .unwrap_or(self.config_field);
        self.move_config_cursor_to_end();
    }

    pub fn move_config_cursor_left(&mut self) {
        if self.config_field.accepts_text() {
            self.config_cursor_index = self.config_cursor_index.saturating_sub(1);
        }
    }

    pub fn move_config_cursor_right(&mut self) {
        if self.config_field.accepts_text() {
            self.config_cursor_index = self
                .config_cursor_index
                .saturating_add(1)
                .min(self.selected_config_field_len());
        }
    }

    pub fn edit_config_char(&mut self, ch: char) {
        if self.config_field.accepts_text() {
            let cursor_index = self.clamped_config_cursor_index();
            let field = self.selected_config_field_mut();
            let byte_index = char_to_byte_index(field, cursor_index);
            field.insert(byte_index, ch);
            self.config_cursor_index = cursor_index.saturating_add(1);
        }
    }

    pub fn pop_config_char(&mut self) {
        if self.config_field.accepts_text() {
            let cursor_index = self.clamped_config_cursor_index();
            if cursor_index == 0 {
                return;
            }

            let field = self.selected_config_field_mut();
            let start = char_to_byte_index(field, cursor_index - 1);
            let end = char_to_byte_index(field, cursor_index);
            field.replace_range(start..end, "");
            self.config_cursor_index = cursor_index - 1;
        }
    }

    pub fn toggle_sound_enabled(&mut self) {
        self.config.game.toggle_sound();
    }

    fn selected_config_field_mut(&mut self) -> &mut String {
        match self.config_field {
            ConfigField::GoogleApiUrl => &mut self.config.google.api_url,
            ConfigField::GoogleApiKey => &mut self.config.google.api_key,
            ConfigField::GoogleModel => &mut self.config.google.model,
            ConfigField::GroqApiUrl => &mut self.config.groq.api_url,
            ConfigField::GroqApiKey => &mut self.config.groq.api_key,
            ConfigField::GroqModel => &mut self.config.groq.model,
            ConfigField::GameTimeout => &mut self.config.game.timeout,
            ConfigField::GameTextScale => &mut self.config.game.text_scale,
            ConfigField::GameRhythmSpeed => &mut self.config.game.rhythm_speed,
            ConfigField::GameFreq => &mut self.config.game.freq,
            ConfigField::GameSoundEnabled => &mut self.config.game.sound_enabled,
        }
    }

    fn selected_config_field(&self) -> &str {
        match self.config_field {
            ConfigField::GoogleApiUrl => &self.config.google.api_url,
            ConfigField::GoogleApiKey => &self.config.google.api_key,
            ConfigField::GoogleModel => &self.config.google.model,
            ConfigField::GroqApiUrl => &self.config.groq.api_url,
            ConfigField::GroqApiKey => &self.config.groq.api_key,
            ConfigField::GroqModel => &self.config.groq.model,
            ConfigField::GameTimeout => &self.config.game.timeout,
            ConfigField::GameTextScale => &self.config.game.text_scale,
            ConfigField::GameRhythmSpeed => &self.config.game.rhythm_speed,
            ConfigField::GameFreq => &self.config.game.freq,
            ConfigField::GameSoundEnabled => &self.config.game.sound_enabled,
        }
    }

    fn selected_config_field_len(&self) -> usize {
        if self.config_field.accepts_text() {
            self.selected_config_field().chars().count()
        } else {
            0
        }
    }

    fn clamped_config_cursor_index(&self) -> usize {
        self.config_cursor_index
            .min(self.selected_config_field_len())
    }

    pub(super) fn move_config_cursor_to_end(&mut self) {
        self.config_cursor_index = self.selected_config_field_len();
    }

    fn config_field_index(&self) -> usize {
        ConfigField::ALL
            .iter()
            .position(|field| *field == self.config_field)
            .unwrap_or(0)
    }
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .map(|(byte_index, _)| byte_index)
        .nth(char_index)
        .unwrap_or(value.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{AppConfig, GameSettings, ProviderConfig};

    fn app_config() -> AppConfig {
        AppConfig {
            google: ProviderConfig {
                api_url: "abc".into(),
                api_key: "secret".into(),
                model: "gemini".into(),
            },
            groq: ProviderConfig {
                api_url: String::new(),
                api_key: String::new(),
                model: String::new(),
            },
            game: GameSettings::default(),
        }
    }

    #[test]
    fn config_input_inserts_at_cursor_position() {
        let mut app = App::new(app_config());
        app.open_config();
        app.move_config_cursor_left();
        app.edit_config_char('X');

        assert_eq!(app.config().google.api_url, "abXc");
        assert_eq!(app.config_cursor_index(), 3);
    }

    #[test]
    fn config_backspace_deletes_before_cursor_position() {
        let mut app = App::new(app_config());
        app.open_config();
        app.move_config_cursor_left();
        app.pop_config_char();

        assert_eq!(app.config().google.api_url, "ac");
        assert_eq!(app.config_cursor_index(), 1);
    }

    #[test]
    fn config_cursor_stops_at_field_bounds() {
        let mut app = App::new(app_config());
        app.open_config();

        app.move_config_cursor_right();
        assert_eq!(app.config_cursor_index(), 3);

        app.move_config_cursor_left();
        app.move_config_cursor_left();
        app.move_config_cursor_left();
        app.move_config_cursor_left();
        assert_eq!(app.config_cursor_index(), 0);
    }

    #[test]
    fn moving_config_field_places_cursor_at_target_field_end() {
        let mut app = App::new(app_config());
        app.open_config();

        app.move_config_down();

        assert_eq!(app.config_field(), ConfigField::GoogleApiKey);
        assert_eq!(app.config_cursor_index(), 6);
    }

    #[test]
    fn sound_enabled_ignores_text_cursor_editing() {
        let mut app = App::new(app_config());
        app.open_config();
        for _ in 0..10 {
            app.move_config_down();
        }

        app.move_config_cursor_left();
        app.edit_config_char('x');
        app.pop_config_char();

        assert_eq!(app.config_field(), ConfigField::GameSoundEnabled);
        assert_eq!(app.config().game.sound_enabled, "false");
        assert_eq!(app.config_cursor_index(), 0);
    }
}
