use std::fmt::Display;

use anyhow::Result;
use tabled::Tabled;

pub mod actions;
pub mod cli;
pub mod db;
mod update_check;

#[derive(PartialEq, Debug, Tabled, Clone)]
pub struct Program {
    name: String,
    latest_version: String,
    provider: Provider,
}

impl Program {
    pub async fn init(name: &str, provider: Provider) -> Result<Self> {
        let latest_version = provider.check_for_latest_version().await?;
        Ok(Self {
            name: name.to_string(),
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
