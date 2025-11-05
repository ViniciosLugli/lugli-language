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
