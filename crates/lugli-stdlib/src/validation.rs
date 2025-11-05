use lugli_common::{LugliError, StringPool, Value};

/// Validates that args has exactly `expected` number of arguments
pub fn check_arity(args: &[Value], expected: usize, fn_name: &str) -> Result<(), LugliError> {
    if args.len() != expected {
        return Err(LugliError::runtime(format!(
            "{} expects {} argument(s), got {}",
            fn_name,
            expected,
            args.len()
        )));
    }
    Ok(())
}

/// Validates that args has at least `min` number of arguments
pub fn check_min_arity(args: &[Value], min: usize, fn_name: &str) -> Result<(), LugliError> {
    if args.len() < min {
        return Err(LugliError::runtime(format!(
            "{} expects at least {} argument(s), got {}",
            fn_name,
            min,
            args.len()
        )));
    }
    Ok(())
}

/// Validates that args has between `min` and `max` number of arguments
pub fn check_arity_range(
    args: &[Value],
    min: usize,
    max: usize,
    fn_name: &str,
) -> Result<(), LugliError> {
    if args.len() < min || args.len() > max {
        return Err(LugliError::runtime(format!(
            "{} expects {} to {} argument(s), got {}",
            fn_name,
            min,
            max,
            args.len()
        )));
    }
    Ok(())
}

/// Extracts a string from a Value, returning error if not a string
pub fn expect_string<'a>(
    value: &'a Value,
    pool: &'a StringPool,
    fn_name: &str,
    arg_pos: usize,
) -> Result<String, LugliError> {
    match value {
        Value::String(id) => Ok(pool.resolve(*id).to_string()),
        _ => Err(LugliError::runtime(format!(
            "{} argument {} must be string, got {}",
            fn_name,
            arg_pos,
            value.type_name()
        ))),
    }
}

/// Extracts a number from a Value, returning error if not a number
pub fn expect_number(
    value: &Value,
    fn_name: &str,
    arg_pos: usize,
) -> Result<f64, LugliError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(LugliError::runtime(format!(
            "{} argument {} must be number, got {}",
            fn_name,
            arg_pos,
            value.type_name()
        ))),
    }
}

/// Extracts a boolean from a Value, returning error if not a boolean
pub fn expect_bool(
    value: &Value,
    fn_name: &str,
    arg_pos: usize,
) -> Result<bool, LugliError> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(LugliError::runtime(format!(
            "{} argument {} must be boolean, got {}",
            fn_name,
            arg_pos,
            value.type_name()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_arity() {
        let args = vec![Value::Number(1.0), Value::Number(2.0)];
        assert!(check_arity(&args, 2, "test").is_ok());
        assert!(check_arity(&args, 1, "test").is_err());
        assert!(check_arity(&args, 3, "test").is_err());
    }

    #[test]
    fn test_check_min_arity() {
        let args = vec![Value::Number(1.0), Value::Number(2.0)];
        assert!(check_min_arity(&args, 1, "test").is_ok());
        assert!(check_min_arity(&args, 2, "test").is_ok());
        assert!(check_min_arity(&args, 3, "test").is_err());
    }

    #[test]
    fn test_check_arity_range() {
        let args = vec![Value::Number(1.0), Value::Number(2.0)];
        assert!(check_arity_range(&args, 1, 3, "test").is_ok());
        assert!(check_arity_range(&args, 2, 2, "test").is_ok());
        assert!(check_arity_range(&args, 3, 5, "test").is_err());
        assert!(check_arity_range(&args, 0, 1, "test").is_err());
    }

    #[test]
    fn test_expect_number() {
        let num = Value::Number(42.0);
        assert_eq!(expect_number(&num, "test", 1).unwrap(), 42.0);

        let not_num = Value::Bool(true);
        assert!(expect_number(&not_num, "test", 1).is_err());
    }

    #[test]
    fn test_expect_bool() {
        let b = Value::Bool(true);
        assert_eq!(expect_bool(&b, "test", 1).unwrap(), true);

        let not_bool = Value::Number(1.0);
        assert!(expect_bool(&not_bool, "test", 1).is_err());
    }
}
