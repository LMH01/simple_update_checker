use clap::Parser;
use simple_update_checker::{
    cli::{Cli, Command, UpdateProviderAdd},
    db::ProgramDb,
    Program, Provider,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::AddProgram(add_program_args) => match add_program_args.provider {
            UpdateProviderAdd::Github(add_github_program_args) => {
                let db = ProgramDb::connect(&add_program_args.db_args.db_path)
                    .await
                    .unwrap();
                let program = Program::init(
                    &add_program_args.name,
                    Provider::Github(add_github_program_args.repository),
                )
                .await
                .unwrap();

                db.add_program(&program).await.unwrap();
                println!(
                    "Program {} successfully added to database!",
                    &add_program_args.name
                );
            }
        },
        _ => (),
    }
}
