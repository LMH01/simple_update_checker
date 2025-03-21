use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

use crate::Provider;

impl Provider {
    // Checks what the latest version for the program using this provider is.
    pub async fn check_for_latest_version(&self) -> Result<String> {
        match self {
            Self::Github(repo) => {
                let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
                let response = Client::new()
                    .get(&url)
                    .header("User-Agent", "reqwest")
                    .send()
                    .await?;

                if response.status().is_success() {
                    let json: Value = response.json().await?;
                    if let Some(tag_name) = json["tag_name"].as_str() {
                        return Ok(tag_name.to_string());
                    } else {
                        return Err(anyhow::anyhow!(
                            "Response was success but did not contain tag_name"
                        ));
                    }
                }
                Err(anyhow::anyhow!("Request failed with error: {response:?}"))
            }
        }
    }
}
