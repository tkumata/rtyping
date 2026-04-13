use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::domain::config::{AppConfig, ConfigLoadReport, GameSettings, ProviderConfig};

use super::crypto::{
    decrypt_with_candidates, encrypt_value, ensure_key, read_key, set_private_permissions,
};
use super::paths::alternate_config_paths;

#[derive(Copy, Clone)]
struct ProviderRestoreSpec<'a> {
    current_aad_label: &'a str,
    legacy_aad_label: &'a str,
    provider_name: &'a str,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoredProviderConfig {
    api_url: String,
    api_key_ciphertext: String,
    api_key_nonce: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredGameSettings {
    #[serde(default = "default_timeout")]
    timeout: String,
    #[serde(default = "default_text_scale")]
    text_scale: String,
    #[serde(default = "default_freq")]
    freq: String,
    #[serde(default = "default_sound_enabled")]
    sound_enabled: String,
}

fn default_timeout() -> String {
    "60".to_string()
}
fn default_text_scale() -> String {
    "60".to_string()
}
fn default_freq() -> String {
    "80.0".to_string()
}
fn default_sound_enabled() -> String {
    "false".to_string()
}

impl Default for StoredGameSettings {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            text_scale: default_text_scale(),
            freq: default_freq(),
            sound_enabled: default_sound_enabled(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoredAppConfig {
    google: StoredProviderConfig,
    groq: StoredProviderConfig,
    #[serde(default)]
    game: StoredGameSettings,
}

pub(super) fn load_config_from_paths(
    config_path: &Path,
    key_path: &Path,
) -> io::Result<ConfigLoadReport> {
    if !config_path.exists() {
        return Ok(ConfigLoadReport {
            config: AppConfig::default(),
            warnings: Vec::new(),
        });
    }

    let contents = fs::read_to_string(config_path)?;
    let stored: StoredAppConfig = serde_json::from_str(&contents)
        .map_err(|err| io::Error::other(format!("failed to parse config.json: {err}")))?;
    let mut warnings = Vec::new();
    let keys = read_candidate_keys(key_path, &stored, &mut warnings)?;

    let google = restore_provider_config(
        &stored.google,
        &keys,
        ProviderRestoreSpec {
            current_aad_label: "google",
            legacy_aad_label: "Google",
            provider_name: "Google",
        },
        &mut warnings,
    );
    let groq = restore_provider_config(
        &stored.groq,
        &keys,
        ProviderRestoreSpec {
            current_aad_label: "groq",
            legacy_aad_label: "Groq",
            provider_name: "Groq",
        },
        &mut warnings,
    );

    Ok(ConfigLoadReport {
        config: AppConfig {
            google,
            groq,
            game: GameSettings {
                timeout: stored.game.timeout.clone(),
                text_scale: stored.game.text_scale.clone(),
                freq: stored.game.freq.clone(),
                sound_enabled: stored.game.sound_enabled.clone(),
            },
        },
        warnings,
    })
}

pub(super) fn save_config_to_paths(
    config: &AppConfig,
    config_path: &Path,
    key_path: &Path,
) -> io::Result<()> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let key = ensure_key(key_path)?;
    let stored = StoredAppConfig {
        google: store_provider_config(&config.google, &key, "google")?,
        groq: store_provider_config(&config.groq, &key, "groq")?,
        game: StoredGameSettings {
            timeout: config.game.timeout.clone(),
            text_scale: config.game.text_scale.clone(),
            freq: config.game.freq.clone(),
            sound_enabled: config.game.sound_enabled.clone(),
        },
    };
    let body = serde_json::to_string_pretty(&stored)
        .map_err(|err| io::Error::other(format!("failed to serialize config: {err}")))?;

    fs::write(config_path, body)?;
    set_private_permissions(config_path)?;
    Ok(())
}

fn has_encrypted_values(config: &StoredAppConfig) -> bool {
    !config.google.api_key_ciphertext.is_empty() || !config.groq.api_key_ciphertext.is_empty()
}

fn read_candidate_keys(
    key_path: &Path,
    stored: &StoredAppConfig,
    warnings: &mut Vec<String>,
) -> io::Result<Vec<Vec<u8>>> {
    let mut keys = Vec::new();

    for candidate in candidate_key_paths(key_path)? {
        if !candidate.exists() {
            continue;
        }

        match read_key(&candidate) {
            Ok(key) => {
                if !keys.iter().any(|known| known == &key) {
                    keys.push(key);
                }
            }
            Err(err) => {
                warnings.push(format!(
                    "failed to read key file {}: {err}",
                    candidate.display()
                ));
            }
        }
    }

    if keys.is_empty() && has_encrypted_values(stored) {
        warnings.push("no usable config key was found; API key fields were left blank".into());
    }

    Ok(keys)
}

fn candidate_key_paths(primary_key_path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = vec![primary_key_path.to_path_buf()];

    if let Some(alternate) = alternate_config_paths()?
        && alternate.key_path != primary_key_path
    {
        paths.push(alternate.key_path);
    }

    Ok(paths)
}

fn store_provider_config(
    provider: &ProviderConfig,
    key: &[u8],
    aad_label: &str,
) -> io::Result<StoredProviderConfig> {
    let (ciphertext, nonce) = if provider.api_key.is_empty() {
        (String::new(), String::new())
    } else {
        encrypt_value(&provider.api_key, key, aad_label)?
    };

    Ok(StoredProviderConfig {
        api_url: provider.api_url.clone(),
        api_key_ciphertext: ciphertext,
        api_key_nonce: nonce,
        model: provider.model.clone(),
    })
}

fn restore_provider_config(
    stored: &StoredProviderConfig,
    keys: &[Vec<u8>],
    spec: ProviderRestoreSpec<'_>,
    warnings: &mut Vec<String>,
) -> ProviderConfig {
    let api_key = if stored.api_key_ciphertext.is_empty() {
        String::new()
    } else {
        match decrypt_with_candidates(
            &stored.api_key_ciphertext,
            &stored.api_key_nonce,
            keys,
            &[spec.current_aad_label, spec.legacy_aad_label],
        ) {
            Ok(api_key) => api_key,
            Err(last_error) => {
                warnings.push(format!(
                    "{} API key could not be restored: {}",
                    spec.provider_name, last_error
                ));
                String::new()
            }
        }
    };

    ProviderConfig {
        api_url: stored.api_url.clone(),
        api_key,
        model: stored.model.clone(),
    }
}

#[cfg(test)]
pub(super) mod test_support {
    #![expect(clippy::expect_used)]
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use rand::Rng;
    use std::fs;
    use std::path::Path;

    use super::StoredAppConfig;
    use super::StoredProviderConfig;
    use crate::config::crypto::{encrypt_value, ensure_key, xor_with_keystream};

    pub(crate) fn write_legacy_aad_config(config_path: &Path, key_path: &Path) -> (String, String) {
        let key = ensure_key(key_path).expect("key should be created");
        let google_cipher = encrypt_value("google-secret", &key, "Google").expect("encrypt");
        let groq_cipher = encrypt_value("groq-secret", &key, "Groq").expect("encrypt");
        let stored = StoredAppConfig {
            google: StoredProviderConfig {
                api_url: "https://example.com/google".into(),
                api_key_ciphertext: google_cipher.0,
                api_key_nonce: google_cipher.1,
                model: "gemini".into(),
            },
            groq: StoredProviderConfig {
                api_url: "https://example.com/groq".into(),
                api_key_ciphertext: groq_cipher.0,
                api_key_nonce: groq_cipher.1,
                model: "llama".into(),
            },
            ..StoredAppConfig::default()
        };
        let body = serde_json::to_string_pretty(&stored).expect("json should serialize");
        fs::write(config_path, body).expect("config should be written");
        ("google-secret".into(), "groq-secret".into())
    }

    pub(crate) fn write_legacy_xor_config(config_path: &Path, key_path: &Path) -> (String, String) {
        let key = ensure_key(key_path).expect("key should be created");
        let google_cipher = legacy_encrypt_value("google-secret", &key);
        let groq_cipher = legacy_encrypt_value("groq-secret", &key);
        let stored = StoredAppConfig {
            google: StoredProviderConfig {
                api_url: "https://example.com/google".into(),
                api_key_ciphertext: google_cipher.0,
                api_key_nonce: google_cipher.1,
                model: "gemini".into(),
            },
            groq: StoredProviderConfig {
                api_url: "https://example.com/groq".into(),
                api_key_ciphertext: groq_cipher.0,
                api_key_nonce: groq_cipher.1,
                model: "llama".into(),
            },
            ..StoredAppConfig::default()
        };
        let body = serde_json::to_string_pretty(&stored).expect("json should serialize");
        fs::write(config_path, body).expect("config should be written");
        ("google-secret".into(), "groq-secret".into())
    }

    fn legacy_encrypt_value(value: &str, key: &[u8]) -> (String, String) {
        let mut nonce = [0u8; 16];
        rand::rng().fill_bytes(&mut nonce);
        let ciphertext = xor_with_keystream(value.as_bytes(), key, &nonce);
        (STANDARD.encode(ciphertext), STANDARD.encode(nonce))
    }
}
