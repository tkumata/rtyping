mod crypto;
mod paths;
mod storage;

#[cfg(test)]
mod tests;

use std::io;

use crate::domain::config::{AppConfig, ConfigLoadReport};

pub fn load_config() -> io::Result<ConfigLoadReport> {
    let paths = paths::config_paths()?;
    if paths.config_path.exists() {
        return storage::load_config_from_paths(&paths.config_path, &paths.key_path);
    }

    if let Some(alternate_paths) = paths::alternate_config_paths()?
        && alternate_paths.config_path.exists()
    {
        return storage::load_config_from_paths(&alternate_paths.config_path, &alternate_paths.key_path);
    }

    storage::load_config_from_paths(&paths.config_path, &paths.key_path)
}

pub fn save_config(config: &AppConfig) -> io::Result<()> {
    let paths = paths::config_paths()?;
    storage::save_config_to_paths(config, &paths.config_path, &paths.key_path)
}
