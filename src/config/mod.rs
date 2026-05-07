mod crypto;
mod history_storage;
mod paths;
mod storage;

#[cfg(test)]
mod tests;

use std::io;

use crate::domain::config::{AppConfig, ConfigLoadReport};
use crate::domain::history::{HistoryEntry, HistoryLoadReport};

pub fn load_config() -> io::Result<ConfigLoadReport> {
    let paths = paths::config_paths()?;
    if paths.config_path.exists() {
        return storage::load_config_from_paths(&paths.config_path, &paths.key_path);
    }

    if let Some(alternate_paths) = paths::alternate_config_paths()?
        && alternate_paths.config_path.exists()
    {
        return storage::load_config_from_paths(
            &alternate_paths.config_path,
            &alternate_paths.key_path,
        );
    }

    storage::load_config_from_paths(&paths.config_path, &paths.key_path)
}

pub fn save_config(config: &AppConfig) -> io::Result<()> {
    let paths = paths::config_paths()?;
    storage::save_config_to_paths(config, &paths.config_path, &paths.key_path)
}

pub fn load_history() -> io::Result<HistoryLoadReport> {
    let history_path = paths::history_path()?;
    history_storage::load_history_from_path(&history_path)
}

pub fn save_history(entries: &[HistoryEntry]) -> io::Result<()> {
    let history_path = paths::history_path()?;
    history_storage::save_history_to_path(entries, &history_path)
}
