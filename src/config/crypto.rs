use base64::{Engine as _, engine::general_purpose::STANDARD};
use rand::Rng;
use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, Nonce, UnboundKey};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(super) fn ensure_key(key_path: &Path) -> io::Result<Vec<u8>> {
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

pub(super) fn read_key(key_path: &Path) -> io::Result<Vec<u8>> {
    let encoded = fs::read_to_string(key_path)?;
    let key = STANDARD
        .decode(encoded.trim())
        .map_err(|err| io::Error::other(format!("failed to decode key: {err}")))?;

    if key.len() != 32 {
        return Err(io::Error::other("config.key has invalid length"));
    }

    Ok(key)
}

pub(super) fn set_private_permissions(path: &Path) -> io::Result<()> {
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

pub(super) fn decrypt_with_candidates(
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

pub(super) fn encrypt_value(value: &str, key: &[u8], aad_label: &str) -> io::Result<(String, String)> {
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

pub(super) fn xor_with_keystream(input: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
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
