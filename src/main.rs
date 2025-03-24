use std::env;

use clap::Parser;
use simple_update_checker::{
    DbConfig,
    actions::{self, add_program, run_timed},
    cli::{Cli, Command, UpdateProviderAdd},
};
use tracing::Level;

#[tokio::main]
async fn main() {
    // load environment variables
    let _ = dotenvy::dotenv();

    // setup logging
    init_logger();

    let cli = Cli::parse();

    let db_config = DbConfig::try_create(cli.db_args).unwrap();

    match cli.command {
        Command::AddProgram(add_program_args) => match &add_program_args.provider {
            UpdateProviderAdd::Github(add_github_program_args) => {
                add_program::add_program_github(
                    db_config,
                    &add_program_args,
                    add_github_program_args,
                    cli.github_access_token,
                )
                .await
            }
        },
        Command::RemoveProgram(remove_program_args) => {
            actions::remove_program(db_config, remove_program_args).await
        }
        Command::ListPrograms => actions::list_programs(db_config).await,
        Command::Check(check_args) => {
            actions::check(db_config, check_args, cli.github_access_token).await
        }
        Command::Update(update_args) => actions::update(db_config, update_args).await,
        Command::UpdateHistory(update_history_args) => {
            actions::update_history(db_config, update_history_args).await
        }
        Command::UpdateCheckHistory(update_check_history_args) => {
            actions::update_check_history(db_config, update_check_history_args).await
        }
        Command::RunTimed(run_timed_args) => {
            run_timed::run(db_config, run_timed_args, cli.github_access_token).await
        }
    }
}

fn init_logger() {
    let level = match env::var("LOG_LEVEL").unwrap_or("INFO".to_string()).as_str() {
        "TRACE" => Level::TRACE,
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        l => {
            println!(
                "Warning, LOG_LEVEL {l} not recognized. Should be one of: [TRACE, DEBUG, INFO, WARN, ERROR]. Setting log level to INFO."
            );
            Level::INFO
        }
    };

    // setup logging
    tracing_subscriber::fmt().with_max_level(level).init();
}
