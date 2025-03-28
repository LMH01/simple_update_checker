use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use sqlx::types::chrono::Utc;

use crate::{Program, Provider, UpdateCheckHistoryEntry, UpdateCheckType, cli::CheckArgs, db::Db};

impl Provider {
    // Checks what the latest version for the program using this provider is.
    pub async fn check_for_latest_version(
        &self,
        github_access_token: &Option<String>,
    ) -> Result<String> {
        match self {
            Self::Github(repo) => {
                let url = format!("https://api.github.com/repos/{repo}/releases/latest");
                let mut request = Client::new().get(&url).header("User-Agent", "reqwest");

                if let Some(token) = github_access_token {
                    request = request.header("Authorization", format!("Bearer {token}"));
                };
                let response = request.send().await?;

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

/// Checks all programs in the database for updates. Updates `latest_version` when update was found.
/// Returns a vector containing all programs for which updates are available.
pub async fn check_for_updates(
    db: &Db,
    check_args: Option<CheckArgs>,
    github_access_token: &Option<String>,
    print_messages: bool,
    update_check_type: UpdateCheckType,
) -> Result<Vec<Program>> {
    let mut programs = db.get_all_programs().await.unwrap();
    programs.sort_by(|a, b| a.name.cmp(&b.name));

    let mut programs_with_available_updates = Vec::new();

    for mut program in programs {
        let latest_version = program
            .provider
            .check_for_latest_version(github_access_token)
            .await?;
        if latest_version != program.latest_version {
            // new version found that does not yet exist in database
            // reset notification info as new version is available and notification for that version was not yet sent

            db.set_notification_sent(&program.name, false).await?;
            db.set_notification_sent_on(&program.name, None).await?;

            // update version in db
            db.update_latest_version(&program.name, &latest_version, Utc::now().naive_utc())
                .await
                .unwrap();
            if let Some(check_args) = &check_args {
                if check_args.set_current_version {
                    db.update_current_version(
                        &program.name,
                        &latest_version,
                        Utc::now().naive_utc(),
                    )
                    .await
                    .unwrap();
                }
            }
            program.latest_version = latest_version;
            if print_messages {
                println!(
                    "{}: update found {} -> {}",
                    program.name, program.current_version, program.latest_version
                );
            }

            // if update check was performed manually we don't want so sent a notification when timed mode is run
            // so we set notification sent to true
            if update_check_type == UpdateCheckType::Manual {
                if let Some(check_args) = &check_args {
                    if !check_args.allow_notification {
                        db.set_notification_sent(&program.name, true).await?;
                    }
                }
            }

            programs_with_available_updates.push(program);
        } else if latest_version != program.current_version {
            // newest latest_version already exists in database but program has not been updated yet
            if print_messages {
                println!(
                    "{}: update found {} -> {}",
                    program.name, program.current_version, program.latest_version
                );
            }

            // if update check was performed manually we don't want so sent a notification when timed mode is run
            // so we set notification sent to true
            if update_check_type == UpdateCheckType::Manual {
                if let Some(check_args) = &check_args {
                    if !check_args.allow_notification {
                        db.set_notification_sent(&program.name, true).await?;
                    }
                }
            }

            programs_with_available_updates.push(program);
        } else if print_messages {
            println!("{}: no update found", program.name);
        }
    }

    // add entry to database that update check was performed
    db.insert_update_check_history(&UpdateCheckHistoryEntry::from_now(
        update_check_type,
        programs_with_available_updates.clone(),
    ))
    .await?;

    Ok(programs_with_available_updates)
}
