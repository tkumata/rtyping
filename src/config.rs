use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::Rng;
use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoredProviderConfig {
    api_url: String,
    api_key_ciphertext: String,
    api_key_nonce: String,
    model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoredAppConfig {
    google: StoredProviderConfig,
    groq: StoredProviderConfig,
}

pub fn load_config() -> io::Result<AppConfig> {
    let paths = config_paths()?;
    load_config_from_paths(&paths.config_path, &paths.key_path)
}

pub fn save_config(config: &AppConfig) -> io::Result<()> {
    let paths = config_paths()?;
    save_config_to_paths(config, &paths.config_path, &paths.key_path)
}

struct ConfigPaths {
    config_path: PathBuf,
    key_path: PathBuf,
}

fn config_paths() -> io::Result<ConfigPaths> {
    let base_dir = dirs::config_dir()
        .ok_or_else(|| io::Error::other("failed to resolve OS config directory"))?
        .join("rtyping");
    Ok(ConfigPaths {
        config_path: base_dir.join("config.json"),
        key_path: base_dir.join("config.key"),
    })
}

fn load_config_from_paths(config_path: &Path, key_path: &Path) -> io::Result<AppConfig> {
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    let contents = fs::read_to_string(config_path)?;
    let stored: StoredAppConfig = serde_json::from_str(&contents)
        .map_err(|err| io::Error::other(format!("failed to parse config.json: {err}")))?;
    let key = if key_path.exists() {
        read_key(key_path)?
    } else if has_encrypted_values(&stored) {
        return Err(io::Error::other("config.key is missing"));
    } else {
        Vec::new()
    };

    Ok(AppConfig {
        google: restore_provider_config(&stored.google, &key, "google")?,
        groq: restore_provider_config(&stored.groq, &key, "groq")?,
    })
}

fn save_config_to_paths(config: &AppConfig, config_path: &Path, key_path: &Path) -> io::Result<()> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let key = ensure_key(key_path)?;
    let stored = StoredAppConfig {
        google: store_provider_config(&config.google, &key, "google")?,
        groq: store_provider_config(&config.groq, &key, "groq")?,
    };
    let body = serde_json::to_string_pretty(&stored)
        .map_err(|err| io::Error::other(format!("failed to serialize config: {err}")))?;

    fs::write(config_path, body)?;
    set_private_permissions(config_path)?;
    Ok(())
}

fn ensure_key(key_path: &Path) -> io::Result<Vec<u8>> {
    if key_path.exists() {
        return read_key(key_path);
    }

    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);
    let encoded = STANDARD.encode(key);
    fs::write(key_path, encoded)?;
    set_private_permissions(key_path)?;
    Ok(key.to_vec())
}

fn read_key(key_path: &Path) -> io::Result<Vec<u8>> {
    let encoded = fs::read_to_string(key_path)?;
    let key = STANDARD
        .decode(encoded.trim())
        .map_err(|err| io::Error::other(format!("failed to decode key: {err}")))?;

    if key.len() != 32 {
        return Err(io::Error::other("config.key has invalid length"));
    }

    Ok(key)
}

fn set_private_permissions(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions)?;
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    Ok(())
}

fn has_encrypted_values(config: &StoredAppConfig) -> bool {
    !config.google.api_key_ciphertext.is_empty() || !config.groq.api_key_ciphertext.is_empty()
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
    key: &[u8],
    aad_label: &str,
) -> io::Result<ProviderConfig> {
    let api_key = if stored.api_key_ciphertext.is_empty() {
        String::new()
    } else {
        decrypt_value(
            &stored.api_key_ciphertext,
            &stored.api_key_nonce,
            key,
            aad_label,
        )?
    };

    Ok(ProviderConfig {
        api_url: stored.api_url.clone(),
        api_key,
        model: stored.model.clone(),
    })
}

fn encrypt_value(value: &str, key: &[u8], aad_label: &str) -> io::Result<(String, String)> {
    let cipher = build_cipher(key)?;
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce_text = STANDARD.encode(nonce_bytes);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let aad = Aad::from(aad_label.as_bytes());
    let mut buffer = value.as_bytes().to_vec();

    cipher
        .seal_in_place_append_tag(nonce, aad, &mut buffer)
        .map_err(|_| io::Error::other("failed to encrypt API key"))?;

    Ok((STANDARD.encode(buffer), nonce_text))
}

fn decrypt_value(
    ciphertext_b64: &str,
    nonce_b64: &str,
    key: &[u8],
    aad_label: &str,
) -> io::Result<String> {
    let cipher = build_cipher(key)?;
    let nonce_bytes = decode_array::<12>(nonce_b64, "nonce")?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let aad = Aad::from(aad_label.as_bytes());
    let mut buffer = STANDARD
        .decode(ciphertext_b64)
        .map_err(|err| io::Error::other(format!("failed to decode ciphertext: {err}")))?;

    let plaintext = cipher
        .open_in_place(nonce, aad, &mut buffer)
        .map_err(|_| io::Error::other("failed to decrypt API key"))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|err| io::Error::other(format!("failed to decode plaintext: {err}")))
}

fn build_cipher(key: &[u8]) -> io::Result<LessSafeKey> {
    let unbound = UnboundKey::new(&CHACHA20_POLY1305, key)
        .map_err(|_| io::Error::other("failed to initialize cipher"))?;
    Ok(LessSafeKey::new(unbound))
}

fn decode_array<const N: usize>(input: &str, label: &str) -> io::Result<[u8; N]> {
    let decoded = STANDARD
        .decode(input)
        .map_err(|err| io::Error::other(format!("failed to decode {label}: {err}")))?;
    let len = decoded.len();
    decoded.try_into().map_err(|_| {
        io::Error::other(format!(
            "{label} has invalid length: expected {N}, got {}",
            len
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("rtyping-config-test-{nanos}"))
    }

    #[test]
    fn save_and_load_config_round_trip() {
        let dir = unique_test_dir();
        let config_path = dir.join("config.json");
        let key_path = dir.join("config.key");

        let config = AppConfig {
            google: ProviderConfig {
                api_url: "https://example.com/google".into(),
                api_key: "google-secret".into(),
                model: "gemini".into(),
            },
            groq: ProviderConfig {
                api_url: "https://example.com/groq".into(),
                api_key: "groq-secret".into(),
                model: "llama".into(),
            },
        };

        save_config_to_paths(&config, &config_path, &key_path).expect("save should succeed");
        let raw = fs::read_to_string(&config_path).expect("config should exist");

        assert!(!raw.contains("google-secret"));
        assert!(!raw.contains("groq-secret"));

        let loaded = load_config_from_paths(&config_path, &key_path).expect("load should succeed");
        assert_eq!(loaded, config);

        let _ = fs::remove_dir_all(dir);
    }
}
