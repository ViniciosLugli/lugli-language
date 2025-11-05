use crate::validation::*;
use lugli_common::{LugliError, StringPool, Value};
use std::{cell::RefCell, rc::Rc};

pub fn dict_keys(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "keys")?;
    match &args[0] {
        Value::Dict(d) => {
            let keys: Vec<Value> = d.borrow().keys().map(|k| Value::String(*k)).collect();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_contains(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 2, "dict.contains")?;
    match &args[0] {
        Value::Dict(d) => {
            // Convert key to string if needed
            let key_id = match &args[1] {
                Value::String(id) => *id,
                Value::Number(n) => pool.intern(&n.to_string()),
                Value::Bool(b) => pool.intern(if *b { "true" } else { "false" }),
                Value::Null => pool.intern("null"),
                other => return Err(LugliError::runtime(format!("Cannot use {} as dictionary key", other.type_name()))),
            };
            Ok(Value::Bool(d.borrow().contains_key(&key_id)))
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_values(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "values")?;
    match &args[0] {
        Value::Dict(d) => {
            let values: Vec<Value> = d.borrow().values().cloned().collect();
            Ok(Value::List(Rc::new(RefCell::new(values))))
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_get(args: &[Value], pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity_range(args, 2, 3, "dict.get")?;
    match &args[0] {
        Value::Dict(d) => {
            // Convert key to string if needed
            let key_id = match &args[1] {
                Value::String(id) => *id,
                Value::Number(n) => pool.intern(&n.to_string()),
                Value::Bool(b) => pool.intern(if *b { "true" } else { "false" }),
                Value::Null => pool.intern("null"),
                other => return Err(LugliError::runtime(format!("Cannot use {} as dictionary key", other.type_name()))),
            };

            if let Some(value) = d.borrow().get(&key_id) {
                Ok(value.clone())
            } else if args.len() == 3 {
                // Return default value
                Ok(args[2].clone())
            } else {
                Err(LugliError::runtime("Key not found in dict".to_string()))
            }
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_len(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "dict.len")?;
    match &args[0] {
        Value::Dict(d) => Ok(Value::Number(d.borrow().len() as f64)),
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}
