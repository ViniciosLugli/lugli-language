use lugli_common::{Value, LugliError};
use std::{rc::Rc, cell::RefCell};

pub fn dict_keys(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("keys expects 1 argument"));
    }
    match &args[0] {
        Value::Dict(d) => {
            let keys: Vec<Value> = d.keys().map(|k| Value::String(k.clone())).collect();
            Ok(Value::List(Rc::new(RefCell::new(keys))))
        },
        _ => Err(LugliError::type_error("dict", args[0].type_name())),
    }
}