use lugli_common::{LugliError, StringPool, Value};

pub fn number_abs(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("abs expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}
