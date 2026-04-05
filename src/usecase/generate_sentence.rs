use rand::RngExt;
use rand::prelude::IndexedRandom;
use rand::rng;
use std::collections::HashMap;
use std::io;
use std::process::Command;

use crate::config::ProviderConfig;
use crate::domain::entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationSource {
    Local,
    Google,
    Groq,
}

pub fn generate_sentence(
    level: usize,
    source: GenerationSource,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let target_chars = target_character_count(level);
    let sentence = match source {
        GenerationSource::Local => generate_local_sentence(target_chars)?,
        GenerationSource::Google => generate_google_sentence(target_chars, provider_config)?,
        GenerationSource::Groq => generate_groq_sentence(target_chars, provider_config)?,
    };

    Ok(normalize_sentence(&sentence, target_chars))
}

pub fn target_character_count(level: usize) -> usize {
    (level.max(4)) * 5
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
    let body = format!(
        "{{\"contents\":[{{\"parts\":[{{\"text\":\"{}\"}}]}}]}}",
        escape_json(&prompt)
    );
    let output = run_curl(&[
        "-sS",
        "-X",
        "POST",
        url.as_str(),
        "-H",
        "Content-Type: application/json",
        "-d",
        body.as_str(),
    ])?;

    extract_string_after_marker(&output, "\"candidates\"", "\"text\"")
        .ok_or_else(|| io::Error::other("Failed to parse Google AI Studio response"))
}

fn generate_groq_sentence(
    target_chars: usize,
    provider_config: Option<ProviderConfig>,
) -> Result<String, io::Error> {
    let config = provider_config.ok_or_else(|| io::Error::other("Groq config is missing"))?;
    validate_provider_config("Groq", &config)?;

    let prompt = build_prompt(target_chars);
    let auth_header = format!("Authorization: Bearer {}", config.api_key.trim());
    let body = format!(
        "{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
        escape_json(config.model.trim()),
        escape_json(&prompt)
    );
    let output = run_curl(&[
        "-sS",
        "-X",
        "POST",
        config.api_url.trim(),
        "-H",
        "Content-Type: application/json",
        "-H",
        auth_header.as_str(),
        "-d",
        body.as_str(),
    ])?;

    extract_string_after_marker(&output, "\"message\"", "\"content\"")
        .ok_or_else(|| io::Error::other("Failed to parse Groq response"))
}

fn build_google_url(config: &ProviderConfig) -> String {
    let base = config.api_url.trim().trim_end_matches('/');
    let model = config.model.trim().trim_matches('/');
    format!(
        "{base}/{model}:generateContent?key={}",
        config.api_key.trim()
    )
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

fn run_curl(args: &[&str]) -> Result<String, io::Error> {
    let output = Command::new("curl").args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!("curl failed: {stderr}")));
    }

    String::from_utf8(output.stdout)
        .map_err(|err| io::Error::other(format!("invalid response encoding: {err}")))
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

    while result.join(" ").chars().count() < target_chars {
        if let Some(next_words) = markov_chain.get(&current_state) {
            let next_word = next_words.choose(&mut rng).expect("next word should exist");
            result.push(*next_word);
            current_state.push(*next_word);
            current_state.remove(0);
        } else {
            break;
        }
    }

    result.join(" ")
}

fn extract_string_after_marker(contents: &str, marker: &str, key: &str) -> Option<String> {
    let section_start = contents.find(marker)?;
    let section = &contents[section_start..];
    extract_string_field(section, key)
}

fn extract_string_field(contents: &str, key: &str) -> Option<String> {
    let key_pos = contents.find(key)?;
    let after_key = &contents[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();
    let quoted = after_colon.strip_prefix('"')?;
    let mut escaped = false;
    let mut value = String::new();

    for ch in quoted.chars() {
        if escaped {
            let actual = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                'u' => return Some(value),
                other => other,
            };
            value.push(actual);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some(value),
            other => value.push(other),
        }
    }

    None
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
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
            "https://example.com/v1beta/models/gemini-2.0-flash:generateContent?key=secret"
        );
    }
}
