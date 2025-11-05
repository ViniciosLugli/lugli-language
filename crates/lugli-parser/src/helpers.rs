use crate::{Parser, error::ParseError};
use lugli_ast::{FStringPart, TypeHint};
use lugli_common::Span;
use lugli_lexer::{Token, TokenKind};
use std::sync::OnceLock;

static ERROR_TOKEN: OnceLock<Token> = OnceLock::new();

pub struct Param {
    pub name: String,
    pub type_hint: Option<TypeHint>,
}

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

    pub(crate) fn check(&self, token_type: &TokenKind) -> bool { self.scanner.current().map(|token| &token.kind == token_type).unwrap_or(false) }

    pub(crate) fn advance(&mut self) {
        self.previous = self.scanner.current().cloned();
        let _ = self.scanner.advance();
    }

    pub(crate) fn previous(&self) -> &Token {
        self.previous.as_ref().unwrap_or_else(|| {
            ERROR_TOKEN.get_or_init(|| Token {
                kind: TokenKind::Eof,
                span: Span {
                    start: 0,
                    end: 0,
                },
            })
        })
    }

    pub(crate) fn previous_span(&self) -> Span {
        self.previous.as_ref().map(|t| t.span).unwrap_or(Span {
            start: 0,
            end: 0,
        })
    }

    pub(crate) fn current_span(&self) -> Span {
        self.scanner.current().map(|t| t.span).unwrap_or(Span {
            start: 0,
            end: 0,
        })
    }

    pub(crate) fn merge_spans(&self, start: Span, end: Span) -> Span {
        Span {
            start: start.start,
            end: end.end,
        }
    }

    pub(crate) fn span_from_to(&self, start: usize, end: usize) -> Span {
        Span {
            start,
            end,
        }
    }

    /// Get the span for an expression ID from the span map.
    /// This should never fail as all parsed expressions must have spans.
    /// If this panics, it indicates a compiler bug.
    pub(crate) fn get_expr_span(&self, node_id: lugli_ast::NodeId) -> Span {
        self.span_map.get(node_id).expect("BUG: Expression missing from span map - all parsed expressions must have spans")
    }

    pub(crate) fn peek_kind(&self) -> Option<&TokenKind> { self.scanner.current().map(|token| &token.kind) }

    pub(crate) fn peek_next_kind(&self) -> Option<TokenKind> { self.scanner.peek().map(|token| token.kind.clone()) }

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
        if let Some(TokenKind::Identifier(id)) = self.peek_kind() {
            let name = self.scanner.pool().resolve(*id).to_string();
            self.advance();
            Ok(name)
        } else {
            Err(self.expected_error(message))
        }
    }

    /// Consume an identifier OR a keyword token, returning it as a string.
    /// This allows keywords like `from`, `where`, `match`, `if`, etc. to be used
    /// as method/property names in contexts where they're unambiguous.
    pub(crate) fn consume_identifier_or_keyword(&mut self, message: &str) -> Result<String, ParseError> {
        if let Some(kind) = self.peek_kind() {
            match kind {
                TokenKind::Identifier(id) => {
                    let name = self.scanner.pool().resolve(*id).to_string();
                    self.advance();
                    Ok(name)
                }
                // Allow all keywords as identifiers in this context
                TokenKind::From => {
                    self.advance();
                    Ok("from".to_string())
                }
                TokenKind::Where => {
                    self.advance();
                    Ok("where".to_string())
                }
                TokenKind::Match => {
                    self.advance();
                    Ok("match".to_string())
                }
                TokenKind::If => {
                    self.advance();
                    Ok("if".to_string())
                }
                TokenKind::Else => {
                    self.advance();
                    Ok("else".to_string())
                }
                TokenKind::For => {
                    self.advance();
                    Ok("for".to_string())
                }
                TokenKind::While => {
                    self.advance();
                    Ok("while".to_string())
                }
                TokenKind::Loop => {
                    self.advance();
                    Ok("loop".to_string())
                }
                TokenKind::In => {
                    self.advance();
                    Ok("in".to_string())
                }
                TokenKind::Break => {
                    self.advance();
                    Ok("break".to_string())
                }
                TokenKind::Continue => {
                    self.advance();
                    Ok("continue".to_string())
                }
                TokenKind::Return => {
                    self.advance();
                    Ok("return".to_string())
                }
                TokenKind::Fn => {
                    self.advance();
                    Ok("fn".to_string())
                }
                TokenKind::Struct => {
                    self.advance();
                    Ok("struct".to_string())
                }
                TokenKind::Impl => {
                    self.advance();
                    Ok("impl".to_string())
                }
                TokenKind::Trait => {
                    self.advance();
                    Ok("trait".to_string())
                }
                TokenKind::Type => {
                    self.advance();
                    Ok("type".to_string())
                }
                TokenKind::Enum => {
                    self.advance();
                    Ok("enum".to_string())
                }
                TokenKind::Class => {
                    self.advance();
                    Ok("class".to_string())
                }
                TokenKind::Let => {
                    self.advance();
                    Ok("let".to_string())
                }
                TokenKind::Mut => {
                    self.advance();
                    Ok("mut".to_string())
                }
                TokenKind::Const => {
                    self.advance();
                    Ok("const".to_string())
                }
                TokenKind::Import => {
                    self.advance();
                    Ok("import".to_string())
                }
                TokenKind::Export => {
                    self.advance();
                    Ok("export".to_string())
                }
                TokenKind::As => {
                    self.advance();
                    Ok("as".to_string())
                }
                TokenKind::Pub => {
                    self.advance();
                    Ok("pub".to_string())
                }
                TokenKind::Async => {
                    self.advance();
                    Ok("async".to_string())
                }
                TokenKind::Await => {
                    self.advance();
                    Ok("await".to_string())
                }
                TokenKind::Try => {
                    self.advance();
                    Ok("try".to_string())
                }
                TokenKind::Catch => {
                    self.advance();
                    Ok("catch".to_string())
                }
                TokenKind::Throw => {
                    self.advance();
                    Ok("throw".to_string())
                }
                TokenKind::Panic => {
                    self.advance();
                    Ok("panic".to_string())
                }
                TokenKind::Not => {
                    self.advance();
                    Ok("not".to_string())
                }
                // Note: we don't allow 'self' as a regular identifier since it has special semantics
                _ => Err(self.expected_error(message)),
            }
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
        matches!(token_type, TokenKind::LeftBrace | TokenKind::LeftBracket | TokenKind::LeftParen | TokenKind::Comma | TokenKind::Colon)
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

    pub(crate) fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        if self.check(&TokenKind::RightParen) {
            return Ok(params);
        }

        loop {
            if self.check(&TokenKind::SelfKeyword) {
                self.advance();
                params.push(Param {
                    name: "self".to_string(),
                    type_hint: None,
                });
            } else if self.check(&TokenKind::Mut) && self.peek_next_kind() == Some(TokenKind::SelfKeyword) {
                self.advance(); // consume mut
                self.advance(); // consume self
                params.push(Param {
                    name: "self".to_string(),
                    type_hint: None,
                });
            } else {
                // Allow keywords as parameter names
                let param_name = self.consume_identifier_or_keyword("Expected parameter name")?;

                // Parse optional type hint
                let type_hint = if self.match_any(&[TokenKind::Colon]) { Some(self.parse_type_hint()?) } else { None };

                params.push(Param {
                    name: param_name,
                    type_hint,
                });
            }

            if !self.match_any(&[TokenKind::Comma]) {
                break;
            }
            self.skip_newlines();
        }

        Ok(params)
    }

    pub(crate) fn parse_delimited<T, F>(&mut self, end_token: &TokenKind, separator: &TokenKind, parse_fn: F) -> Result<Vec<T>, ParseError>
    where F: Fn(&mut Self) -> Result<T, ParseError> {
        let mut items = Vec::new();

        if self.check(end_token) {
            return Ok(items);
        }

        loop {
            items.push(parse_fn(self)?);

            if !self.match_any(std::slice::from_ref(separator)) {
                break;
            }

            self.skip_newlines();
            if self.check(end_token) {
                break;
            }
        }

        Ok(items)
    }

    pub(crate) fn parse_type_hint(&mut self) -> Result<TypeHint, ParseError> {
        let base_name = self.consume_identifier("Expected type name")?;

        // Check for generic type: List<T>, Dict<K, V>
        if self.match_any(&[TokenKind::Less]) {
            let args = self.parse_type_arguments()?;
            self.consume(&TokenKind::Greater, "Expected '>' after type arguments")?;

            return Ok(TypeHint::Generic {
                base: base_name,
                args,
            });
        }

        // Check for optional: T?
        if self.match_any(&[TokenKind::Question]) {
            return Ok(TypeHint::Optional(Box::new(TypeHint::Simple(base_name))));
        }

        // Check for union: T | U | V
        if self.match_any(&[TokenKind::Pipe]) {
            let mut types = vec![TypeHint::Simple(base_name)];
            loop {
                let next_name = self.consume_identifier("Expected type name")?;
                types.push(TypeHint::Simple(next_name));
                if !self.match_any(&[TokenKind::Pipe]) {
                    break;
                }
            }
            return Ok(TypeHint::Union(types));
        }

        // Simple type
        Ok(TypeHint::Simple(base_name))
    }

    fn parse_type_arguments(&mut self) -> Result<Vec<TypeHint>, ParseError> {
        let mut args = Vec::new();

        if self.check(&TokenKind::Greater) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_type_hint()?);

            if !self.match_any(&[TokenKind::Comma]) {
                break;
            }
        }

        Ok(args)
    }

    pub(crate) fn track_binary_span(&mut self, left: lugli_ast::NodeId, right: lugli_ast::NodeId) -> lugli_ast::NodeId {
        let span = self.merge_spans(self.get_expr_span(left), self.get_expr_span(right));
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);
        id
    }

    pub(crate) fn track_unary_span(&mut self, token_span: Span, operand: lugli_ast::NodeId) -> lugli_ast::NodeId {
        let span = self.merge_spans(token_span, self.get_expr_span(operand));
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);
        id
    }

    pub(crate) fn track_span(&mut self, span: Span) -> lugli_ast::NodeId {
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params_empty() {
        let source = "fn test() {}";
        let mut parser = Parser::new(source).unwrap();
        parser.advance(); // fn
        parser.advance(); // test
        parser.advance(); // (
        let params = parser.parse_params().unwrap();
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_parse_params_simple() {
        let source = "fn test(x, y, z) {}";
        let mut parser = Parser::new(source).unwrap();
        parser.advance(); // fn
        parser.advance(); // test
        parser.advance(); // (
        let params = parser.parse_params().unwrap();
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[1].name, "y");
        assert_eq!(params[2].name, "z");
    }

    #[test]
    fn test_parse_params_with_self() {
        let source = "fn test(self, x) {}";
        let mut parser = Parser::new(source).unwrap();
        parser.advance(); // fn
        parser.advance(); // test
        parser.advance(); // (
        let params = parser.parse_params().unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "self");
        assert_eq!(params[1].name, "x");
    }

    #[test]
    fn test_parse_params_with_type_hints() {
        let source = "fn test(x: num, y: str) {}";
        let mut parser = Parser::new(source).unwrap();
        parser.advance(); // fn
        parser.advance(); // test
        parser.advance(); // (
        let params = parser.parse_params().unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[1].name, "y");
    }

    #[test]
    fn test_parse_delimited_empty() {
        let source = "]";
        let mut parser = Parser::new(source).unwrap();
        let items: Vec<String> =
            parser.parse_delimited(&TokenKind::RightBracket, &TokenKind::Comma, |p| p.consume_identifier("Expected identifier")).unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_parse_delimited_single() {
        let source = "x]";
        let mut parser = Parser::new(source).unwrap();
        let items = parser.parse_delimited(&TokenKind::RightBracket, &TokenKind::Comma, |p| p.consume_identifier("Expected identifier")).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "x");
    }

    #[test]
    fn test_parse_delimited_multiple() {
        let source = "a, b, c]";
        let mut parser = Parser::new(source).unwrap();
        let items = parser.parse_delimited(&TokenKind::RightBracket, &TokenKind::Comma, |p| p.consume_identifier("Expected identifier")).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "a");
        assert_eq!(items[1], "b");
        assert_eq!(items[2], "c");
    }
}
