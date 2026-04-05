use super::{load_config, save_config};
use super::paths::alternate_config_paths;
use super::storage::{load_config_from_paths, save_config_to_paths, test_support};
use crate::domain::config::{AppConfig, ProviderConfig};
use rand::RngExt;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct TestConfigSandbox {
    dir: PathBuf,
    config_path: PathBuf,
    key_path: PathBuf,
}

impl TestConfigSandbox {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        let random = rand::rng().random::<u64>();
        let dir = std::env::temp_dir().join(format!(
            "rtyping-config-test-{}-{nanos}-{random}",
            process::id()
        ));

        Self {
            config_path: dir.join("config.json"),
            key_path: dir.join("config.key"),
            dir,
        }
    }
}

impl Drop for TestConfigSandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

struct EnvSandbox {
    _lock: MutexGuard<'static, ()>,
    root: PathBuf,
    old_home: Option<std::ffi::OsString>,
    old_xdg_config_home: Option<std::ffi::OsString>,
}

impl EnvSandbox {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().expect("env lock should be available");
        let sandbox = TestConfigSandbox::new();
        fs::create_dir_all(&sandbox.dir).expect("sandbox dir should be created");

        let old_home = env::var_os("HOME");
        let old_xdg_config_home = env::var_os("XDG_CONFIG_HOME");
        let home_dir = sandbox.dir.join("home");
        let xdg_dir = sandbox.dir.join("xdg");
        fs::create_dir_all(&home_dir).expect("home dir should be created");
        fs::create_dir_all(&xdg_dir).expect("xdg dir should be created");

        unsafe {
            env::set_var("HOME", &home_dir);
            env::set_var("XDG_CONFIG_HOME", &xdg_dir);
        }

        Self {
            _lock: lock,
            root: sandbox.dir.clone(),
            old_home,
            old_xdg_config_home,
        }
    }

    fn preferred_dir(&self) -> PathBuf {
        self.root.join("xdg").join("rtyping")
    }

    fn legacy_dir(&self) -> PathBuf {
        dirs::config_dir()
            .expect("config dir should resolve")
            .join("rtyping")
    }
}

impl Drop for EnvSandbox {
    fn drop(&mut self) {
        unsafe {
            match &self.old_home {
                Some(value) => env::set_var("HOME", value),
                None => env::remove_var("HOME"),
            }
            match &self.old_xdg_config_home {
                Some(value) => env::set_var("XDG_CONFIG_HOME", value),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn sample_config() -> AppConfig {
    AppConfig {
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
    }
}

#[test]
fn save_and_load_config_round_trip() {
    let sandbox = TestConfigSandbox::new();
    let config = sample_config();

    save_config_to_paths(&config, &sandbox.config_path, &sandbox.key_path).expect("save should succeed");
    let raw = fs::read_to_string(&sandbox.config_path).expect("config should exist");

    assert!(!raw.contains("google-secret"));
    assert!(!raw.contains("groq-secret"));

    let report = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect("load should succeed");
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(report.config, config);
}

#[test]
fn load_missing_config_returns_default_without_warnings() {
    let sandbox = TestConfigSandbox::new();
    let report = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect("load should succeed");
    assert_eq!(report.config, AppConfig::default());
    assert!(report.warnings.is_empty());
}

#[test]
fn load_invalid_json_returns_parse_error() {
    let sandbox = TestConfigSandbox::new();
    fs::create_dir_all(&sandbox.dir).expect("dir should be created");
    fs::write(&sandbox.config_path, "{not-json").expect("broken config should be written");

    let err = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect_err("load should fail");
    assert!(err.to_string().contains("failed to parse config.json"));
}

#[test]
fn save_config_uses_xdg_config_home() {
    let env_sandbox = EnvSandbox::new();
    let config = sample_config();

    save_config(&config).expect("save should succeed");

    let config_path = env_sandbox.preferred_dir().join("config.json");
    let key_path = env_sandbox.preferred_dir().join("config.key");
    assert!(config_path.exists());
    assert!(key_path.exists());

    let raw = fs::read_to_string(config_path).expect("config should be readable");
    assert!(!raw.contains("google-secret"));
    assert!(!raw.contains("groq-secret"));
}

#[test]
fn load_config_prefers_xdg_path_over_legacy_path() {
    let env_sandbox = EnvSandbox::new();

    let preferred = AppConfig {
        google: ProviderConfig {
            api_url: "https://preferred.example/google".into(),
            api_key: "preferred-secret".into(),
            model: "preferred-model".into(),
        },
        groq: ProviderConfig::default(),
    };
    let legacy = AppConfig {
        google: ProviderConfig {
            api_url: "https://legacy.example/google".into(),
            api_key: "legacy-secret".into(),
            model: "legacy-model".into(),
        },
        groq: ProviderConfig::default(),
    };

    let preferred_dir = env_sandbox.preferred_dir();
    let legacy_dir = env_sandbox.legacy_dir();
    save_config_to_paths(&preferred, &preferred_dir.join("config.json"), &preferred_dir.join("config.key"))
        .expect("preferred config should be saved");
    save_config_to_paths(&legacy, &legacy_dir.join("config.json"), &legacy_dir.join("config.key"))
        .expect("legacy config should be saved");

    let report = load_config().expect("load should succeed");
    assert_eq!(report.config, preferred);
}

#[test]
fn load_config_falls_back_to_legacy_path_when_preferred_is_missing() {
    let env_sandbox = EnvSandbox::new();
    let legacy = AppConfig {
        google: ProviderConfig {
            api_url: "https://legacy.example/google".into(),
            api_key: "legacy-secret".into(),
            model: "legacy-model".into(),
        },
        groq: ProviderConfig::default(),
    };

    let legacy_dir = env_sandbox.legacy_dir();
    save_config_to_paths(&legacy, &legacy_dir.join("config.json"), &legacy_dir.join("config.key"))
        .expect("legacy config should be saved");

    let report = load_config().expect("load should succeed");
    assert_eq!(report.config, legacy);
}

#[test]
fn load_keeps_non_secret_fields_when_key_is_missing() {
    let sandbox = TestConfigSandbox::new();
    let config = sample_config();

    save_config_to_paths(&config, &sandbox.config_path, &sandbox.key_path).expect("save should succeed");
    fs::remove_file(&sandbox.key_path).expect("key should be removable");

    let report = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect("load should succeed");

    assert_eq!(report.config.google.api_url, config.google.api_url);
    assert_eq!(report.config.google.model, config.google.model);
    assert!(report.config.google.api_key.is_empty());
    assert_eq!(report.config.groq.api_url, config.groq.api_url);
    assert_eq!(report.config.groq.model, config.groq.model);
    assert!(report.config.groq.api_key.is_empty());
    assert!(!report.warnings.is_empty());
}

#[test]
fn load_keeps_non_secret_fields_when_key_is_invalid() {
    let sandbox = TestConfigSandbox::new();
    let config = sample_config();

    save_config_to_paths(&config, &sandbox.config_path, &sandbox.key_path).expect("save should succeed");
    fs::write(&sandbox.key_path, "broken-key").expect("broken key should be written");

    let report = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect("load should succeed");

    assert_eq!(report.config.google.api_url, config.google.api_url);
    assert_eq!(report.config.google.model, config.google.model);
    assert!(report.config.google.api_key.is_empty());
    assert_eq!(report.config.groq.api_url, config.groq.api_url);
    assert_eq!(report.config.groq.model, config.groq.model);
    assert!(report.config.groq.api_key.is_empty());
    assert!(!report.warnings.is_empty());
}

#[test]
fn load_restores_api_key_from_legacy_aad_label() {
    let sandbox = TestConfigSandbox::new();
    let (google_secret, groq_secret) = test_support::write_legacy_aad_config(&sandbox.config_path, &sandbox.key_path);

    let report = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect("load should succeed");

    assert!(report.warnings.is_empty());
    assert_eq!(report.config.google.api_key, google_secret);
    assert_eq!(report.config.groq.api_key, groq_secret);
}

#[test]
fn load_restores_api_key_from_legacy_xor_format() {
    let sandbox = TestConfigSandbox::new();
    let (google_secret, groq_secret) = test_support::write_legacy_xor_config(&sandbox.config_path, &sandbox.key_path);

    let report = load_config_from_paths(&sandbox.config_path, &sandbox.key_path).expect("load should succeed");

    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(report.config.google.api_key, google_secret);
    assert_eq!(report.config.groq.api_key, groq_secret);
}

#[test]
fn alternate_path_is_distinct_from_primary_when_xdg_is_set() {
    let _env_sandbox = EnvSandbox::new();
    let alternate = alternate_config_paths().expect("path lookup should succeed");
    assert!(alternate.is_some());
}
