use clap::Parser;
use simple_update_checker::{
    actions,
    cli::{Cli, Command, UpdateProviderAdd},
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::AddProgram(add_program_args) => match &add_program_args.provider {
            UpdateProviderAdd::Github(add_github_program_args) => {
                actions::add_program_github(&add_program_args, add_github_program_args)
                    .await
                    .unwrap()
            }
        },
        _ => (),
    }
}
