use lugli_common::{LugliError, StringPool, Value};
use std::io::{self, Write};

pub fn console_input(pool: &mut StringPool) -> Result<Value, LugliError> {
    let mut input = String::new();
    io::stdout().flush().map_err(|e| LugliError::runtime(format!("Failed to flush stdout: {}", e)))?;
    io::stdin().read_line(&mut input).map_err(|e| LugliError::runtime(format!("Failed to read input: {}", e)))?;
    Ok(Value::String(pool.intern(input.trim())))
}

pub fn console_clear(_pool: &mut StringPool) -> Result<Value, LugliError> {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().map_err(|e| LugliError::runtime(format!("Failed to flush stdout: {}", e)))?;
    Ok(Value::Null)
}
