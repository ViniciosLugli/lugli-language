use lugli_common::{LugliError, Value};
use std::{cell::RefCell, rc::Rc};

pub fn dict_keys(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("keys expects 1 argument"));
    }
    match &args[0] {
        Value::Dict(d) => {
            let keys: Vec<Value> = d.borrow().keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        }
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}

pub fn dict_contains(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 2 {
        return Err(LugliError::runtime("dict.contains expects 2 arguments"));
    }
    match (&args[0], &args[1]) {
        (Value::Dict(d), Value::String(key)) => Ok(Value::Bool(d.borrow().contains_key(key))),
        (Value::Dict(_), _) => Err(LugliError::runtime("dict.contains key must be a string")),
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}
