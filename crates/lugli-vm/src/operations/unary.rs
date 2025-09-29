use lugli_common::{Value, LugliError};
use lugli_lexer::TokenKind;

pub fn apply_unary_op(op: &TokenKind, operand: &Value) -> Result<Value, LugliError> {
    match (op, operand) {
        (TokenKind::Minus, Value::Number(n)) => Ok(Value::Number(-n)),
        (TokenKind::Bang, val) => Ok(Value::Bool(!val.is_truthy())),
        _ => Err(LugliError::runtime(format!(
            "Cannot apply {:?} to {}",
            op,
            operand.type_name()
        ))),
    }
}