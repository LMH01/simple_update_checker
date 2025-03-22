use clap::Parser;
use simple_update_checker::{
    DbConfig,
    actions::{self, add_program},
    cli::{Cli, Command, UpdateProviderAdd},
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let db_config = DbConfig::try_create(cli.db_args).unwrap();

    match cli.command {
        Command::AddProgram(add_program_args) => match &add_program_args.provider {
            UpdateProviderAdd::Github(add_github_program_args) => {
                add_program::add_program_github(
                    db_config,
                    &add_program_args,
                    add_github_program_args,
                )
                .await
            }
        },
        Command::RemoveProgram(remove_program_args) => {
            actions::remove_program(db_config, remove_program_args).await
        }
        Command::ListPrograms => actions::list_programs(db_config).await,
        Command::Check(check_args) => actions::check(db_config, check_args).await,
        Command::RunTimed(_run_timed_args) => todo!("This command is not yet implemented."),
    }
}
