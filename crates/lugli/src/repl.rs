use crate::cli::CliError;
use colored::Colorize;
use std::io::{self, Write};

pub fn start() -> Result<(), CliError> {
    println!("{}", "Lugli REPL".bright_cyan().bold());
    println!("Type 'exit' to quit");

    loop {
        print!("{} ", "lugli>".bright_green().bold());
        if let Err(e) = io::stdout().flush() {
            eprintln!("{}: Failed to flush stdout: {}", "IO Error".red().bold(), e);
            break;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                if input == "exit" || input == "quit" {
                    println!("Goodbye!");
                    break;
                }

                match lugli_parser::parse(input) {
                    Ok(program) => match lugli_vm::compile_and_run(&program) {
                        Ok(value) => {
                            if !matches!(value, lugli_common::Value::Null) {
                                println!("{}", value);
                            }
                        }
                        Err(e) => {
                            eprintln!("{}: {}", "Runtime Error".red().bold(), e);
                        }
                    },
                    Err(e) => {
                        eprintln!("{}: {}", "Parse Error".red().bold(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "Input Error".red().bold(), e);
                break;
            }
        }
    }

    Ok(())
}
