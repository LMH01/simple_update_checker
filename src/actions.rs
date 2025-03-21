use std::process;

use crate::{
    cli::{AddGithubProgramArgs, AddProgramArgs, RemoveProgramArgs},
    db::ProgramDb,
    Program, Provider,
};

pub async fn add_program_github(
    add_program_args: &AddProgramArgs,
    add_github_program_args: &AddGithubProgramArgs,
) {
    let db = ProgramDb::connect(&add_program_args.db_args.db_path)
        .await
        .unwrap();

    if db
        .get_program(&add_program_args.name)
        .await
        .unwrap()
        .is_some()
    {
        println!(
            "Program named {} already exists in database.",
            &add_program_args.name
        );
        process::exit(0);
    }

    let program = Program::init(
        &add_program_args.name,
        Provider::Github(add_github_program_args.repository.to_string()),
    )
    .await
    .unwrap();

    db.add_program(&program).await.unwrap();
    println!(
        "Program {} successfully added to database!",
        &add_program_args.name
    );
}

pub async fn remove_program(remove_program_args: RemoveProgramArgs) {
    let db = ProgramDb::connect(&remove_program_args.db_args.db_path)
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
