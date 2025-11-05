mod commands;
mod completer;
mod highlighter;
mod validator;

use crate::cli::CliError;
use colored::Colorize;
use lugli_vm::Machine;
use rustyline::{Config, Editor, error::ReadlineError};
use std::path::PathBuf;

use commands::handle_command;
use completer::LugliCompleter;
use highlighter::LugliHighlighter;
use validator::InputValidator;

const HISTORY_FILE: &str = ".lugli_history";
const MAX_HISTORY_SIZE: usize = 1000;

pub fn start() -> Result<(), CliError> {
    println!("{}", "Lugli REPL v0.4.0".bright_cyan().bold());
    println!("Type 'exit', 'quit', or press Ctrl+D to quit");
    println!("Type ':help' for available commands\n");

    let config = Config::builder().max_history_size(MAX_HISTORY_SIZE)?.auto_add_history(true).build();

    let helper = ReplHelper::new();
    let mut editor = Editor::with_config(config)?;
    editor.set_helper(Some(helper));

    // Load history from file
    let history_path = get_history_path();
    if let Err(e) = editor.load_history(&history_path)
        && !matches!(e, ReadlineError::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::NotFound)
    {
        eprintln!("{}: Failed to load history: {}", "Warning".yellow().bold(), e);
    }

    let mut vm = Machine::new();
    let mut bytecodes: Vec<lugli_vm::Bytecode> = Vec::new();

    loop {
        let readline = editor.readline(&format!("{} ", "lugli>".bright_green().bold()));

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Handle special commands
                if line.starts_with(':') {
                    if let Err(e) = handle_command(line, &mut vm, &mut editor, &bytecodes) {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    }
                    continue;
                }

                // Handle exit commands
                if line == "exit" || line == "quit" {
                    println!("Goodbye!");
                    break;
                }

                // Reset VM state for this REPL iteration
                vm.reset_for_repl();

                // Parse and execute
                match lugli_parser::parse(line) {
                    Ok((program, span_map)) => {
                        match lugli_vm::compile(&program, span_map) {
                            Ok(bytecode) => {
                                match lugli_vm::run_with_vm(&mut vm, &bytecode) {
                                    Ok(value) => {
                                        if !matches!(value, lugli_common::Value::Null) {
                                            println!("{}", format_value(&value, &bytecode));
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("{}: {}", "Runtime Error".red().bold(), e);
                                    }
                                }
                                // Always save bytecode for string pool access
                                bytecodes.push(bytecode);
                            }
                            Err(e) => {
                                eprintln!("{}: {}", "Compilation Error".red().bold(), e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Parse Error".red().bold(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - cancel current line
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - exit
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("{}: {}", "Error".red().bold(), err);
                break;
            }
        }
    }

    // Save history before exiting
    if let Err(e) = editor.save_history(&history_path) {
        eprintln!("{}: Failed to save history: {}", "Warning".yellow().bold(), e);
    }

    Ok(())
}

fn get_history_path() -> PathBuf { if let Some(home) = dirs::home_dir() { home.join(HISTORY_FILE) } else { PathBuf::from(HISTORY_FILE) } }

fn format_value(value: &lugli_common::Value, bytecode: &lugli_vm::Bytecode) -> String {
    use lugli_common::Value;

    match value {
        Value::String(id) => {
            let pool = bytecode.string_pool.borrow();
            let s = pool.resolve(*id);
            format!("\"{}\"", s)
        }
        Value::List(l) => match l.try_borrow() {
            Ok(list_ref) => {
                let items: Vec<String> = list_ref.iter().map(|v| format_value(v, bytecode)).collect();
                format!("[{}]", items.join(", "))
            }
            Err(_) => "[<borrowed list>]".to_string(),
        },
        Value::Dict(d) => match d.try_borrow() {
            Ok(dict_ref) => {
                let pool = bytecode.string_pool.borrow();
                let items: Vec<String> = dict_ref
                    .iter()
                    .map(|(k, v)| {
                        let key = pool.resolve(*k);
                        format!("\"{}\": {}", key, format_value(v, bytecode))
                    })
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
            Err(_) => "{<borrowed dict>}".to_string(),
        },
        _ => value.to_string(),
    }
}

// Helper struct that combines all REPL features
struct ReplHelper {
    completer: LugliCompleter,
    highlighter: LugliHighlighter,
    validator: InputValidator,
}

impl ReplHelper {
    fn new() -> Self {
        Self {
            completer: LugliCompleter::new(),
            highlighter: LugliHighlighter::new(),
            validator: InputValidator::new(),
        }
    }
}

// Implement rustyline's Helper trait by delegating to individual components
impl rustyline::Helper for ReplHelper {}

impl rustyline::completion::Completer for ReplHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, _ctx)
    }
}

impl rustyline::hint::Hinter for ReplHelper {
    type Hint = String;
}

impl rustyline::highlight::Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> { self.highlighter.highlight(line, pos) }

    fn highlight_char(&self, line: &str, pos: usize, forced: rustyline::highlight::CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl rustyline::validate::Validator for ReplHelper {
    fn validate(&self, ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
        self.validator.validate(ctx)
    }
}
