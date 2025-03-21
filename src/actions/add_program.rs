use std::process;

use crate::{
    cli::{AddGithubProgramArgs, AddProgramArgs},
    db::ProgramDb,
    DbConfig, Program, Provider,
};

pub async fn add_program_github(
    db_config: DbConfig,
    add_program_args: &AddProgramArgs,
    add_github_program_args: &AddGithubProgramArgs,
) {
    let db = ProgramDb::connect(&db_config.db_path).await.unwrap();

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
