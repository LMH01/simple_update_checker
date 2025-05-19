use clap::{Parser, Subcommand};

use crate::config::ConfigFile;

#[derive(Parser, Debug)]
#[command(
    author = "LMH01",
    version,
    about,
    long_about = "Simple program that can be used to automatically check for updates of programs. Optionally allows to send push notifications using ntfy.sh when an update is found."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub db_args: DbArgs,

    #[arg(
        short,
        long,
        help = "Set to increase rate limit of github api.\nSee https://github.com/settings/personal-access-tokens",
        env
    )]
    pub github_access_token: Option<String>,
}

impl Cli {

    /// Applies the values set in the provided config file.
    /// 
    /// If a value is defined in the cli and in the config file, the value provided by the cli will take precedence.
    pub fn apply_config_file(&mut self, config_file: ConfigFile) {
        if self.github_access_token.is_none() && config_file.github_access_token.is_some() {
            self.github_access_token = config_file.github_access_token;
        }
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    #[command(
        about = "Add a program to the database that should be checked for updates. Sets the latest version to the latest version currently available.",
        subcommand_value_name = "PROVIDER"
    )]
    AddProgram(AddProgramArgs),
    #[command(
        about = "Remove a program from the database that should no longer be checked for updates.",
        subcommand_value_name = "PROVIDER"
    )]
    RemoveProgram(RemoveProgramArgs),
    #[command(about = "Lists all programs that are checked for updates.")]
    ListPrograms,
    #[command{
        about = "Check all programs once for updates.",
        long_about = "Check all programs once for updates. Does not send a push notification when updates are found."
    }]
    Check(CheckArgs),
    #[command(about = "Update current_version of a program to the currently found latest_version.")]
    Update(UpdateArgs),
    #[command(about = "Show the history of performed updates.")]
    UpdateHistory(UpdateHistoryArgs),
    #[command(about = "Show the history of performed updates checks.")]
    UpdateCheckHistory(UpdateCheckHistoryArgs),
    #[command{
        about = "Periodically check all programs for updates.",
        long_about = "Periodically check all programs for updates. Sends a push notification when updates are found and the ntfy.sh topic is configured."
    }]
    RunTimed(RunTimedArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct AddProgramArgs {
    #[command(subcommand)]
    pub provider: UpdateProviderAdd,

    #[arg(short, long, help = "Display name for the program")]
    pub name: String,
}

#[derive(Parser, Debug, Clone)]
pub struct RemoveProgramArgs {
    #[arg(
        short,
        long,
        help = "Name of the program that should no longer be checked for updates"
    )]
    pub name: String,
}

#[derive(Parser, Debug, Clone)]
pub enum UpdateProviderAdd {
    #[command{
        about = "Use Github as provider for update information"
    }]
    Github(AddGithubProgramArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct AddGithubProgramArgs {
    #[arg(
        short,
        long,
        help = "Github repository where the program can be found and where the latest version is taken from"
    )]
    pub repository: String,
}

#[derive(Parser, Debug, Clone)]
pub struct CheckArgs {
    #[arg{
        short,
        long,
        help = "When set, the newest found version will also be set as current version.",
        env
    }]
    pub set_current_version: bool,

    #[arg{
        short,
        long,
        help = "Normally notifications are not sent in run-timed mode for updates that where seen manually.\nSet this flag to not mark the update as seen and to make the notification get sent when run-timed mode is used the next time.",
        env
    }]
    pub allow_notification: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct UpdateArgs {
    #[arg(
        short,
        long,
        help = "Name of the program for which the current_version should be set to latest_version."
    )]
    pub name: String,
}

#[derive(Parser, Debug, Clone)]
pub struct UpdateHistoryArgs {
    #[arg(
        short,
        long,
        help = "How many entries should be shown at max.",
        default_value = "20"
    )]
    pub max_entries: u32,
}

#[derive(Parser, Debug, Clone)]
pub struct UpdateCheckHistoryArgs {
    #[arg(
        short,
        long,
        help = "How many entries should be shown at max.",
        default_value = "20"
    )]
    pub max_entries: u32,
}

#[derive(Parser, Debug, Clone)]
pub struct RunTimedArgs {
    #[arg{
        short,
        long,
        help = "Topic under which the update checks should be published.",
        env
    }]
    pub ntfy_topic: String,
    #[arg(
        env,
        short,
        long,
        help = "Interval in which the update check should be run. Time in seconds.",
        default_value = "3600",
        env
    )]
    pub check_interval: u32,
}

#[derive(Parser, Debug, Clone)]
pub struct DbArgs {
    #[arg{
        short,
        long,
        help = "Path where 'programs.db' is located that contains the programs that should be checked for updates and their latest versions. If not set and config file not existing will default to 'programs.db'.",
        env
    }]
    pub db_path: Option<String>,
}
