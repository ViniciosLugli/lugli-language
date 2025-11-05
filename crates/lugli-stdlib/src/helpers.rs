use lugli_common::{LugliError, StringId, StringPool, Value};
use hashbrown::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! validate_arity {
    ($args:expr, $expected:expr, $func:expr) => {
        if $args.len() != $expected {
            return Err(LugliError::runtime(format!(
                "{} expects {} argument(s), got {}",
                $func,
                $expected,
                $args.len()
            )));
        }
    };
    ($args:expr, $min:expr, $max:expr, $func:expr) => {
        if $args.len() < $min || $args.len() > $max {
            return Err(LugliError::runtime(format!(
                "{} expects {}-{} argument(s), got {}",
                $func,
                $min,
                $max,
                $args.len()
            )));
        }
    };
}

pub(crate) use validate_arity;

pub fn expect_string(v: &Value) -> Result<StringId, LugliError> {
    match v {
        Value::String(id) => Ok(*id),
        _ => Err(LugliError::type_error("string", v.type_name())),
    }
}

pub fn expect_list(v: &Value) -> Result<Rc<RefCell<Vec<Value>>>, LugliError> {
    match v {
        Value::List(l) => Ok(Rc::clone(l)),
        _ => Err(LugliError::type_error("list", v.type_name())),
    }
}

pub fn expect_dict(v: &Value) -> Result<Rc<RefCell<HashMap<StringId, Value>>>, LugliError> {
    match v {
        Value::Dict(d) => Ok(Rc::clone(d)),
        _ => Err(LugliError::type_error("dict", v.type_name())),
    }
}

pub fn expect_number(v: &Value) -> Result<f64, LugliError> {
    match v {
        Value::Number(n) => Ok(*n),
        _ => Err(LugliError::type_error("number", v.type_name())),
    }
}

pub fn expect_bool(v: &Value) -> Result<bool, LugliError> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(LugliError::type_error("bool", v.type_name())),
    }
}
