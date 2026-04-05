use rand::RngExt;
use rand::prelude::IndexedRandom;
use rand::rng;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io;
use std::time::Duration;

use crate::config::ProviderConfig;
use crate::domain::entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationSource {
    Local,
    Google,
    Groq,
}

pub fn generate_sentence(
    text_scale: usize,
    source: GenerationSource,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let target_chars = target_character_count(text_scale);
    let sentence = match source {
        GenerationSource::Local => generate_local_sentence(target_chars)?,
        GenerationSource::Google => generate_google_sentence(target_chars, provider_config)?,
        GenerationSource::Groq => generate_groq_sentence(target_chars, provider_config)?,
    };

    Ok(normalize_sentence(&sentence, target_chars))
}

pub fn target_character_count(text_scale: usize) -> usize {
    (text_scale.max(4)) * 5
}

fn generate_local_sentence(target_chars: usize) -> Result<String, io::Error> {
    match entity::get_sample() {
        Ok(sampling_contents) => Ok(generate_markov_chain(&sampling_contents, 4, target_chars)),
        Err(err) => {
            eprintln!("Failed to read file: {}", err);
            Err(err)
        }
    }
}

fn generate_google_sentence(
    target_chars: usize,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let config = provider_config.ok_or_else(|| io::Error::other("Google config is missing"))?;
    validate_provider_config("Google AI Studio", &config)?;

    let prompt = build_prompt(target_chars);
    let url = build_google_url(&config);
    let body = json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": prompt
                    }
                ]
            }
        ]
    });

    let response = build_http_client()?
        .post(format!("{url}?key={}", config.api_key.trim()))
        .json(&body)
        .send()
        .map_err(|err| io::Error::other(format!("Google request failed: {err}")))?;

    let status = response.status();
    let response_text = response
        .text()
        .map_err(|err| io::Error::other(format!("Failed to read Google response body: {err}")))?;
    let payload = parse_json_response("Google AI Studio", status, response_text)?;
    payload["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| io::Error::other("Failed to parse Google AI Studio response"))
}

fn generate_groq_sentence(
    target_chars: usize,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let config = provider_config.ok_or_else(|| io::Error::other("Groq config is missing"))?;
    validate_provider_config("Groq", &config)?;

    let prompt = build_prompt(target_chars);
    let body = json!({
        "model": config.model.trim(),
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ]
    });

    let response = build_http_client()?
        .post(config.api_url.trim())
        .bearer_auth(config.api_key.trim())
        .json(&body)
        .send()
        .map_err(|err| io::Error::other(format!("Groq request failed: {err}")))?;

    let status = response.status();
    let response_text = response
        .text()
        .map_err(|err| io::Error::other(format!("Failed to read Groq response body: {err}")))?;
    let payload = parse_json_response("Groq", status, response_text)?;
    payload["choices"][0]["message"]["content"]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| io::Error::other("Failed to parse Groq response"))
}

fn build_google_url(config: &ProviderConfig) -> String {
    let base = config.api_url.trim().trim_end_matches('/');
    let model = config.model.trim().trim_matches('/');
    format!("{base}/{model}:generateContent")
}

fn validate_provider_config(provider_name: &str, config: &ProviderConfig) -> Result<(), io::Error> {
    if config.is_ready() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{provider_name} config is incomplete"
        )))
    }
}

fn build_http_client() -> Result<Client, io::Error> {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| io::Error::other(format!("Failed to build HTTP client: {err}")))
}

fn parse_json_response(
    provider_name: &str,
    status: StatusCode,
    response_text: String,
) -> Result<Value, io::Error> {
    if !status.is_success() {
        let summary = response_text.replace('\n', " ");
        return Err(io::Error::other(format!(
            "{provider_name} returned HTTP {}: {}",
            status.as_u16(),
            summary
        )));
    }

    serde_json::from_str(&response_text).map_err(|err| {
        io::Error::other(format!(
            "{provider_name} returned invalid JSON: {err}; body={response_text}"
        ))
    })
}

fn build_prompt(target_chars: usize) -> String {
    format!(
        "Generate plain English typing text with about {target_chars} characters. Use ASCII letters, spaces, and simple punctuation only. Do not add markdown, numbering, labels, or quotes."
    )
}

fn normalize_sentence(sentence: &str, target_chars: usize) -> String {
    let cleaned = sentence.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut normalized = String::with_capacity(cleaned.len());

    for ch in cleaned.chars() {
        if ch.is_ascii() && !ch.is_control() {
            normalized.push(ch);
        } else if ch.is_whitespace() {
            normalized.push(' ');
        }
    }

    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return "typing practice fallback text"
            .chars()
            .take(target_chars)
            .collect();
    }

    trimmed.chars().take(target_chars).collect()
}

fn generate_markov_chain(text: &str, n: usize, target_chars: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut markov_chain: HashMap<Vec<&str>, Vec<&str>> = HashMap::new();

    for i in 0..(words.len() - n) {
        let key = words[i..i + n].to_vec();
        let value = words[i + n];
        markov_chain.entry(key).or_default().push(value);
    }

    let mut rng = rng();
    let start_index = rng.random_range(0..words.len() - n);
    let mut current_state = words[start_index..start_index + n].to_vec();
    let mut result = current_state.clone();
    let mut current_len = result
        .iter()
        .map(|word| word.chars().count())
        .sum::<usize>()
        + result.len().saturating_sub(1);

    while current_len < target_chars {
        if let Some(next_words) = markov_chain.get(&current_state) {
            let next_word = next_words.choose(&mut rng).expect("next word should exist");
            result.push(*next_word);
            current_len += next_word.chars().count() + 1;
            current_state.push(*next_word);
            current_state.remove(0);
        } else {
            break;
        }
    }

    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sentence_success() {
        let sentence = generate_sentence(10, GenerationSource::Local, None)
            .expect("local generation should succeed");
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
            build_google_url(&config),
            "https://example.com/v1beta/models/gemini-2.0-flash:generateContent"
        );
    }
}
