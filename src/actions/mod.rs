use std::process;

use tabled::Table;

use crate::{
    cli::{DbArgs, RemoveProgramArgs},
    db::ProgramDb,
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
