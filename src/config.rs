use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::Rng;
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
    let home = env::var("HOME").map_err(|_| io::Error::other("HOME is not set"))?;
    let base_dir = Path::new(&home).join(".config").join("rtyping");
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
    let key = if key_path.exists() {
        Some(read_key(key_path)?)
    } else {
        None
    };

    Ok(AppConfig {
        google: parse_provider_section(&contents, "google", key.as_deref())?,
        groq: parse_provider_section(&contents, "groq", key.as_deref())?,
    })
}

fn save_config_to_paths(config: &AppConfig, config_path: &Path, key_path: &Path) -> io::Result<()> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let key = ensure_key(key_path)?;
    let google = serialize_provider(&config.google, &key)?;
    let groq = serialize_provider(&config.groq, &key)?;

    let body = format!(
        concat!("{{\n", "  \"google\": {},\n", "  \"groq\": {}\n", "}}\n"),
        google, groq
    );

    fs::write(config_path, body)?;
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
    STANDARD
        .decode(encoded.trim())
        .map_err(|err| io::Error::other(format!("failed to decode key: {err}")))
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

fn serialize_provider(provider: &ProviderConfig, key: &[u8]) -> io::Result<String> {
    let (ciphertext, nonce) = encrypt_value(&provider.api_key, key)?;
    Ok(format!(
        concat!(
            "{{",
            "\"api_url\":\"{}\",",
            "\"api_key_ciphertext\":\"{}\",",
            "\"api_key_nonce\":\"{}\",",
            "\"model\":\"{}\"",
            "}}"
        ),
        escape_json(&provider.api_url),
        escape_json(&ciphertext),
        escape_json(&nonce),
        escape_json(&provider.model),
    ))
}

fn parse_provider_section(
    contents: &str,
    section_name: &str,
    key: Option<&[u8]>,
) -> io::Result<ProviderConfig> {
    let Some(section) = extract_object(contents, section_name) else {
        return Ok(ProviderConfig::default());
    };

    let api_url = extract_string_field(section, "api_url").unwrap_or_default();
    let ciphertext = extract_string_field(section, "api_key_ciphertext").unwrap_or_default();
    let nonce = extract_string_field(section, "api_key_nonce").unwrap_or_default();
    let model = extract_string_field(section, "model").unwrap_or_default();

    let api_key = if ciphertext.is_empty() {
        String::new()
    } else if let Some(key) = key {
        decrypt_value(&ciphertext, &nonce, key)?
    } else {
        String::new()
    };

    Ok(ProviderConfig {
        api_url,
        api_key,
        model,
    })
}

fn encrypt_value(value: &str, key: &[u8]) -> io::Result<(String, String)> {
    let mut nonce = [0u8; 16];
    rand::rng().fill_bytes(&mut nonce);
    let ciphertext = xor_with_keystream(value.as_bytes(), key, &nonce);
    Ok((STANDARD.encode(ciphertext), STANDARD.encode(nonce)))
}

fn decrypt_value(ciphertext_b64: &str, nonce_b64: &str, key: &[u8]) -> io::Result<String> {
    let ciphertext = STANDARD
        .decode(ciphertext_b64)
        .map_err(|err| io::Error::other(format!("failed to decode ciphertext: {err}")))?;
    let nonce = STANDARD
        .decode(nonce_b64)
        .map_err(|err| io::Error::other(format!("failed to decode nonce: {err}")))?;
    let plaintext = xor_with_keystream(&ciphertext, key, &nonce);
    String::from_utf8(plaintext)
        .map_err(|err| io::Error::other(format!("failed to decode plaintext: {err}")))
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

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn extract_object<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let key_pos = contents.find(&needle)?;
    let after_key = &contents[key_pos + needle.len()..];
    let brace_offset = after_key.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let object = &after_key[brace_offset..];

    for (idx, ch) in object.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&object[..=idx]);
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_string_field(contents: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let key_pos = contents.find(&needle)?;
    let after_key = &contents[key_pos + needle.len()..];
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        env::temp_dir().join(format!("rtyping-config-test-{nanos}"))
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
