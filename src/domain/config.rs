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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppConfig {
    pub google: ProviderConfig,
    pub groq: ProviderConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLoadReport {
    pub config: AppConfig,
    pub warnings: Vec<String>,
}
