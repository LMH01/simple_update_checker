use clap::Parser;
use cli::Cli;

mod cli;
mod db;

fn main() {

    let _cli = Cli::parse();
    
    println!("Hello, world!");
}
