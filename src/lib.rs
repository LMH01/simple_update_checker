use std::fmt::Display;

use anyhow::Result;
use cli::DbArgs;
use config::Config;
use tabled::Tabled;

pub mod actions;
pub mod cli;
pub mod config;
pub mod db;
mod update_check;

#[derive(PartialEq, Debug, Tabled, Clone)]
pub struct Program {
    name: String,
    /// Version that is currently in use
    current_version: String,
    /// Newest version that is available
    latest_version: String,
    provider: Provider,
}

impl Program {
    pub async fn init(name: &str, provider: Provider) -> Result<Self> {
        let latest_version = provider.check_for_latest_version().await?;
        Ok(Self {
            name: name.to_string(),
            current_version: latest_version.clone(),
            latest_version: latest_version,
            provider,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Provider {
    // String contains the gihub repository. For example: LMH01/simple_update_checker
    Github(String),
}

impl Provider {
    fn identifier(&self) -> String {
        match self {
            Self::Github(_) => "github".to_string(),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

pub struct DbConfig {
    pub db_path: String,
}

impl DbConfig {
    /// Tries to create a db config by trying to load the config file from
    /// '~/.config/simple_update_checker/config.toml'.
    /// If the config is found and the cli argument '--db-path' is not set, the value from that config is taken.
    /// If the cli argument is set, its value will be used instead of the value from the config.
    pub fn try_create(db_args: DbArgs) -> Result<Self> {
        // try to load config at ~/.config/simple_update_checker/config.toml
        let db_config = match Config::try_parse() {
            Err(e) => {
                println!(
                    "Warning: unable to parse config at ~/.config/simple_update_checker/config.toml : {e}"
                );
                DbConfig::from(db_args)
            }
            Ok(Some(config)) => {
                println!("Using config file found at {}", config.path);
                // check if db_path is set using cli
                if let Some(db_path) = &db_args.db_path {
                    println!(
                        "Not using db_path setting found in config file ({}) as --db-path is set ({})",
                        config.db_path, db_path
                    );
                    DbConfig::from(db_args)
                } else {
                    DbConfig {
                        db_path: config.db_path,
                    }
                }
            }
            Ok(None) => DbConfig::from(db_args),
        };

        println!("Using database file: {}", db_config.db_path);

        Ok(db_config)
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            db_path: "programs.db".to_string(),
        }
    }
}

impl From<DbArgs> for DbConfig {
    fn from(value: DbArgs) -> Self {
        if let Some(path) = value.db_path {
            return DbConfig { db_path: path };
        }
        DbConfig::default()
    }
}
