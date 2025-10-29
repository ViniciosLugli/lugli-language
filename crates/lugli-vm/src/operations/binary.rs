use lugli_common::{LugliError, Value};
use lugli_lexer::TokenKind;
use std::cell::RefCell;
use std::rc::Rc;

pub fn apply_binary_op(left: &Value, op: &TokenKind, right: &Value) -> Result<Value, LugliError> {
    match (left, op, right) {
        (Value::Number(l), TokenKind::Plus, Value::Number(r)) => Ok(Value::Number(l + r)),
        (Value::Number(l), TokenKind::Minus, Value::Number(r)) => Ok(Value::Number(l - r)),
        (Value::Number(l), TokenKind::Star, Value::Number(r)) => Ok(Value::Number(l * r)),
        (Value::Number(l), TokenKind::Slash, Value::Number(r)) => {
            if *r == 0.0 {
                Err(LugliError::division_by_zero())
            } else {
                Ok(Value::Number(l / r))
            }
        }
        (Value::Number(l), TokenKind::Percent, Value::Number(r)) => {
            if *r == 0.0 {
                Err(LugliError::division_by_zero())
            } else {
                Ok(Value::Number(l % r))
            }
        }
        (Value::String(l), TokenKind::Plus, Value::String(r)) => Ok(Value::String(format!("{}{}", l, r))),
        // List concatenation
        (Value::List(l), TokenKind::Plus, Value::List(r)) => {
            let mut result = l.borrow().clone();
            result.extend(r.borrow().clone());
            Ok(Value::List(Rc::new(RefCell::new(result))))
        }
        // Comparison operators
        (Value::Number(l), TokenKind::Greater, Value::Number(r)) => Ok(Value::Bool(l > r)),
        (Value::Number(l), TokenKind::GreaterEqual, Value::Number(r)) => Ok(Value::Bool(l >= r)),
        (Value::Number(l), TokenKind::Less, Value::Number(r)) => Ok(Value::Bool(l < r)),
        (Value::Number(l), TokenKind::LessEqual, Value::Number(r)) => Ok(Value::Bool(l <= r)),
        // Equality operators
        (l, TokenKind::EqualEqual, r) => Ok(Value::Bool(l.equals(r))),
        (l, TokenKind::BangEqual, r) => Ok(Value::Bool(!l.equals(r))),
        // Logical operators
        (l, TokenKind::And, r) => Ok(Value::Bool(l.is_truthy() && r.is_truthy())),
        (l, TokenKind::Or, r) => Ok(Value::Bool(l.is_truthy() || r.is_truthy())),
        _ => Err(LugliError::runtime(format!("Cannot apply {:?} to {} and {}", op, left.type_name(), right.type_name()))),
    }
}
