use lugli_common::{Value, LugliError};

pub fn list_length(args: &[Value]) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("length expects 1 argument"));
    }
    match &args[0] {
        Value::List(l) => Ok(Value::Number(l.borrow().len() as f64)),
        _ => Err(LugliError::type_error("list", args[0].type_name())),
    }
}