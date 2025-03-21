use std::process;

use crate::{cli::{DbArgs, RemoveProgramArgs}, db::ProgramDb};

pub mod add_program;

pub async fn remove_program(db_args: DbArgs, remove_program_args: RemoveProgramArgs) {
    let db = ProgramDb::connect(&db_args.db_path)
        .await
        .unwrap();
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

//pub async fn list_programs() {
//    let db = ProgramDb::connect(&)
//}