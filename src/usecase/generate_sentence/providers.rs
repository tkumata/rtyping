use rand::RngExt;
use rand::prelude::IndexedRandom;
use rand::rng;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::domain::config::ProviderConfig;

static PROMPT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

const PROMPT_CONTEXTS: &[&str] = &[
    "morning routine",
    "small city errand",
    "quiet office task",
    "weekend cooking",
    "train station moment",
    "library visit",
    "neighborhood walk",
    "workshop cleanup",
    "planning a short trip",
    "fixing a simple mistake",
    "organizing a desk",
    "talking with a coworker",
];

pub(super) fn generate_google_sentence(
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
    let payload = parse_json_response("Google AI Studio", status, &response_text)?;
    payload
        .get("candidates")
        .and_then(|candidates| candidates.get(0))
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.get(0))
        .and_then(|part| part.get("text"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| io::Error::other("Failed to parse Google AI Studio response"))
}

pub(super) fn generate_groq_sentence(
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
    let payload = parse_json_response("Groq", status, &response_text)?;
    payload
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| io::Error::other("Failed to parse Groq response"))
}

pub(super) fn build_google_url(config: &ProviderConfig) -> String {
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
    response_text: &str,
) -> Result<Value, io::Error> {
    if !status.is_success() {
        let summary = response_text.replace('\n', " ");
        return Err(io::Error::other(format!(
            "{provider_name} returned HTTP {}: {}",
            status.as_u16(),
            summary
        )));
    }

    serde_json::from_str(response_text)
        .map_err(|err| io::Error::other(format!("{provider_name} returned invalid JSON: {err}")))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PromptVariation {
    seed: String,
    context: &'static str,
}

impl PromptVariation {
    fn random() -> Self {
        let sequence = PROMPT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut rng = rng();
        let random = rng.random::<u64>();
        let context = match PROMPT_CONTEXTS.choose(&mut rng) {
            Some(context) => *context,
            None => "everyday task",
        };

        Self {
            seed: format!("{sequence:x}-{random:x}"),
            context,
        }
    }

    #[cfg(test)]
    pub(super) fn for_test(seed: impl Into<String>, context: &'static str) -> Self {
        Self {
            seed: seed.into(),
            context,
        }
    }
}

pub(super) fn build_prompt(target_chars: usize) -> String {
    build_prompt_with_variation(target_chars, &PromptVariation::random())
}

pub(super) fn build_prompt_with_variation(
    target_chars: usize,
    variation: &PromptVariation,
) -> String {
    format!(
        "Generate one plain English typing text with about {target_chars} characters. Make it long enough that the app can trim it to the target length. Variation seed: {}. Situation focus: {}. Use a fresh topic, opening, wording, and sentence order for this seed. Do not print the seed or the focus label. Use ASCII letters, spaces, and simple punctuation only. Do not add markdown, numbering, labels, or quotes.",
        variation.seed, variation.context
    )
}
