use std::{process, time::Duration};

use anyhow::Result;
use tabled::Table;
use tokio::signal::unix::{SignalKind, signal};

use crate::{DbConfig, Program, cli::RunTimedArgs, db::ProgramDb, notification, update_check};

pub async fn run(db_config: DbConfig, run_timed_args: RunTimedArgs) {
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

    spawn(db_config, run_timed_args);

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
fn spawn(db_config: DbConfig, run_timed_args: RunTimedArgs) {
    tokio::spawn(async move {
        tracing::info!(
            "Starting update checker loop, check interval: {} seconds",
            run_timed_args.check_interval
        );
        loop {
            tracing::info!("Starting update check");
            if let Err(e) = check_for_updates(&db_config, &run_timed_args).await {
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

async fn check_for_updates(db_config: &DbConfig, run_timed_args: &RunTimedArgs) -> Result<()> {
    let db = ProgramDb::connect(&db_config.db_path).await.unwrap();
    let mut programs = db.get_all_programs().await.unwrap();
    programs.sort_by(|a, b| a.name.cmp(&b.name));
    tracing::info!("Checking {} programs for updates...", programs.len());

    let programs_with_available_updates = update_check::check_for_updates(&db, None, false)
        .await
        .unwrap();

    let available_updates = programs_with_available_updates.len();

    if !programs_with_available_updates.is_empty() {
        tracing::info!("Found updates for the following programs:");
        let table = Table::new(&programs_with_available_updates);
        tracing::info!("\n{table}");
        send_update_notification(&run_timed_args.ntfy_topic, &programs_with_available_updates)
            .await?;
    }
    tracing::info!("Found {} updates", available_updates);
    Ok(())
}

async fn send_update_notification(topic: &str, programs: &Vec<Program>) -> Result<()> {
    let mut message = String::new();
    for program in programs {
        message.push_str(&format!(
            "{}: {} -> {}\n",
            program.name, program.current_version, program.latest_version
        ));
    }
    tracing::info!("Sending push notification to topic {}", topic);
    notification::send_update_notification(topic, &message).await?;
    Ok(())
}
