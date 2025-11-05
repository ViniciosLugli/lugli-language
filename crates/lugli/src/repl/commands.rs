use colored::Colorize;
use lugli_vm::Machine;
use rustyline::{
    Editor,
    history::{FileHistory, History},
};

pub fn handle_command(
    input: &str,
    vm: &mut Machine,
    editor: &mut Editor<super::ReplHelper, FileHistory>,
    bytecodes: &[lugli_vm::Bytecode],
) -> Result<(), String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        ":help" | ":h" => show_help(),
        ":vars" | ":v" => show_variables(vm, bytecodes),
        ":clear" | ":cls" => clear_screen(),
        ":reset" | ":r" => reset_vm(vm),
        ":history" | ":hist" => show_history(editor),
        ":quit" | ":q" | ":exit" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        _ => return Err(format!("Unknown command: {}. Type :help for available commands.", parts[0])),
    }

    Ok(())
}

fn show_help() {
    println!("\n{}", "REPL Commands:".bright_cyan().bold());
    println!("  {}  - Show this help message", ":help, :h".bright_green());
    println!("  {}  - Show current variables", ":vars, :v".bright_green());
    println!("  {} - Clear the screen", ":clear, :cls".bright_green());
    println!("  {} - Reset VM state (clear all variables)", ":reset, :r".bright_green());
    println!("  {} - Show command history", ":history, :hist".bright_green());
    println!("  {} - Exit the REPL", ":quit, :q, :exit".bright_green());
    println!("\n{}", "Keyboard Shortcuts:".bright_cyan().bold());
    println!("  {} - Navigate command history", "Up/Down Arrow".bright_green());
    println!("  {}    - Tab completion", "Tab".bright_green());
    println!("  {}  - Cancel current line", "Ctrl+C".bright_green());
    println!("  {}  - Exit REPL", "Ctrl+D".bright_green());
    println!("  {}  - Search history", "Ctrl+R".bright_green());
    println!();
}

fn show_variables(vm: &Machine, _bytecodes: &[lugli_vm::Bytecode]) {
    if vm.globals.is_empty() {
        println!("{}", "No variables defined yet.".yellow());
        return;
    }

    println!("\n{}", "Current Variables:".bright_cyan().bold());
    let mut vars: Vec<_> = vm.globals.iter().collect();
    vars.sort_by_key(|(k, _)| *k);

    for (name, value) in vars {
        // Skip native functions from display
        if matches!(value, lugli_common::Value::NativeFunction { .. }) {
            continue;
        }

        // Note: Strings show as <string#N> because they reference string pools
        // from previous REPL commands which are no longer available.
        // The actual string values are correct and accessible in the VM.
        println!("  {} = {}", name.bright_green(), value);
    }
    println!();
}

fn clear_screen() {
    // ANSI escape code to clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
}

fn reset_vm(vm: &mut Machine) {
    let native_functions: Vec<_> =
        vm.globals.iter().filter(|(_, v)| matches!(v, lugli_common::Value::NativeFunction { .. })).map(|(k, v)| (k.clone(), v.clone())).collect();

    vm.reset();

    // Restore native functions
    for (name, func) in native_functions {
        vm.globals.insert(name, func);
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
