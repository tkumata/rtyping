use super::{App, ConfigField};

impl App {
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
        if self.config_field != ConfigField::GameSoundEnabled {
            self.selected_config_field_mut().push(ch);
        }
    }

    pub fn pop_config_char(&mut self) {
        if self.config_field != ConfigField::GameSoundEnabled {
            self.selected_config_field_mut().pop();
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
            ConfigField::GameFreq => &mut self.config.game.freq,
            ConfigField::GameSoundEnabled => &mut self.config.game.sound_enabled,
        }
    }

    fn config_field_index(&self) -> usize {
        ConfigField::ALL
            .iter()
            .position(|field| *field == self.config_field)
            .unwrap_or(0)
    }
}
