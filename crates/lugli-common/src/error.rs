use thiserror::Error;
use crate::Span;

#[derive(Error, Debug, Clone)]
pub enum LugliError {
    #[error("Lexical error: {message} at {span}")]
    Lex { message: String, span: Span },

    #[error("Parse error: {message} at {span}")]
    Parse { message: String, span: Span },

    #[error("Runtime error: {message}")]
    Runtime { message: String },

    #[error("Type error: expected {expected}, found {found}")]
    Type { expected: String, found: String },

    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds { index: usize, length: usize },

    #[error("IO error: {message}")]
    Io { message: String },
}

impl LugliError {
    pub fn lex(message: impl Into<String>, span: Span) -> Self {
        Self::Lex { message: message.into(), span }
    }

    pub fn parse(message: impl Into<String>, span: Span) -> Self {
        Self::Parse { message: message.into(), span }
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime { message: message.into() }
    }

    pub fn type_error(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::Type { expected: expected.into(), found: found.into() }
    }

    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::UndefinedVariable { name: name.into() }
    }
}