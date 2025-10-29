use crate::NativeFunction;
use lugli_common::{LugliError, Value};
use std::io::{self, Write};

pub mod console;

pub fn get_functions() -> Vec<(&'static str, NativeFunction)> { vec![("print", print_fn), ("input", input_fn)] }

fn print_fn(args: &[Value]) -> Result<Value, LugliError> {
    // Print all arguments with spaces between them
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }

    // Always add newline like Python's print()
    println!();

    Ok(Value::Null)
}

fn input_fn(args: &[Value]) -> Result<Value, LugliError> {
    // Print prompt if provided
    if !args.is_empty() {
        for arg in args {
            print!("{}", arg);
        }
        io::stdout().flush().map_err(|e| LugliError::runtime(format!("IO error: {}", e)))?;
    }

    // Read line from stdin
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map_err(|e| LugliError::runtime(format!("Failed to read input: {}", e)))?;

    // Remove trailing newline
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }

    Ok(Value::String(buffer))
}
