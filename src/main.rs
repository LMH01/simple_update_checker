use clap::Parser;
use cli::Cli;

mod cli;

fn main() {

    let _cli = Cli::parse();
    
    println!("Hello, world!");
}
