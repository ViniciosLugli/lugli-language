use colored::Colorize;
use lugli_vm::Vm;
use rustyline::{
    Editor,
    history::{FileHistory, History},
};

pub fn handle_command(
    input: &str,
    vm: &mut Vm,
    editor: &mut Editor<super::ReplHelper, FileHistory>,
    bytecodes: &[lugli_vm::Bytecode],
) -> Result<(), String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        ":help" | ":h" => {
            show_help();
            Ok(())
        }
        ":vars" | ":v" => {
            show_variables(vm, bytecodes);
            Ok(())
        }
        ":clear" | ":cls" => {
            clear_screen();
            Ok(())
        }
        ":reset" | ":r" => {
            reset_vm(vm);
            Ok(())
        }
        ":history" | ":hist" => {
            show_history(editor);
            Ok(())
        }
        ":type" | ":t" => {
            if parts.len() < 2 {
                return Err("Usage: :type <variable>".to_string());
            }
            show_type(vm, parts[1])?;
            Ok(())
        }
        ":load" | ":l" => {
            if parts.len() < 2 {
                return Err("Usage: :load <file>".to_string());
            }
            load_file(vm, parts[1])?;
            Ok(())
        }
        ":quit" | ":q" | ":exit" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        _ => Err(format!("Unknown command: {}. Type :help for available commands.", parts[0])),
    }
}

fn show_help() {
    println!("\n{}", "REPL Commands:".bright_cyan().bold());
    println!("  {}  - Show this help message", ":help, :h".bright_green());
    println!("  {}  - Show current variables", ":vars, :v".bright_green());
    println!("  {} - Show variable type", ":type, :t <var>".bright_green());
    println!("  {} - Load and execute a file", ":load, :l <file>".bright_green());
    println!("  {} - Clear the screen", ":clear, :cls".bright_green());
    println!("  {} - Reset VM state (clear all variables)", ":reset, :r".bright_green());
    println!("  {} - Show command history", ":history, :hist".bright_green());
    println!("  {} - Exit the REPL", ":quit, :q, :exit".bright_green());
    println!("\n{}", "Special Variables:".bright_cyan().bold());
    println!("  {}       - Last result", "_".bright_green());
    println!("\n{}", "Keyboard Shortcuts:".bright_cyan().bold());
    println!("  {} - Navigate command history", "Up/Down Arrow".bright_green());
    println!("  {}    - Tab completion", "Tab".bright_green());
    println!("  {}  - Cancel current line", "Ctrl+C".bright_green());
    println!("  {}  - Exit REPL", "Ctrl+D".bright_green());
    println!("  {}  - Search history", "Ctrl+R".bright_green());
    println!();
}

fn show_variables(vm: &Vm, _bytecodes: &[lugli_vm::Bytecode]) {
    if vm.machine().globals().borrow().is_empty() {
        println!("{}", "No variables defined yet.".yellow());
        return;
    }

    println!("\n{}", "Current Variables:".bright_cyan().bold());
    let mut vars: Vec<(String, lugli_common::Value)> = vm.machine().globals().borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    vars.sort_by_key(|(k, _)| k.clone());

    for (name, value) in vars {
        // Skip native functions from display
        if matches!(value, lugli_common::Value::NativeFunction { .. }) {
            continue;
        }

        println!("  {} = {}", name.bright_green(), vm.machine().format_value(&value));
    }
    println!();
}

fn clear_screen() {
    // ANSI escape code to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
}

fn reset_vm(vm: &mut Vm) {
    let native_functions: Vec<(String, lugli_common::Value)> = vm
        .machine()
        .globals()
        .borrow()
        .iter()
        .filter(|(_, v)| matches!(v, lugli_common::Value::NativeFunction { .. }))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    vm.reset();

    // Restore native functions
    for (name, func) in native_functions {
        vm.machine_mut().globals_mut().borrow_mut().insert(name, func);
    }

    println!("{}", "VM state reset. All user variables cleared.".bright_green());
}

fn show_history(editor: &Editor<super::ReplHelper, FileHistory>) {
    println!("\n{}", "Command History:".bright_cyan().bold());

    let history = editor.history();
    let len = history.len();

    if len == 0 {
        println!("{}", "No history yet.".yellow());
        return;
    }

    // Show last 20 entries
    let start = len.saturating_sub(20);

    for (i, entry) in history.iter().enumerate().skip(start) {
        println!("  {} {}", format!("{}:", i + 1).bright_black(), entry);
    }
    println!();
}

fn show_type(vm: &Vm, var_name: &str) -> Result<(), String> {
    match vm.machine().globals().borrow().get(var_name) {
        Some(value) => {
            println!("{}: {}", var_name.bright_green(), value.type_name().bright_cyan());
            Ok(())
        }
        None => Err(format!("Variable '{}' not found", var_name)),
    }
}

fn load_file(vm: &mut Vm, file_path: &str) -> Result<(), String> {
    use std::fs;

    let source = fs::read_to_string(file_path).map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;

    match lugli_parser::parse(&source) {
        Ok((program, span_map)) => match vm.compile(&program, span_map) {
            Ok(bytecode) => match vm.run(&bytecode) {
                Ok(_) => {
                    println!("{}", format!("Loaded '{}'", file_path).bright_green());
                    Ok(())
                }
                Err(e) => Err(format!("Runtime error: {}", e)),
            },
            Err(e) => Err(format!("Compilation error: {}", e)),
        },
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
