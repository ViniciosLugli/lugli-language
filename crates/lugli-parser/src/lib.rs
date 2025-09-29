use lugli_lexer::LexError;
use lugli_ast::Program;
use thiserror::Error;

pub mod parser;
pub mod precedence;
pub mod error;
mod helpers;
mod literals;
mod expressions;
mod statements;

pub use parser::Parser;
pub use error::ParseError;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: {0}")]
    Lex(#[from] LexError),
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}

pub fn parse(source: &str) -> Result<Program, ParserError> {
    let mut parser = Parser::new(source)?;
    Ok(parser.parse()?)
}

