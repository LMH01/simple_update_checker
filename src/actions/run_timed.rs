use std::{process, time::Duration};

use anyhow::Result;
use sqlx::types::chrono::Utc;
use tabled::Table;
use tokio::signal::unix::{SignalKind, signal};

use crate::{
    DbConfig, Program, UpdateCheckType, cli::RunTimedArgs, db::ProgramDb, notification,
    update_check,
};

pub async fn run(
    db_config: DbConfig,
    run_timed_args: RunTimedArgs,
    github_access_token: Option<String>,
) {
    // check connection with database before starting thread
    tracing::info!("Checking database connection");
    match ProgramDb::connect(&db_config.db_path).await {
        Err(e) => {
            tracing::error!("Error while connecting to database: {e}");
            process::exit(1);
        }
        Ok(db) => {
            tracing::info!("Database connection successful. Currently watched programs:");
            let programs = db.get_all_programs().await.unwrap();
            let table = Table::new(programs);
            tracing::info!("\n{table}");
        }
    }

    spawn(db_config, run_timed_args, github_access_token);

    // setup signal handlers
    let mut sigterm =
        signal(SignalKind::terminate()).expect("Unable to setup SIGTERM signal handler");
    let mut sigint =
        signal(SignalKind::interrupt()).expect("Unable to setup SIGINT signal handler");

    // wait for signals
    tracing::info!("Waiting for shutdown signal");
    tokio::select! {
        _ = sigterm.recv() => tracing::info!("Received SIGTERM"),
        _ = sigint.recv() => tracing::info!("Received SIGINT"),
    }
    tracing::info!("Received shutdown signal, shutting down");
}

/// Spawn the tread that periodically checks for updates
fn spawn(db_config: DbConfig, run_timed_args: RunTimedArgs, github_access_token: Option<String>) {
    tokio::spawn(async move {
        tracing::info!(
            "Starting update checker loop, check interval: {} seconds",
            run_timed_args.check_interval
        );
        loop {
            tracing::info!("Starting update check");
            if let Err(e) =
                check_for_updates(&db_config, &run_timed_args, &github_access_token).await
            {
                tracing::error!("Error while checking for updates: {e}");
                if let Err(e) = notification::send_error_notifictaion(
                    &run_timed_args.ntfy_topic,
                    &e.to_string(),
                )
                .await
                {
                    tracing::error!("Error while sending notification: {e}");
                }
            }
            tracing::info!(
                "Starting next update check in {} seconds",
                run_timed_args.check_interval
            );
            tokio::time::sleep(Duration::from_secs(run_timed_args.check_interval as u64)).await
        }
    });
}

async fn check_for_updates(
    db_config: &DbConfig,
    run_timed_args: &RunTimedArgs,
    github_access_token: &Option<String>,
) -> Result<()> {
    let db = ProgramDb::connect(&db_config.db_path).await?;
    let mut programs = db.get_all_programs().await?;
    programs.sort_by(|a, b| a.name.cmp(&b.name));
    tracing::info!("Checking {} programs for updates...", programs.len());

    let programs_with_available_updates = update_check::check_for_updates(
        &db,
        None,
        github_access_token,
        false,
        UpdateCheckType::Timed,
    )
    .await?;

    let available_updates = programs_with_available_updates.len();

    if !programs_with_available_updates.is_empty() {
        tracing::info!("Found updates for the following programs:");
        let table = Table::new(&programs_with_available_updates);
        tracing::info!("\n{table}");
        send_update_notification(
            &db,
            &run_timed_args.ntfy_topic,
            &programs_with_available_updates,
        )
        .await?;
    }
    tracing::info!("Found {} updates", available_updates);
    Ok(())
}

async fn send_update_notification(
    db: &ProgramDb,
    topic: &str,
    programs: &Vec<Program>,
) -> Result<()> {
    let mut message = String::new();
    let mut programs_with_notifications_to_sent = Vec::new();
    for program in programs {
        // only add program to notification if notification for that program was not yet sent
        let notification_info = match db.get_notification_info(&program.name).await? {
            Some(notification_sent) => notification_sent,
            None => anyhow::bail!("Unable to find program {} in database", program.name),
        };
        if notification_info.sent {
            if let Some(sent_on) = notification_info.sent_on {
                tracing::debug!(
                    "Not adding {} to notification as notification was already sent on {}",
                    program.name,
                    crate::format_datetime(&sent_on),
                )
            } else {
                tracing::debug!(
                    "Not adding {} to notifications as program was manually checked for updates",
                    program.name
                )
            }
        } else {
            message.push_str(&format!(
                "{}: {} -> {}\n",
                program.name, program.current_version, program.latest_version
            ));
            programs_with_notifications_to_sent.push(program);
        }
    }
    if programs_with_notifications_to_sent.is_empty() {
        tracing::debug!(
            "Not sending push notifications as no updates are available for which notifications where not already sent"
        );
    } else {
        tracing::info!("Sending push notification to topic {}", topic);
        match notification::send_update_notification(topic, &message).await {
            Ok(()) => {
                // mark programs with updates available as notification sent
                for program in programs_with_notifications_to_sent {
                    db.set_notification_sent(&program.name, true).await?;
                    db.set_notification_sent_on(&program.name, Some(Utc::now().naive_utc()))
                        .await?;
                }
            }
            Err(e) => {
                // error while sending notifications, so we don't mark the notifications as sent
                anyhow::bail!(e);
            }
        }
    }
    Ok(())
}
