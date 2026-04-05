mod local;
mod normalize;
mod providers;

use std::io;

use crate::domain::config::ProviderConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationSource {
    Local,
    Google,
    Groq,
}

pub fn generate(
    text_scale: usize,
    source: GenerationSource,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let target_chars = target_character_count(text_scale);
    let sentence = match source {
        GenerationSource::Local => local::generate_local_sentence(target_chars)?,
        GenerationSource::Google => providers::generate_google_sentence(target_chars, provider_config)?,
        GenerationSource::Groq => providers::generate_groq_sentence(target_chars, provider_config)?,
    };

    Ok(normalize::normalize_sentence(&sentence, target_chars))
}

pub fn target_character_count(text_scale: usize) -> usize {
    (text_scale.max(4)) * 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sentence_success() {
        let sentence =
            generate(10, GenerationSource::Local, None).expect("local generation should succeed");
        assert!(
            !sentence.is_empty(),
            "Generated sentence should not be empty"
        );
        assert!(sentence.chars().count() <= target_character_count(10));
    }

    #[test]
    fn target_length_scales_with_level() {
        assert!(target_character_count(20) > target_character_count(10));
    }

    #[test]
    fn google_url_is_built_from_base_and_model() {
        let config = ProviderConfig {
            api_url: "https://example.com/v1beta/models".into(),
            api_key: "secret".into(),
            model: "gemini-2.0-flash".into(),
        };

        assert_eq!(
            providers::build_google_url(&config),
            "https://example.com/v1beta/models/gemini-2.0-flash:generateContent"
        );
    }

    #[test]
    fn google_generation_requires_complete_config() {
        let err = generate(10, GenerationSource::Google, None).expect_err("config should be required");
        assert!(err.to_string().contains("Google config is missing"));

        let incomplete = ProviderConfig {
            api_url: "https://example.com".into(),
            api_key: String::new(),
            model: "gemini".into(),
        };
        let err = generate(10, GenerationSource::Google, Some(incomplete))
            .expect_err("incomplete config should fail");
        assert!(err.to_string().contains("Google AI Studio config is incomplete"));
    }

    #[test]
    fn groq_generation_requires_complete_config() {
        let err = generate(10, GenerationSource::Groq, None).expect_err("config should be required");
        assert!(err.to_string().contains("Groq config is missing"));

        let incomplete = ProviderConfig {
            api_url: "https://example.com".into(),
            api_key: "secret".into(),
            model: String::new(),
        };
        let err = generate(10, GenerationSource::Groq, Some(incomplete))
            .expect_err("incomplete config should fail");
        assert!(err.to_string().contains("Groq config is incomplete"));
    }

    #[test]
    fn normalize_sentence_filters_non_ascii_and_truncates() {
        let normalized = normalize::normalize_sentence("Hello\n世界  test\t123!", 12);
        assert!(normalized.is_ascii());
        assert!(normalized.chars().count() <= 12);
        assert!(!normalized.contains('世'));
        assert!(normalized.starts_with("Hello"));
    }
}
