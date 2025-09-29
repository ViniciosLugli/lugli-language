
use crate::{LexError, Lexer, Token, TokenKind};

pub struct Scanner<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
    peek: Option<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Result<Self, LexError> {
        let mut lexer = Lexer::new(source);
        let current = lexer.next().transpose()?;
        let peek = lexer.next().transpose()?;

        Ok(Self {
            lexer,
            current,
            peek,
        })
    }

    pub fn current(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    pub fn peek(&self) -> Option<&Token> {
        self.peek.as_ref()
    }

    pub fn advance(&mut self) -> Result<Option<Token>, LexError> {
        let previous = self.current.take();
        self.current = self.peek.take();
        self.peek = self.lexer.next().transpose()?;
        Ok(previous)
    }

    pub fn check(&self, kind: &TokenKind) -> bool {
        self.current
            .as_ref()
            .map(|token| std::mem::discriminant(&token.kind) == std::mem::discriminant(kind))
            .unwrap_or(false)
    }

    pub fn match_token(&mut self, kind: &TokenKind) -> Result<bool, LexError> {
        if self.check(kind) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current
            .as_ref()
            .map(|token| matches!(token.kind, TokenKind::Eof))
            .unwrap_or(true)
    }
}