//! Parser for Lugli source code.
//!
//! Transforms tokens into an Abstract Syntax Tree (AST) using recursive descent parsing
//! with Pratt parsing for expressions. Includes error recovery and detailed error messages.

use lugli_ast::{Program, SpanMap};
use lugli_lexer::LexError;
use thiserror::Error;

pub mod error;
mod expressions;
mod helpers;
mod literals;
pub mod parser;
pub mod precedence;
mod statements;

pub use error::ParseError;
pub use parser::Parser;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lex(#[from] LexError),
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}

pub fn parse(source: &str) -> Result<(Program, SpanMap), ParserError> {
    let mut parser = Parser::new(source)?;
    Ok(parser.parse()?)
}

// Error conversion methods
impl ParserError {
    /// Convert ParserError to LugliError with proper span information
    pub fn to_lugli_error(&self) -> lugli_common::LugliError {
        match self {
            ParserError::Lex(lex_err) => lugli_common::LugliError::Lex {
                message: lex_err.to_string(),
                span: lugli_common::Span {
                    start: 0,
                    end: 0,
                },
            },
            ParserError::Parse(parse_err) => {
                let span = match parse_err {
                    ParseError::UnexpectedToken {
                        token,
                    } => token.span,
                    ParseError::Expected {
                        found, ..
                    } => found.span,
                    ParseError::Custom {
                        span, ..
                    } => *span,
                    _ => lugli_common::Span {
                        start: 0,
                        end: 0,
                    },
                };
                lugli_common::LugliError::Parse {
                    message: parse_err.to_string(),
                    span,
                }
            }
        }
    }
}
