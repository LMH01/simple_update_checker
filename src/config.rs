use std::fs;

use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub db_path: String,
}

impl Config {
    /// Tries to load the config located at ~/.config/simple_update_checker/config.toml
    ///
    /// ## Returns
    /// - `Ok(Config)` when the config exists and could be parsed.
    /// - `Ok(None)` when the config does not exit.
    /// - `Err(e)` when the config exists but could not be parsed.
    pub fn try_parse() -> Result<Option<Self>> {
        let base_dirs = match BaseDirs::new() {
            Some(base_dirs) => base_dirs,
            None => anyhow::bail!("Home directory path could not be determined"),
        };
        let config_file = base_dirs
            .config_dir()
            .join("simple_update_checker/config.toml");
        if !config_file.exists() {
            return Ok(None);
        }

        let config = toml::from_str::<Config>(&fs::read_to_string(config_file)?)?;

        Ok(Some(config))
    }
}
