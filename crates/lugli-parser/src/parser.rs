use crate::error::ParseError;
use lugli_ast::Program;
use lugli_lexer::{Scanner, Token};

pub struct Parser<'a> {
    pub(crate) scanner: Scanner<'a>,
    pub(crate) previous: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let scanner = Scanner::new(source)?;
        Ok(Self {
            scanner,
            previous: None,
        })
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.scanner.is_at_end() {
            self.skip_newlines();

            if !self.scanner.is_at_end() {
                statements.push(self.statement()?);
            }
        }

        let end_span = self.scanner.current().map(|t| t.span.end).unwrap_or(0);

        Ok(Program::new(statements, self.span_from_to(0, end_span)))
    }
}
