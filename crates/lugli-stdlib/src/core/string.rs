use lugli_common::{Value, LugliError};

pub fn string_length(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("length expects 1 argument"));
    }
    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => Err(LugliError::type_error("string", args[0].type_name())),
    }
}