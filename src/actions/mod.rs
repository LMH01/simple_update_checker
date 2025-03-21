use std::process;

use tabled::{Table, Tabled};

use crate::{
    cli::{CheckArgs, DbArgs, RemoveProgramArgs},
    db::ProgramDb,
    Provider,
};

pub mod add_program;

pub async fn remove_program(db_args: DbArgs, remove_program_args: RemoveProgramArgs) {
    let db = ProgramDb::connect(&db_args.db_path).await.unwrap();
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

pub async fn list_programs(db_args: DbArgs) {
    let db = ProgramDb::connect(&db_args.db_path).await.unwrap();
    let programs = db.get_all_programs().await.unwrap();
    println!("The following programs are currently stored in the database:\n");
    let table = Table::new(programs);
    println!("{}\n", table);
    println!("Note: the latest_version displayed here might not necessarily be the actual newest version. Use command 'check' to check all programs for updates.");
}

#[derive(Tabled, Clone)]
struct CheckedProgram {
    name: String,
    last_version: String,
    latest_version: String,
    provider: Provider,
}

pub async fn check(db_args: DbArgs, check_args: CheckArgs) {
    let db = ProgramDb::connect(&db_args.db_path).await.unwrap();
    let programs = db.get_all_programs().await.unwrap();
    println!("Checking {} programs for updates...", programs.len());
    let mut checked_programs: Vec<CheckedProgram> = programs
        .into_iter()
        .map(|p| CheckedProgram {
            name: p.name,
            last_version: p.latest_version.clone(),
            latest_version: p.latest_version,
            provider: p.provider,
        })
        .collect();
    let mut programs_with_updates = Vec::new();
    checked_programs.sort_by(|a, b| a.name.cmp(&b.name));

    let mut updates_available = false;

    for program in &mut checked_programs {
        let latest_version = match program.provider.check_for_latest_version().await {
            Ok(latest_version) => latest_version,
            Err(e) => {
                println!("Error while checking for latest version: {e:?}");
                process::exit(1);
            }
        };
        if latest_version != program.last_version {
            if !check_args.no_update_db {
                db.update_latest_version(&program.name, &latest_version)
                    .await
                    .unwrap();
            }
            program.latest_version = latest_version;
            println!(
                "{}: update found {} -> {}",
                program.name, program.last_version, program.latest_version
            );
            programs_with_updates.push(program.clone());
            updates_available = true;
        } else {
            println!("{}: no update found", program.name);
        }
    }

    if updates_available {
        println!("\nSummary of programs that have updates available:\n");
        let table = Table::new(programs_with_updates);
        println!("{}", table);
    }
}
