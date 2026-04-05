use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::Rng;
use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
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

pub struct ConfigLoadReport {
    pub config: AppConfig,
    pub warnings: Vec<String>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct StoredAppConfig {
    google: StoredProviderConfig,
    groq: StoredProviderConfig,
}

pub fn load_config() -> io::Result<ConfigLoadReport> {
    let paths = config_paths()?;
    if paths.config_path.exists() {
        return load_config_from_paths(&paths.config_path, &paths.key_path);
    }

    if let Some(alternate_paths) = alternate_config_paths()? {
        if alternate_paths.config_path.exists() {
            return load_config_from_paths(&alternate_paths.config_path, &alternate_paths.key_path);
        }
    }

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
    let base_dir = preferred_config_base_dir()?;
    Ok(ConfigPaths {
        config_path: base_dir.join("config.json"),
        key_path: base_dir.join("config.key"),
    })
}

fn alternate_config_paths() -> io::Result<Option<ConfigPaths>> {
    let preferred_base_dir = preferred_config_base_dir()?;
    let system_base_dir = match dirs::config_dir() {
        Some(path) => path.join("rtyping"),
        None => return Ok(None),
    };

    if system_base_dir == preferred_base_dir {
        return Ok(None);
    }

    Ok(Some(ConfigPaths {
        config_path: system_base_dir.join("config.json"),
        key_path: system_base_dir.join("config.key"),
    }))
}

fn preferred_config_base_dir() -> io::Result<PathBuf> {
    if let Some(dir) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        let path = PathBuf::from(dir);
        if path.is_absolute() {
            return Ok(path.join("rtyping"));
        }
    }

    if let Some(home) = dirs::home_dir() {
        return Ok(home.join(".config").join("rtyping"));
    }

    dirs::config_dir()
        .map(|path| path.join("rtyping"))
        .ok_or_else(|| io::Error::other("failed to resolve config directory"))
}

fn load_config_from_paths(config_path: &Path, key_path: &Path) -> io::Result<ConfigLoadReport> {
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
        config: AppConfig { google, groq },
        warnings,
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

    if let Some(alternate) = alternate_config_paths()? {
        if alternate.key_path != primary_key_path {
            paths.push(alternate.key_path);
        }
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

fn decrypt_with_candidates(
    ciphertext_b64: &str,
    nonce_b64: &str,
    keys: &[Vec<u8>],
    aad_labels: &[&str],
) -> io::Result<String> {
    let mut last_error = io::Error::other("no usable config key was found");

    for key in keys {
        if let Ok(value) = try_decrypt_with_aead_labels(ciphertext_b64, nonce_b64, key, aad_labels)
        {
            return Ok(value);
        }

        match decrypt_legacy_value(ciphertext_b64, nonce_b64, key) {
            Ok(value) => return Ok(value),
            Err(err) => last_error = err,
        }
    }

    Err(last_error)
}

fn try_decrypt_with_aead_labels(
    ciphertext_b64: &str,
    nonce_b64: &str,
    key: &[u8],
    aad_labels: &[&str],
) -> io::Result<String> {
    let mut last_error = io::Error::other("no AEAD labels were provided");

    for aad_label in aad_labels {
        match decrypt_value(ciphertext_b64, nonce_b64, key, aad_label) {
            Ok(value) => return Ok(value),
            Err(err) => last_error = err,
        }
    }

    Err(last_error)
}

fn decrypt_legacy_value(ciphertext_b64: &str, nonce_b64: &str, key: &[u8]) -> io::Result<String> {
    let ciphertext = STANDARD
        .decode(ciphertext_b64)
        .map_err(|err| io::Error::other(format!("failed to decode ciphertext: {err}")))?;
    let nonce = decode_array::<16>(nonce_b64, "legacy nonce")?;
    let plaintext = xor_with_keystream(&ciphertext, key, &nonce);
    String::from_utf8(plaintext)
        .map_err(|err| io::Error::other(format!("failed to decode legacy plaintext: {err}")))
}

fn xor_with_keystream(input: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut counter = 0u64;

    while output.len() < input.len() {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(counter.to_be_bytes());
        let block = hasher.finalize();
        let remaining = input.len() - output.len();
        let block_len = remaining.min(block.len());
        let start = output.len();

        for idx in 0..block_len {
            output.push(input[start + idx] ^ block[idx]);
        }

        counter += 1;
    }

    output
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

        let report = load_config_from_paths(&config_path, &key_path).expect("load should succeed");
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(report.config, config);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_keeps_non_secret_fields_when_key_is_missing() {
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
        fs::remove_file(&key_path).expect("key should be removable");

        let report = load_config_from_paths(&config_path, &key_path).expect("load should succeed");

        assert_eq!(report.config.google.api_url, config.google.api_url);
        assert_eq!(report.config.google.model, config.google.model);
        assert!(report.config.google.api_key.is_empty());
        assert_eq!(report.config.groq.api_url, config.groq.api_url);
        assert_eq!(report.config.groq.model, config.groq.model);
        assert!(report.config.groq.api_key.is_empty());
        assert!(!report.warnings.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_keeps_non_secret_fields_when_key_is_invalid() {
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
        fs::write(&key_path, "broken-key").expect("broken key should be written");

        let report = load_config_from_paths(&config_path, &key_path).expect("load should succeed");

        assert_eq!(report.config.google.api_url, config.google.api_url);
        assert_eq!(report.config.google.model, config.google.model);
        assert!(report.config.google.api_key.is_empty());
        assert_eq!(report.config.groq.api_url, config.groq.api_url);
        assert_eq!(report.config.groq.model, config.groq.model);
        assert!(report.config.groq.api_key.is_empty());
        assert!(!report.warnings.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_restores_api_key_from_legacy_aad_label() {
        let dir = unique_test_dir();
        let config_path = dir.join("config.json");
        let key_path = dir.join("config.key");
        let key = ensure_key(&key_path).expect("key should be created");

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
        };
        let body = serde_json::to_string_pretty(&stored).expect("json should serialize");
        fs::write(&config_path, body).expect("config should be written");

        let report = load_config_from_paths(&config_path, &key_path).expect("load should succeed");

        assert!(report.warnings.is_empty());
        assert_eq!(report.config.google.api_key, "google-secret");
        assert_eq!(report.config.groq.api_key, "groq-secret");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_restores_api_key_from_legacy_xor_format() {
        let dir = unique_test_dir();
        let config_path = dir.join("config.json");
        let key_path = dir.join("config.key");
        let key = ensure_key(&key_path).expect("key should be created");
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
        };
        let body = serde_json::to_string_pretty(&stored).expect("json should serialize");
        fs::write(&config_path, body).expect("config should be written");

        let report = load_config_from_paths(&config_path, &key_path).expect("load should succeed");

        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(report.config.google.api_key, "google-secret");
        assert_eq!(report.config.groq.api_key, "groq-secret");

        let _ = fs::remove_dir_all(dir);
    }

    fn legacy_encrypt_value(value: &str, key: &[u8]) -> (String, String) {
        let mut nonce = [0u8; 16];
        rand::rng().fill_bytes(&mut nonce);
        let ciphertext = xor_with_keystream(value.as_bytes(), key, &nonce);
        (STANDARD.encode(ciphertext), STANDARD.encode(nonce))
    }
}
