use crate::validation::check_arity;
use lugli_common::{LugliError, StringPool, Value};

pub fn number_abs(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "abs")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_ceil(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "ceil")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.ceil())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_floor(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "floor")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_sqrt(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "sqrt")?;
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
    check_arity(args, 1, "sin")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.sin())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_cos(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "cos")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.cos())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_tan(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "tan")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.tan())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}

pub fn number_log(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "log")?;
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
    check_arity(args, 1, "exp")?;
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.exp())),
        _ => Err(LugliError::type_error("number", args[0].type_name())),
    }
}
