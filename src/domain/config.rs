#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

impl ProviderConfig {
    pub fn is_ready(&self) -> bool {
        !self.api_url.trim().is_empty()
            && !self.api_key.trim().is_empty()
            && !self.model.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettings {
    pub timeout: String,
    pub text_scale: String,
    pub freq: String,
    pub sound_enabled: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            timeout: "60".to_string(),
            text_scale: "60".to_string(),
            freq: "80.0".to_string(),
            sound_enabled: "false".to_string(),
        }
    }
}

impl GameSettings {
    pub fn timeout_value(&self) -> i32 {
        self.timeout.trim().parse().unwrap_or(60)
    }

    pub fn text_scale_value(&self) -> usize {
        self.text_scale.trim().parse().unwrap_or(60)
    }

    pub fn freq_value(&self) -> f32 {
        self.freq.trim().parse().unwrap_or(80.0)
    }

    pub fn sound_enabled_value(&self) -> bool {
        self.sound_enabled.trim().eq_ignore_ascii_case("true")
    }

    pub fn toggle_sound(&mut self) {
        if self.sound_enabled_value() {
            self.sound_enabled = "false".to_string();
        } else {
            self.sound_enabled = "true".to_string();
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppConfig {
    pub google: ProviderConfig,
    pub groq: ProviderConfig,
    pub game: GameSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLoadReport {
    pub config: AppConfig,
    pub warnings: Vec<String>,
}
