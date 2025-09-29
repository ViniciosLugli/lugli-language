use lugli_common::{Value, LugliError};
use std::io::{self, Write};

pub fn console_input() -> Result<Value, LugliError> {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).map_err(|_| {
        LugliError::runtime("Failed to read input")
    })?;
    Ok(Value::String(input.trim().to_string()))
}

pub fn console_clear() -> Result<Value, LugliError> {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
    Ok(Value::Null)
}