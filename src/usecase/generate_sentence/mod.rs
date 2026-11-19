mod local;
mod normalize;
mod providers;

use std::io;

use crate::domain::config::{GameSettings, ProviderConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationSource {
    Local,
    Google,
    Groq,
}

impl GenerationSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Google => "Google AI Studio",
            Self::Groq => "Groq",
        }
    }
}

pub fn generate(
    text_scale: usize,
    source: GenerationSource,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let target_chars = target_character_count(text_scale)?;
    let sentence = match source {
        GenerationSource::Local => local::generate_local_sentence(target_chars),
        GenerationSource::Google => {
            providers::generate_google_sentence(target_chars, provider_config)?
        }
        GenerationSource::Groq => providers::generate_groq_sentence(target_chars, provider_config)?,
    };

    Ok(normalize::normalize_sentence(&sentence, target_chars))
}

fn target_character_count(text_scale: usize) -> io::Result<usize> {
    if text_scale > GameSettings::MAX_TEXT_SCALE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "TextScale exceeds the supported range",
        ));
    }
    Ok(text_scale.max(4) * 5)
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used)]
    use super::*;

    #[test]
    fn google_request_error_does_not_display_the_api_key() {
        let config = ProviderConfig {
            api_url: "ftp://127.0.0.1/models".into(),
            api_key: "dummy-private-token".into(),
            model: "test".into(),
        };
        let result = generate(4, GenerationSource::Google, Some(config));
        assert!(
            matches!(result, Err(err) if err.to_string().contains("Google request failed") && !err.to_string().contains("dummy-private-token"))
        );
    }

    #[test]
    fn test_generate_sentence_success() {
        let sentence =
            generate(10, GenerationSource::Local, None).expect("local generation should succeed");
        assert!(
            !sentence.is_empty(),
            "Generated sentence should not be empty"
        );
        assert!(sentence.chars().count() <= target_character_count(10).unwrap_or_default());
    }

    #[test]
    fn target_length_scales_with_level() {
        assert_eq!(target_character_count(20).ok(), Some(100));
        assert_eq!(target_character_count(10).ok(), Some(50));
        assert!(target_character_count(usize::MAX).is_err());
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
    fn external_prompt_includes_variation_without_token_limit_instruction() {
        let variation = providers::PromptVariation::for_test("seed-123", "library visit");
        let prompt = providers::build_prompt_with_variation(120, &variation);

        assert!(prompt.contains("120"));
        assert!(prompt.contains("seed-123"));
        assert!(prompt.contains("library visit"));
        assert!(prompt.contains("fresh topic"));
        assert!(prompt.contains("ASCII"));
        assert!(!prompt.contains("maxOutputTokens"));
        assert!(!prompt.contains("max_completion_tokens"));
    }

    #[test]
    fn external_prompt_changes_between_requests() {
        let first = providers::build_prompt(120);
        let second = providers::build_prompt(120);

        assert_ne!(first, second);
    }

    #[test]
    fn google_generation_requires_complete_config() {
        let err =
            generate(10, GenerationSource::Google, None).expect_err("config should be required");
        assert!(err.to_string().contains("Google config is missing"));

        let incomplete = ProviderConfig {
            api_url: "https://example.com".into(),
            api_key: String::new(),
            model: "gemini".into(),
        };
        let err = generate(10, GenerationSource::Google, Some(incomplete))
            .expect_err("incomplete config should fail");
        assert!(
            err.to_string()
                .contains("Google AI Studio config is incomplete")
        );
    }

    #[test]
    fn groq_generation_requires_complete_config() {
        let err =
            generate(10, GenerationSource::Groq, None).expect_err("config should be required");
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
