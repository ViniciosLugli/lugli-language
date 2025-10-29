use crate::error::ParseError;
use lugli_ast::{Program, SpanMap};
use lugli_lexer::{Scanner, Token};

pub struct Parser<'a> {
    pub(crate) scanner: Scanner<'a>,
    pub(crate) previous: Option<Token>,
    pub(crate) span_map: SpanMap,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let scanner = Scanner::new(source)?;
        Ok(Self {
            scanner,
            previous: None,
            span_map: SpanMap::new(),
        })
    }

    pub fn parse(&mut self) -> Result<(Program, SpanMap), ParseError> {
        let mut statements = Vec::new();

        while !self.scanner.is_at_end() {
            self.skip_newlines();

            if !self.scanner.is_at_end() {
                statements.push(self.statement()?);
            }
        }

        let end_span = self.scanner.current().map(|t| t.span.end).unwrap_or(0);
        let span = self.span_from_to(0, end_span);

        let program = Program::new(statements, span);
        Ok((program, self.span_map.clone()))
    }
}
