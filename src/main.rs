use clap::Parser;
use simple_update_checker::{
    actions::{self, add_program},
    cli::{Cli, Command, UpdateProviderAdd},
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::AddProgram(add_program_args) => match &add_program_args.provider {
            UpdateProviderAdd::Github(add_github_program_args) => {
                add_program::add_program_github(
                    cli.db_args,
                    &add_program_args,
                    add_github_program_args,
                )
                .await
            }
        },
        Command::RemoveProgram(remove_program_args) => {
            actions::remove_program(cli.db_args, remove_program_args).await
        }
        Command::ListPrograms => actions::list_programs(cli.db_args).await,
        Command::Check(check_args) => actions::check(cli.db_args, check_args).await,
        _ => (),
    }
}
