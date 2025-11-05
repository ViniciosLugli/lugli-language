use lugli_common::{LugliError, StringPool, Value};
use std::{cell::RefCell, rc::Rc};

pub fn dict_keys(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("keys expects 1 argument"));
    }
    match &args[0] {
        Value::Dict(d) => {
            let keys: Vec<Value> = d.borrow().keys().map(|k| Value::String(*k)).collect();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_contains(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("dict.contains expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::Dict(d), Value::String(key)) => Ok(Value::Bool(d.borrow().contains_key(key))),
        (Value::Dict(_), _) => Err(LugliError::runtime("dict.contains key must be a string")),
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_values(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("values expects 1 argument"));
    }
    match &args[0] {
        Value::Dict(d) => {
            let values: Vec<Value> = d.borrow().values().cloned().collect();
            Ok(Value::List(Rc::new(RefCell::new(values))))
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_get(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LugliError::runtime("dict.get expects 2 or 3 arguments (dict, key, [default])"));
    }
    match (&args[0], &args[1]) {
        (Value::Dict(d), Value::String(key)) => {
            if let Some(value) = d.borrow().get(key) {
                Ok(value.clone())
            } else if args.len() == 3 {
                // Return default value
                Ok(args[2].clone())
            } else {
                Err(LugliError::runtime("Key not found in dict".to_string()))
            }
        }
        (Value::Dict(_), _) => Err(LugliError::runtime("dict.get key must be a string")),
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}
