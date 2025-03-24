use std::process;

use sqlx::types::chrono::Utc;
use tabled::Table;

use crate::{
    DbConfig, Identifier, UpdateCheckType, UpdateHistoryEntry,
    cli::{CheckArgs, RemoveProgramArgs, UpdateArgs, UpdateCheckHistoryArgs, UpdateHistoryArgs},
    db::Db,
    update_check,
};

pub mod add_program;
pub mod run_timed;

pub async fn remove_program(db_config: DbConfig, remove_program_args: RemoveProgramArgs) {
    let db = Db::connect(&db_config.db_path).await.unwrap();
    if db
        .get_program(&remove_program_args.name)
        .await
        .unwrap()
        .is_none()
    {
        println!(
            "Program {} did not exist in database.",
            &remove_program_args.name
        );
        process::exit(0);
    }
    db.remove_program(&remove_program_args.name).await.unwrap();
    println!(
        "Program {} has been removed from the database.",
        &remove_program_args.name
    );
}

pub async fn list_programs(db_config: DbConfig) {
    let db = Db::connect(&db_config.db_path).await.unwrap();
    let mut programs = db.get_all_programs().await.unwrap();
    programs.sort_by(|a, b| a.name.cmp(&b.name));
    println!("The following programs are currently stored in the database:\n");
    let table = Table::new(programs);
    println!("{}\n", table);

    if let Some(last_update_check) = db.get_latest_update_check_from_history().await.unwrap() {
        println!(
            "Last update check performed on: (UTC) {} ({} update check)",
            last_update_check.date.format("%Y-%m-%d %H:%M:%S"),
            last_update_check.r#type.identifier()
        );
    } else {
        println!("Last update check performed on: never")
    }
    println!("\nUse command 'check' to check all programs for updates.")
}

pub async fn check(db_args: DbConfig, check_args: CheckArgs, github_access_token: Option<String>) {
    let db = Db::connect(&db_args.db_path).await.unwrap();
    let mut programs = db.get_all_programs().await.unwrap();
    programs.sort_by(|a, b| a.name.cmp(&b.name));
    println!("Checking {} programs for updates...", programs.len());

    let programs_with_available_updates = update_check::check_for_updates(
        &db,
        Some(check_args),
        &github_access_token,
        true,
        UpdateCheckType::Manual,
    )
    .await
    .unwrap();

    if !programs_with_available_updates.is_empty() {
        println!("\nSummary of programs that have updates available:\n");
        let table = Table::new(programs_with_available_updates);
        println!("{}", table);
    }
}

pub async fn update(db_config: DbConfig, update_args: UpdateArgs) {
    let db = Db::connect(&db_config.db_path).await.unwrap();
    if db.get_program(&update_args.name).await.unwrap().is_none() {
        println!(
            "Unable to update current_version: Program {} does not exist in database.",
            &update_args.name
        );
        process::exit(0);
    }
    let program = db.get_program(&update_args.name).await.unwrap().unwrap();
    if program.current_version.eq(&program.latest_version) {
        println!(
            "current_version of {} is already equal to latest_version",
            &program.name
        );
        process::exit(0);
    }
    db.update_current_version(
        &update_args.name,
        &program.latest_version,
        Utc::now().naive_utc(),
    )
    .await
    .unwrap();
    db.insert_performed_update(&UpdateHistoryEntry {
        date: Utc::now().naive_utc(),
        name: program.name.clone(),
        old_version: program.current_version,
        updated_to: program.latest_version.clone(),
    })
    .await
    .unwrap();
    println!(
        "current_version of {} has been updated to latest version ({})",
        &program.name, &program.latest_version
    );
}

pub async fn update_history(db_config: DbConfig, update_history_args: UpdateHistoryArgs) {
    let db = Db::connect(&db_config.db_path).await.unwrap();
    let mut updates = db
        .get_all_updates(Some(update_history_args.max_entries))
        .await
        .unwrap();
    updates.reverse();
    println!(
        "Showing the latest {} performed updates:\n(Newest update at the bottom)\n",
        update_history_args.max_entries
    );
    let table = Table::new(updates);
    println!("{}\n", table);
}

pub async fn update_check_history(
    db_config: DbConfig,
    update_check_history_args: UpdateCheckHistoryArgs,
) {
    let db = Db::connect(&db_config.db_path).await.unwrap();
    let mut updates = db
        .get_all_update_checks(Some(update_check_history_args.max_entries))
        .await
        .unwrap();
    updates.reverse();
    println!(
        "Showing the latest {} performed update checks:\n(Newest update check at the bottom)\n",
        update_check_history_args.max_entries
    );
    let table = Table::new(updates);
    println!("{}\n", table);
}
