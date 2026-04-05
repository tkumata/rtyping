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
