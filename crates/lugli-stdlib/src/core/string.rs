use lugli_common::{LugliError, Value};
use std::{cell::RefCell, rc::Rc};

pub fn string_length(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.len expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_trim(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.trim expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.trim().to_string())),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_lower(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.lower expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_lowercase())),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_upper(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.upper expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::String(s.to_uppercase())),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_chars(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.chars expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(Rc::new(RefCell::new(chars))))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_split(args: &[Value]) -> Result<Value, LugliError> {
    if args.is_empty() || args.len() > 2 {
        return Err(LugliError::runtime("string.split expects 1 or 2 arguments"));
    }

    let separator = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.as_str(),
            _ => return Err(LugliError::type_error("string", args[1].type_name())),
        }
    } else {
        " " // Default to space
    };

    match &args[0] {
        Value::String(s) => {
            let parts: Vec<Value> = s.split(separator).map(|part| Value::String(part.to_string())).collect();
            Ok(Value::List(Rc::new(RefCell::new(parts))))
        }
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_is_alphabetic(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("string.is_alphabetic expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Bool(s.chars().all(|c| c.is_alphabetic()))),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}

pub fn string_starts_with(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("string.starts_with expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix.as_str()))),
        _ => Err(LugliError::runtime("string.starts_with expects string arguments")),
    }
}

pub fn string_ends_with(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("string.ends_with expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix.as_str()))),
        _ => Err(LugliError::runtime("string.ends_with expects string arguments")),
    }
}

pub fn string_contains(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("string.contains expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::String(s), Value::String(substring)) => Ok(Value::Bool(s.contains(substring.as_str()))),
        _ => Err(LugliError::runtime("string.contains expects string arguments")),
    }
}

pub fn string_replace(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 3 {
        return Err(LugliError::runtime("string.replace expects 3 arguments"));
    }
    match (&args[0], &args[1], &args[2]) {
        (Value::String(s), Value::String(from), Value::String(to)) => Ok(Value::String(s.replace(from.as_str(), to.as_str()))),
        _ => Err(LugliError::runtime("string.replace expects string arguments")),
    }
}
