use anyhow::Result;

use crate::{
    cli::{AddGithubProgramArgs, AddProgramArgs},
    db::ProgramDb,
    Program, Provider,
};

pub async fn add_program_github(
    add_program_args: &AddProgramArgs,
    add_github_program_args: &AddGithubProgramArgs,
) -> Result<()> {
    let db = ProgramDb::connect(&add_program_args.db_args.db_path)
        .await
        .unwrap();
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
    Ok(())
}
