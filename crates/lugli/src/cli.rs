use colored::Colorize;
use lugli_vm::{ErrorFormatter, Vm, VmError};
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("REPL error: {0}")]
    ReplError(String),
}

impl From<rustyline::error::ReadlineError> for CliError {
    fn from(err: rustyline::error::ReadlineError) -> Self { CliError::ReplError(err.to_string()) }
}

pub fn run_file(file_path: &str) -> Result<(), CliError> {
    let source = fs::read_to_string(file_path).map_err(|_| CliError::FileNotFound(file_path.to_string()))?;

    let (program, span_map) = lugli_parser::parse(&source).map_err(|e| {
        let lugli_error = e.to_lugli_error();
        let formatter = if should_use_colors() { ErrorFormatter::new() } else { ErrorFormatter::without_colors() };
        CliError::Runtime(formatter.format(&lugli_error))
    })?;

    let mut vm = Vm::builder().with_source(file_path.to_string(), source).build();

    match vm.compile_and_run(&program, span_map) {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError::Runtime(format_vm_error(e))),
    }
}

fn format_vm_error(error: VmError) -> String {
    let use_colors = should_use_colors();
    let formatter = if use_colors { ErrorFormatter::new() } else { ErrorFormatter::without_colors() };

    match error {
        VmError::RuntimeError(ref lugli_error) => formatter.format(lugli_error),
        VmError::CompilationError(ref msg) => {
            if use_colors {
                format!("{} {}: {}", "Error".red().bold(), "[Compilation]".yellow(), msg)
            } else {
                format!("Error [Compilation]: {}", msg)
            }
        }
        VmError::VmError(ref msg) => {
            if use_colors {
                format!("{} {}: {}", "Error".red().bold(), "[VM]".yellow(), msg)
            } else {
                format!("Error [VM]: {}", msg)
            }
        }
    }
}

fn should_use_colors() -> bool {
    // Check NO_COLOR environment variable (standard convention)
    std::env::var("NO_COLOR").is_err()
}

// Reserved for future use when parse errors get rich formatting
#[allow(dead_code)]
fn calculate_line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (i, ch) in source.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

#[allow(dead_code)]
fn get_source_line(source: &str, line: usize) -> String { source.lines().nth(line.saturating_sub(1)).unwrap_or("").to_string() }

pub fn check_file(file_path: &str) -> Result<(), CliError> {
    let source = fs::read_to_string(file_path).map_err(|_| CliError::FileNotFound(file_path.to_string()))?;

    match lugli_parser::parse(&source) {
        Ok(_) => {
            println!("✓ Syntax is valid");
            Ok(())
        }
        Err(e) => Err(CliError::Runtime(e.to_string())),
    }
}

pub fn disassemble_file(file_path: &str) -> Result<(), CliError> {
    let source = fs::read_to_string(file_path).map_err(|_| CliError::FileNotFound(file_path.to_string()))?;

    let (program, span_map) = lugli_parser::parse(&source).map_err(|e| CliError::Runtime(e.to_string()))?;

    let vm = Vm::new();
    match vm.compile(&program, span_map) {
        Ok(bytecode) => {
            println!("{}", lugli_vm::debug::disassemble(&bytecode, file_path));
            Ok(())
        }
        Err(e) => Err(CliError::Runtime(e.to_string())),
    }
}
