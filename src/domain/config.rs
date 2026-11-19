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
    pub rhythm_speed: String,
    pub freq: String,
    pub sound_enabled: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            timeout: "60".to_string(),
            text_scale: "60".to_string(),
            rhythm_speed: "2".to_string(),
            freq: "80.0".to_string(),
            sound_enabled: "false".to_string(),
        }
    }
}

impl GameSettings {
    pub const MAX_TEXT_SCALE: usize = 1000;

    pub fn timeout_value(&self) -> i32 {
        self.checked_timeout().unwrap_or(60)
    }

    pub fn text_scale_value(&self) -> usize {
        self.checked_text_scale().unwrap_or(60)
    }

    pub fn rhythm_speed_value(&self) -> u8 {
        self.rhythm_speed.trim().parse().unwrap_or(2).clamp(1, 5)
    }

    pub fn freq_value(&self) -> f32 {
        self.checked_frequency().unwrap_or(80.0)
    }

    pub fn sound_enabled_value(&self) -> bool {
        self.sound_enabled.trim().eq_ignore_ascii_case("true")
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut draft = self.clone();
        let warnings = draft.repair();
        if warnings.is_empty() {
            Ok(())
        } else {
            Err(warnings.join(" / "))
        }
    }

    pub fn repair(&mut self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.checked_timeout().is_none() {
            warnings.push("Timeout must be an integer from 0 to 2147483647".into());
            self.timeout = "60".into();
        }
        if self.checked_text_scale().is_none() {
            warnings.push(format!(
                "TextScale must be an integer from 0 to {}",
                Self::MAX_TEXT_SCALE
            ));
            self.text_scale = "60".into();
        }
        if !matches!(self.rhythm_speed.trim().parse::<u8>(), Ok(1..=5)) {
            warnings.push("RhythmSpeed must be an integer from 1 to 5".into());
            self.rhythm_speed = self.rhythm_speed_value().to_string();
        }
        if self.checked_frequency().is_none() {
            warnings.push("Freq must be a positive finite number".into());
            self.freq = "80.0".into();
        }
        if !self.sound_enabled_value() && !self.sound_enabled.trim().eq_ignore_ascii_case("false") {
            warnings.push("SoundEnabled must be true or false".into());
            self.sound_enabled = "false".into();
        }
        warnings
    }

    fn checked_timeout(&self) -> Option<i32> {
        self.timeout.trim().parse().ok().filter(|value| *value >= 0)
    }

    fn checked_text_scale(&self) -> Option<usize> {
        self.text_scale
            .trim()
            .parse()
            .ok()
            .filter(|value| *value <= Self::MAX_TEXT_SCALE)
    }

    fn checked_frequency(&self) -> Option<f32> {
        self.freq
            .trim()
            .parse()
            .ok()
            .filter(|value: &f32| value.is_finite() && *value > 0.0)
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

#[cfg(test)]
mod tests {
    use super::GameSettings;

    #[test]
    fn rhythm_speed_defaults_to_two_for_invalid_values() {
        let settings = GameSettings {
            rhythm_speed: "invalid".to_string(),
            ..GameSettings::default()
        };

        assert_eq!(settings.rhythm_speed_value(), 2);
    }

    #[test]
    fn rhythm_speed_is_clamped_to_supported_range() {
        let low = GameSettings {
            rhythm_speed: "0".to_string(),
            ..GameSettings::default()
        };
        let high = GameSettings {
            rhythm_speed: "9".to_string(),
            ..GameSettings::default()
        };

        assert_eq!(low.rhythm_speed_value(), 1);
        assert_eq!(high.rhythm_speed_value(), 5);
    }
}
