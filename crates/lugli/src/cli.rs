use thiserror::Error;
use std::fs;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Runtime error: {0}")]
    Runtime(String),
}

pub fn run_file(file_path: &str) -> Result<(), CliError> {
    let source = fs::read_to_string(file_path)
        .map_err(|_| CliError::FileNotFound(file_path.to_string()))?;

    let program = lugli_parser::parse(&source)
        .map_err(|e| CliError::Runtime(e.to_string()))?;

    match lugli_vm::compile_and_run(&program) {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError::Runtime(e.to_string())),
    }
}

pub fn check_file(file_path: &str) -> Result<(), CliError> {
    let source = fs::read_to_string(file_path)
        .map_err(|_| CliError::FileNotFound(file_path.to_string()))?;

    match lugli_parser::parse(&source) {
        Ok(_) => {
            println!("✓ Syntax is valid");
            Ok(())
        },
        Err(e) => Err(CliError::Runtime(e.to_string())),
    }
}