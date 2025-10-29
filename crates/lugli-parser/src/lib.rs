use lugli_ast::Program;
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

pub fn parse(source: &str) -> Result<Program, ParserError> {
    let mut parser = Parser::new(source)?;
    Ok(parser.parse()?)
}
