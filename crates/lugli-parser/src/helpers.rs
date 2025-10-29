use lugli_lexer::{Token, TokenKind};
use lugli_common::Span;
use lugli_ast::FStringPart;
use crate::error::ParseError;
use crate::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn match_any(&mut self, types: &[TokenKind]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub(crate) fn check(&self, token_type: &TokenKind) -> bool {
        self.scanner.current()
            .map(|token| &token.kind == token_type)
            .unwrap_or(false)
    }

    pub(crate) fn advance(&mut self) {
        self.previous = self.scanner.current().cloned();
        let _ = self.scanner.advance();
    }

    pub(crate) fn previous(&self) -> &Token {
        self.previous.as_ref().expect("BUG: No previous token (advance() must be called before previous())")
    }

    pub(crate) fn previous_span(&self) -> Span {
        self.previous.as_ref()
            .map(|t| t.span)
            .unwrap_or(Span { start: 0, end: 0 })
    }

    pub(crate) fn current_span(&self) -> Span {
        self.scanner.current()
            .map(|t| t.span)
            .unwrap_or(Span { start: 0, end: 0 })
    }

    pub(crate) fn merge_spans(&self, start: Span, end: Span) -> Span {
        Span {
            start: start.start,
            end: end.end,
        }
    }

    pub(crate) fn span_from_to(&self, start: usize, end: usize) -> Span {
        Span { start, end }
    }



    pub(crate) fn peek_kind(&self) -> Option<&TokenKind> {
        self.scanner.current().map(|token| &token.kind)
    }

    pub(crate) fn peek_next_kind(&self) -> Option<TokenKind> {
        self.scanner.peek().map(|token| token.kind.clone())
    }

    pub(crate) fn consume(&mut self, token_type: &TokenKind, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            if let Some(token) = self.scanner.current() {
                let token = token.clone();
                self.advance();
                Ok(token)
            } else {
                Err(ParseError::UnexpectedEof)
            }
        } else {
            Err(self.expected_error(message))
        }
    }

    pub(crate) fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if let Some(TokenKind::Identifier(name)) = self.peek_kind() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.expected_error(message))
        }
    }

    pub(crate) fn consume_statement_terminator(&mut self) -> Result<(), ParseError> {
        if self.check(&TokenKind::Newline) || self.check(&TokenKind::Semicolon) {
            self.advance();
        }
        Ok(())
    }

    pub(crate) fn expected_error(&self, expected: &str) -> ParseError {
        if let Some(token) = self.scanner.current() {
            ParseError::Expected {
                expected: expected.to_string(),
                found: token.clone(),
            }
        } else {
            ParseError::UnexpectedEof
        }
    }

    pub(crate) fn unexpected_token_error(&self, context: &str) -> ParseError {
        if let Some(token) = self.scanner.current() {
            ParseError::Custom {
                message: format!("Unexpected token {} {}", token, context),
                span: token.span,
            }
        } else {
            ParseError::UnexpectedEof
        }
    }

    pub(crate) fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    pub(crate) fn consume_with_newlines(&mut self, token_type: &TokenKind, message: &str) -> Result<Token, ParseError> {
        let token = self.consume(token_type, message)?;
        if self.should_skip_newlines_after(token_type) {
            self.skip_newlines();
        }
        Ok(token)
    }

    pub(crate) fn consume_closing(&mut self, token_type: &TokenKind, message: &str) -> Result<Token, ParseError> {
        self.skip_newlines();
        self.consume(token_type, message)
    }

    pub(crate) fn match_any_with_newlines(&mut self, types: &[TokenKind]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                if self.should_skip_newlines_after(token_type) {
                    self.skip_newlines();
                }
                return true;
            }
        }
        false
    }	

    fn should_skip_newlines_after(&self, token_type: &TokenKind) -> bool {
        matches!(token_type,
            TokenKind::LeftBrace | TokenKind::LeftBracket | TokenKind::LeftParen |
            TokenKind::Comma | TokenKind::Colon 
        )
    }

    pub(crate) fn is_dict_literal(&self) -> bool {
        if !self.check(&TokenKind::LeftBrace) {
            return false;
        }

        if let Some(next_token) = self.scanner.peek() {
            match &next_token.kind {
                // Empty dict: {}
                TokenKind::RightBrace => true,
                // Immediate string key: {"key": ...}
                TokenKind::String(_) => true,
                // Multiline dict: {\n    "key": ...}
                // Be permissive with newlines and let expression parser validate
                TokenKind::Newline => true,
                // Identifier could be a variable reference for computed keys, not a dict
                TokenKind::Identifier(_) => false,
                // Other tokens suggest this might be a block statement
                _ => false,
            }
        } else {
            false
        }
    }

    pub(crate) fn parse_fstring_content(&mut self, content: &str) -> Result<Vec<FStringPart>, ParseError> {
        self.parse_fstring_content_with_depth(content, 0)
    }

    fn parse_fstring_content_with_depth(&mut self, content: &str, depth: usize) -> Result<Vec<FStringPart>, ParseError> {
        const MAX_FSTRING_DEPTH: usize = 10;

        if depth > MAX_FSTRING_DEPTH {
            return Err(ParseError::Custom {
                message: format!("F-string nesting exceeds maximum depth of {}", MAX_FSTRING_DEPTH),
                span: self.current_span(),
            });
        }

        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // Escaped brace: {{ → {
                    chars.next();
                    current_text.push('{');
                } else {
                    // Start of interpolation
                    if !current_text.is_empty() {
                        parts.push(FStringPart::Text(current_text.clone()));
                        current_text.clear();
                    }

                    // Extract expression until }
                    let mut expr_str = String::new();
                    let mut brace_depth = 1;
                    for ch in chars.by_ref() {
                        if ch == '{' {
                            brace_depth += 1;
                            expr_str.push(ch);
                        } else if ch == '}' {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                break;
                            }
                            expr_str.push(ch);
                        } else {
                            expr_str.push(ch);
                        }
                    }

                    if brace_depth != 0 {
                        return Err(ParseError::Custom {
                            message: "Unterminated expression in f-string".to_string(),
                            span: self.current_span(),
                        });
                    }

                    // Parse the expression by creating a sub-parser
                    let mut expr_parser = Parser::new(&expr_str)?;
                    let expr = expr_parser.expression()?;
                    parts.push(FStringPart::Expression(Box::new(expr)));
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    // Escaped brace: }} → }
                    chars.next();
                    current_text.push('}');
                } else {
                    return Err(ParseError::Custom {
                        message: "Unmatched } in f-string".to_string(),
                        span: self.current_span(),
                    });
                }
            } else {
                current_text.push(ch);
            }
        }

        if !current_text.is_empty() {
            parts.push(FStringPart::Text(current_text));
        }

        // Handle empty f-string
        if parts.is_empty() {
            parts.push(FStringPart::Text(String::new()));
        }

        Ok(parts)
    }

}