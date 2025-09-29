use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process;

mod cli;
mod repl;

use cli::CliError;

#[derive(Parser)]
#[command(name = "lugli")]
#[command(about = "Lugli programming language interpreter")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { file: String },
    Repl,
    Check { file: String },
    Version,
}

fn main() {
    let cli = Cli::parse();

    match run_command(cli.command) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            process::exit(1);
        }
    }
}

fn run_command(command: Commands) -> Result<(), CliError> {
    match command {
        Commands::Run { file } => cli::run_file(&file),
        Commands::Repl => repl::start(),
        Commands::Check { file } => cli::check_file(&file),
        Commands::Version => {
            println!("Lugli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}