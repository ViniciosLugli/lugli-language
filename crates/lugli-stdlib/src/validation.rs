use lugli_common::{LugliError, StringId, StringPool, Value};
use hashbrown::HashMap;
use std::{cell::RefCell, rc::Rc};

/// Validates that args has exactly `expected` number of arguments
pub fn check_arity(args: &[Value], expected: usize, fn_name: &str) -> Result<(), LugliError> {
    if args.len() != expected {
        return Err(LugliError::runtime(format!("{} expects {} argument(s), got {}", fn_name, expected, args.len())));
    }
    Ok(())
}

/// Validates that args has at least `min` number of arguments
pub fn check_min_arity(args: &[Value], min: usize, fn_name: &str) -> Result<(), LugliError> {
    if args.len() < min {
        return Err(LugliError::runtime(format!("{} expects at least {} argument(s), got {}", fn_name, min, args.len())));
    }
    Ok(())
}

/// Validates that args has between `min` and `max` number of arguments
pub fn check_arity_range(args: &[Value], min: usize, max: usize, fn_name: &str) -> Result<(), LugliError> {
    if args.len() < min || args.len() > max {
        return Err(LugliError::runtime(format!("{} expects {} to {} argument(s), got {}", fn_name, min, max, args.len())));
    }
    Ok(())
}

/// Extracts a string from a Value, returning error if not a string
pub fn expect_string<'a>(value: &'a Value, pool: &'a StringPool, fn_name: &str, arg_pos: usize) -> Result<String, LugliError> {
    match value {
        Value::String(id) => Ok(pool.resolve(*id).to_string()),
        _ => Err(LugliError::type_error("string", value.type_name())),
    }
}

/// Extracts a number from a Value, returning error if not a number
pub fn expect_number(value: &Value, fn_name: &str, arg_pos: usize) -> Result<f64, LugliError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(LugliError::type_error("number", value.type_name())),
    }
}

/// Extracts a boolean from a Value, returning error if not a boolean
pub fn expect_bool(value: &Value, fn_name: &str, arg_pos: usize) -> Result<bool, LugliError> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(LugliError::type_error("boolean", value.type_name())),
    }
}

/// Extracts a list from a Value, returning error if not a list
pub fn expect_list<'a>(value: &'a Value, fn_name: &str, arg_pos: usize) -> Result<&'a Rc<RefCell<Vec<Value>>>, LugliError> {
    match value {
        Value::List(l) => Ok(l),
        _ => Err(LugliError::type_error("list", value.type_name())),
    }
}

/// Extracts a dict from a Value, returning error if not a dict
pub fn expect_dict<'a>(value: &'a Value, fn_name: &str, arg_pos: usize) -> Result<&'a Rc<RefCell<HashMap<StringId, Value>>>, LugliError> {
    match value {
        Value::Dict(d) => Ok(d),
        _ => Err(LugliError::type_error("dict", value.type_name())),
    }
}

/// Converts a Value to a dictionary key (StringId)
pub fn value_to_dict_key(value: &Value, pool: &mut StringPool) -> Result<StringId, LugliError> {
    match value {
        Value::String(id) => Ok(*id),
        Value::Number(n) => Ok(pool.intern(&n.to_string())),
        Value::Bool(b) => Ok(pool.intern(if *b { "true" } else { "false" })),
        Value::Null => Ok(pool.intern("null")),
        other => Err(LugliError::runtime(format!("Cannot use {} as dictionary key", other.type_name()))),
    }
}

/// Validates and normalizes a list index, handling negative indices
/// Returns Ok(Some(idx)) for valid index, Ok(None) for out-of-bounds (get operations)
pub fn validate_list_index_for_get(index: &Value, list_len: usize) -> Result<Option<usize>, LugliError> {
    match index {
        Value::Number(idx) => {
            if !idx.is_finite() {
                return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
            }
            if idx.fract() != 0.0 {
                return Err(LugliError::runtime(format!("Index must be an integer, got {}", idx)));
            }

            let idx_i64 = *idx as i64;
            let len = list_len as i64;

            // Handle negative indices
            let actual_idx = if idx_i64 < 0 {
                let positive_offset = len + idx_i64;
                if positive_offset < 0 {
                    return Ok(None); // Out of bounds
                }
                positive_offset as usize
            } else {
                if idx_i64 >= len {
                    return Ok(None); // Out of bounds
                }
                idx_i64 as usize
            };

            Ok(Some(actual_idx))
        }
        _ => Err(LugliError::type_error("number", index.type_name())),
    }
}

/// Validates and normalizes a list index for set operations (errors on out-of-bounds)
pub fn validate_list_index_for_set(index: &Value, list_len: usize) -> Result<usize, LugliError> {
    match index {
        Value::Number(idx) => {
            if !idx.is_finite() {
                return Err(LugliError::runtime(format!("Index must be a finite number, got {}", idx)));
            }
            if idx.fract() != 0.0 {
                return Err(LugliError::runtime(format!("Index must be an integer, got {}", idx)));
            }

            let idx_i64 = *idx as i64;
            let len = list_len as i64;

            // Handle negative indices
            let actual_idx = if idx_i64 < 0 {
                let positive_offset = len + idx_i64;
                if positive_offset < 0 {
                    return Err(LugliError::runtime(format!(
                        "Negative index {} out of range for list assignment (length {}, minimum is -{})",
                        idx_i64, len, len
                    )));
                }
                positive_offset as usize
            } else {
                if idx_i64 >= len {
                    return Err(LugliError::runtime(format!(
                        "Index {} out of range for list assignment (length {})",
                        idx_i64, len
                    )));
                }
                idx_i64 as usize
            };

            Ok(actual_idx)
        }
        _ => Err(LugliError::type_error("number", index.type_name())),
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
