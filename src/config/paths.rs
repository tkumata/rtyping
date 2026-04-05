use std::env;
use std::io;
use std::path::PathBuf;

pub(super) struct ConfigPaths {
    pub config_path: PathBuf,
    pub key_path: PathBuf,
}

pub(super) fn config_paths() -> io::Result<ConfigPaths> {
    let base_dir = preferred_config_base_dir()?;
    Ok(ConfigPaths {
        config_path: base_dir.join("config.json"),
        key_path: base_dir.join("config.key"),
    })
}

pub(super) fn alternate_config_paths() -> io::Result<Option<ConfigPaths>> {
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
