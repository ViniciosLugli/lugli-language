use lugli_common::Span;
use lugli_lexer::{LexError, Token};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token: {token}")]
    UnexpectedToken { token: Token },
    #[error("Expected {expected}, found {found}")]
    Expected { expected: String, found: Token },
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Lexer error: {0}")]
    Lex(#[from] LexError),
    #[error("{message}")]
    Custom { message: String, span: Span },
}

impl ParseError {
    pub fn new(message: &str, span: Span) -> Self {
        Self::Custom {
            message: message.to_string(),
            span,
        }
    }
}
