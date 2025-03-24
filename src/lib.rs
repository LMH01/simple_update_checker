use std::{fmt::Display, str::FromStr, vec};

use anyhow::Result;
use cli::DbArgs;
use config::Config;
use sqlx::{
    prelude::FromRow,
    types::chrono::{NaiveDateTime, Utc},
};
use tabled::Tabled;

pub mod actions;
pub mod cli;
pub mod config;
pub mod db;
mod notification;
mod update_check;

#[derive(PartialEq, Debug, Tabled, Clone)]
pub struct Program {
    #[tabled(rename = "Name")]
    name: String,
    /// Version that is currently in use
    #[tabled(rename = "Current version (CV)")]
    current_version: String,
    /// Last time '`current_version`' was updated
    #[tabled(rename = "CV last updated", display("format_datetime"))]
    current_version_last_updated: NaiveDateTime,
    /// Newest version that is available
    #[tabled(rename = "Latest version (LV)")]
    latest_version: String,
    /// Last time '`current_version`' was updated
    #[tabled(rename = "LV last updated", display("format_datetime"))]
    latest_version_last_updated: NaiveDateTime,
    #[tabled(rename = "Provider")]
    provider: Provider,
}

impl Program {
    pub async fn init(
        name: &str,
        provider: Provider,
        github_access_token: Option<String>,
    ) -> Result<Self> {
        let latest_version = provider
            .check_for_latest_version(&github_access_token)
            .await?;
        Ok(Self {
            name: name.to_string(),
            current_version: latest_version.clone(),
            current_version_last_updated: Utc::now().naive_utc(),
            latest_version,
            latest_version_last_updated: Utc::now().naive_utc(),
            provider,
        })
    }
}

#[must_use]
pub fn format_datetime(value: &NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Returns an identifier for this type.
pub trait Identifier {
    fn identifier(&self) -> String;
}

#[derive(PartialEq, Debug, Clone)]
pub enum Provider {
    // String contains the gihub repository. For example: LMH01/simple_update_checker
    Github(String),
}

impl Identifier for Provider {
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
    /// '~/.`config/simple_update_checker/config.toml`'.
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

/// Represents a single update check.
#[derive(FromRow, Debug, PartialEq, Tabled)]
pub struct UpdateCheckHistoryEntry {
    #[tabled(rename = "Date", display("format_datetime"))]
    pub date: NaiveDateTime,
    #[tabled(rename = "Type")]
    pub r#type: UpdateCheckType,
    #[tabled(rename = "Updates available")]
    pub updates_available: u32,
    #[tabled(rename = "Programs")]
    pub programs: String,
}

impl UpdateCheckHistoryEntry {
    /// Creates a new `UpdateCheck` entry from the current time and date.
    #[must_use]
    pub fn from_now(r#type: UpdateCheckType, mut programs_with_updates: Vec<Program>) -> Self {
        programs_with_updates.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            date: Utc::now().naive_utc(),
            r#type,
            updates_available: programs_with_updates.len() as u32,
            programs: programs_with_updates
                .into_iter()
                .map(|f| f.name)
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UpdateCheckType {
    Manual,
    Timed,
}

impl Identifier for UpdateCheckType {
    fn identifier(&self) -> String {
        match self {
            UpdateCheckType::Manual => "manual".to_string(),
            UpdateCheckType::Timed => "timed".to_string(),
        }
    }
}

impl Display for UpdateCheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

impl FromStr for UpdateCheckType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "manual" => Ok(UpdateCheckType::Manual),
            "timed" => Ok(UpdateCheckType::Timed),
            _ => Err(anyhow::anyhow!(
                "UpdateCheckType could not be parsed from {s}"
            )),
        }
    }
}

pub struct NotificationInfo {
    pub sent: bool,
    pub sent_on: Option<NaiveDateTime>,
}

#[derive(FromRow, Debug, PartialEq, Tabled)]
pub struct UpdateHistoryEntry {
    #[tabled(rename = "Date", display("format_datetime"))]
    pub date: NaiveDateTime,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Old version")]
    pub old_version: String,
    #[tabled(rename = "Updated to")]
    pub updated_to: String,
}
