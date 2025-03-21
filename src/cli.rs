use clap::{Parser, Subcommand};

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
    Run(DbArgs),
    #[command{
        about = "Periodically check all programs for updates.",
        long_about = "Periodically check all programs for updates. Sends a push notification when updates are found and the ntfy.sh topic is configured."
    }]
    RunTimed(RunTimedArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct AddProgramArgs {
    #[command(flatten)]
    pub db_args: DbArgs,
    
    #[command(subcommand)]
    pub provider: UpdateProviderAdd,
    
    #[arg(short, long, help = "Display name for the program")]
    pub name: String,
}

#[derive(Parser, Debug, Clone)]
pub struct RemoveProgramArgs {
    #[command(flatten)]
    pub db_args: DbArgs,

    #[arg(short, long, help = "Name of the program that should no longer be checked for updates")]
    pub name: String
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
pub struct RunTimedArgs {
    #[command(flatten)]
    pub db_args: DbArgs,

    #[arg{
        short,
        long,
        help = "Topic under which the update checks should be published.",
        env
    }]
    pub ntfy_topic: String,
}

#[derive(Parser, Debug, Clone)]
pub struct DbArgs {
    #[arg{
        short,
        long,
        help = "Path where 'programs.db' is located that contains the programs that should be checked for updates and their latest versions.",
        default_value = "programs.db"
    }]
    pub db_path: String,
}
