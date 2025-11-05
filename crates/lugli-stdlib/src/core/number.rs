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

pub fn number_ceil(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("ceil expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.ceil())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_floor(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("floor expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_sqrt(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("sqrt expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                Err(LugliError::runtime("sqrt expects a non-negative number"))
            } else {
                Ok(Value::Number(n.sqrt()))
            }
        }
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_sin(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("sin expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.sin())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_cos(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("cos expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.cos())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_tan(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("tan expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.tan())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_log(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("log expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => {
            if *n <= 0.0 {
                Err(LugliError::runtime("log expects a positive number"))
            } else {
                Ok(Value::Number(n.ln()))
            }
        }
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_exp(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    if args.len() != 1 {
        return Err(LugliError::runtime("exp expects 1 argument"));
    }
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.exp())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}
