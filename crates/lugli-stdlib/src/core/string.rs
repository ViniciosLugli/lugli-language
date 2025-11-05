use lugli_common::{LugliError, StringPool, Value};
use std::{cell::RefCell, rc::Rc};

pub fn string_length(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.len expects 1 argument"));
    }
    match &args[0] {
        Value::String(id) => Ok(Value::Number(pool.resolve(*id).len() as f64)),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_trim(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.trim expects 1 argument"));
    }
    match &args[0] {
        Value::String(id) => {
            let trimmed = pool.resolve(*id).trim().to_string();
            Ok(Value::String(pool.intern(&trimmed)))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_lower(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.lower expects 1 argument"));
    }
    match &args[0] {
        Value::String(id) => {
            let lower = pool.resolve(*id).to_lowercase();
            Ok(Value::String(pool.intern(&lower)))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_upper(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.upper expects 1 argument"));
    }
    match &args[0] {
        Value::String(id) => {
            let upper = pool.resolve(*id).to_uppercase();
            Ok(Value::String(pool.intern(&upper)))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_chars(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.chars expects 1 argument"));
    }
    match &args[0] {
        Value::String(id) => {
            let s = pool.resolve(*id).to_string();
            let chars: Vec<Value> = s.chars().map(|c| Value::String(pool.intern(&c.to_string()))).collect();
            Ok(Value::List(Rc::new(RefCell::new(chars))))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_split(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("string.split expects 1 or 2 arguments"));
    }

    let separator = if args.len() == 2 {
        match &args[1] {
            Value::String(id) => pool.resolve(*id).to_string(),
            _ => return Err(LugliError::type_error("string", args[1].type_name())),
        }
    } else {
        " ".to_string() // Default to space
    };

    match &args[0] {
        Value::String(id) => {
            let s = pool.resolve(*id).to_string();
            let parts: Vec<Value> = s.split(separator.as_str()).map(|part| Value::String(pool.intern(part))).collect();
            Ok(Value::List(Rc::new(RefCell::new(parts))))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_is_alphabetic(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.is_alphabetic expects 1 argument"));
    }
    match &args[0] {
        Value::String(id) => {
            let s = pool.resolve(*id);
            Ok(Value::Bool(s.chars().all(|c| c.is_alphabetic())))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_starts_with(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("string.starts_with expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::String(id1), Value::String(id2)) => {
            let s = pool.resolve(*id1);
            let prefix = pool.resolve(*id2);
            Ok(Value::Bool(s.starts_with(prefix)))
        }
        _ => Err(LugliError::runtime("string.starts_with expects string arguments")),
    }
}

pub fn string_ends_with(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("string.ends_with expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::String(id1), Value::String(id2)) => {
            let s = pool.resolve(*id1);
            let suffix = pool.resolve(*id2);
            Ok(Value::Bool(s.ends_with(suffix)))
        }
        _ => Err(LugliError::runtime("string.ends_with expects string arguments")),
    }
}

pub fn string_contains(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("string.contains expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::String(id1), Value::String(id2)) => {
            let s = pool.resolve(*id1);
            let substring = pool.resolve(*id2);
            Ok(Value::Bool(s.contains(substring)))
        }
        _ => Err(LugliError::runtime("string.contains expects string arguments")),
    }
}

pub fn string_replace(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 3 {
        return Err(LugliError::runtime("string.replace expects 3 arguments"));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(id1), Value::String(id2), Value::String(id3)) => {
            let s = pool.resolve(*id1);
            let from = pool.resolve(*id2);
            let to = pool.resolve(*id3);
            let replaced = s.replace(from, to);
            Ok(Value::String(pool.intern(&replaced)))
        }
        _ => Err(LugliError::runtime("string.replace expects string arguments")),
    }
}
