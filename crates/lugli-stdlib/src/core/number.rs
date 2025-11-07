use crate::validation::{check_arity, expect_number};
use lugli_common::{LugliError, StringPool, Value};

pub fn number_abs(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "abs")?;
    let n = expect_number(&args[0], "abs", 1)?;
    Ok(Value::Number(n.abs()))
}

pub fn number_ceil(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "ceil")?;
    let n = expect_number(&args[0], "ceil", 1)?;
    Ok(Value::Number(n.ceil()))
}

pub fn number_floor(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "floor")?;
    let n = expect_number(&args[0], "floor", 1)?;
    Ok(Value::Number(n.floor()))
}

pub fn number_sqrt(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "sqrt")?;
    let n = expect_number(&args[0], "sqrt", 1)?;
    if n < 0.0 { Err(LugliError::runtime("sqrt expects a non-negative number")) } else { Ok(Value::Number(n.sqrt())) }
}

pub fn number_sin(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "sin")?;
    let n = expect_number(&args[0], "sin", 1)?;
    Ok(Value::Number(n.sin()))
}

pub fn number_cos(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "cos")?;
    let n = expect_number(&args[0], "cos", 1)?;
    Ok(Value::Number(n.cos()))
}

pub fn number_tan(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "tan")?;
    let n = expect_number(&args[0], "tan", 1)?;
    Ok(Value::Number(n.tan()))
}

pub fn number_log(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "log")?;
    let n = expect_number(&args[0], "log", 1)?;
    if n <= 0.0 { Err(LugliError::runtime("log expects a positive number")) } else { Ok(Value::Number(n.ln())) }
}

pub fn number_exp(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    check_arity(args, 1, "exp")?;
    let n = expect_number(&args[0], "exp", 1)?;
    Ok(Value::Number(n.exp()))
}
