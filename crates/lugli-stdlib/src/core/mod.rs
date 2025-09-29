use lugli_common::{Value, LugliError};
use crate::NativeFunction;

pub mod string;
pub mod number;
pub mod list;
pub mod dict;

pub fn get_functions() -> Vec<(&'static str, NativeFunction)> {
    vec![
        ("type", type_of),
        ("len", len_fn),
    ]
}

fn type_of(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("type expects 1 argument"));
    }
    Ok(Value::String(args[0].type_name().to_string()))
}

fn len_fn(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("len expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        Value::Dict(d) => Ok(Value::Number(d.len() as f64)),
        _ => Err(LugliError::runtime(format!("Object of type {} has no len()", args[0].type_name()))),
    }
}