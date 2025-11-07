use crate::{Parser, error::ParseError, literals::unescape_string};
use lugli_ast::{FStringPart, Pattern, TypeHint};
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
        macro_rules! keyword_as_ident {
            ($($variant:ident => $str:literal),* $(,)?) => {
                match self.peek_kind() {
                    Some(TokenKind::Identifier(id)) => {
                        let name = self.scanner.pool().resolve(*id).to_string();
                        self.advance();
                        return Ok(name);
                    }
                    $(
                        Some(TokenKind::$variant) => {
                            self.advance();
                            return Ok($str.to_string());
                        }
                    )*
                    _ => {}
                }
            };
        }

        keyword_as_ident! {
            From => "from", Where => "where", Match => "match", If => "if",
            Else => "else", For => "for", While => "while", Loop => "loop",
            In => "in", Break => "break", Continue => "continue", Return => "return",
            Fn => "fn", Struct => "struct", Impl => "impl", Trait => "trait",
            Type => "type", Enum => "enum", Class => "class", Let => "let",
            Mut => "mut", Const => "const", Import => "import", Export => "export",
            As => "as", Pub => "pub", Async => "async", Await => "await",
            Try => "try", Catch => "catch", Throw => "throw", Panic => "panic",
            Not => "not",
        }

        Err(self.expected_error(message))
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
                        parts.push(FStringPart::Text(unescape_string(&current_text)?));
                        current_text.clear();
                    }

                    // Extract expression with string context tracking
                    let expr_str = self.extract_interpolation_expression(&mut chars)?;

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
            parts.push(FStringPart::Text(unescape_string(&current_text)?));
        }

        // Handle empty f-string
        if parts.is_empty() {
            parts.push(FStringPart::Text(String::new()));
        }

        Ok(parts)
    }

    fn extract_interpolation_expression(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, ParseError> {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum StringType {
            Regular,
            FString,
        }

        let mut expr_str = String::new();
        let mut brace_depth = 1;
        let mut delimiter_stack: Vec<(char, StringType)> = Vec::new();
        let mut escape_next = false;

        while let Some(ch) = chars.next() {
            // Handle escape sequences
            if escape_next {
                expr_str.push(ch);
                escape_next = false;
                continue;
            }

            // Check for backslash (escape character) when inside a string
            if !delimiter_stack.is_empty() && ch == '\\' {
                expr_str.push(ch);
                escape_next = true;
                continue;
            }

            // Check for f-string prefix (f" or f') when NOT inside a string
            if delimiter_stack.is_empty()
                && ch == 'f'
                && let Some(&quote) = chars.peek()
                && (quote == '"' || quote == '\'')
            {
                expr_str.push(ch); // push 'f'
                if let Some(quote_ch) = chars.next() {
                    expr_str.push(quote_ch); // push quote
                    delimiter_stack.push((quote, StringType::FString));
                    continue;
                }
            }

            // Handle quote characters
            if ch == '"' || ch == '\'' {
                expr_str.push(ch);

                if let Some(&(delimiter, _string_type)) = delimiter_stack.last() {
                    // Inside a string - check if this closes it
                    if ch == delimiter {
                        // This quote matches the opening delimiter
                        delimiter_stack.pop();

                        // For f-strings, we need to check if there are interpolations
                        // The braces inside f-strings should be counted
                    } else {
                        // Different quote inside string - start a nested string
                        delimiter_stack.push((ch, StringType::Regular));
                    }
                } else {
                    // Not in a string - this opens a regular string
                    delimiter_stack.push((ch, StringType::Regular));
                }
                continue;
            }

            // Handle braces - the key logic for interpolation extraction
            if ch == '{' {
                // Count braces ONLY if:
                // 1. We're not inside any string, OR
                // 2. We're inside an f-string (but not inside a nested regular string)
                let should_count = if delimiter_stack.is_empty() {
                    true // Not in any string
                } else {
                    // Check if we're directly in an f-string (not nested in regular string inside it)
                    delimiter_stack.last() == Some(&('"', StringType::FString)) || delimiter_stack.last() == Some(&('\'', StringType::FString))
                };

                if should_count {
                    brace_depth += 1;
                }
                expr_str.push(ch);
            } else if ch == '}' {
                // Mirror logic for closing braces
                let should_count = if delimiter_stack.is_empty() {
                    true
                } else {
                    delimiter_stack.last() == Some(&('"', StringType::FString)) || delimiter_stack.last() == Some(&('\'', StringType::FString))
                };

                if should_count {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        // Found the matching closing brace for our interpolation
                        break;
                    }
                }
                expr_str.push(ch);
            } else {
                // Regular character
                expr_str.push(ch);
            }
        }

        // Validation: ensure all braces and strings are properly closed
        if brace_depth != 0 {
            return Err(ParseError::Custom {
                message: "Unterminated expression in f-string".to_string(),
                span: self.current_span(),
            });
        }

        if !delimiter_stack.is_empty() {
            return Err(ParseError::Custom {
                message: "Unterminated string in f-string expression".to_string(),
                span: self.current_span(),
            });
        }

        Ok(expr_str)
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

    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        if self.check(&TokenKind::LeftBracket) {
            self.parse_list_pattern()
        } else if self.check(&TokenKind::LeftBrace) {
            self.parse_dict_pattern()
        } else {
            let name = self.consume_identifier("Expected identifier in pattern")?;
            if name == "_" { Ok(Pattern::Wildcard) } else { Ok(Pattern::Identifier(name)) }
        }
    }

    fn parse_list_pattern(&mut self) -> Result<Pattern, ParseError> {
        self.consume(&TokenKind::LeftBracket, "Expected '['")?;
        let patterns = self.parse_delimited(&TokenKind::RightBracket, &TokenKind::Comma, |parser| parser.parse_pattern())?;
        self.consume(&TokenKind::RightBracket, "Expected ']' after list pattern")?;
        Ok(Pattern::List(patterns))
    }

    fn parse_dict_pattern(&mut self) -> Result<Pattern, ParseError> {
        self.consume(&TokenKind::LeftBrace, "Expected '{'")?;
        let mut fields = Vec::new();

        if !self.check(&TokenKind::RightBrace) {
            loop {
                let key = self.consume_identifier("Expected field name in dict pattern")?;

                let pattern = if self.match_any(&[TokenKind::Colon]) { self.parse_pattern()? } else { Pattern::Identifier(key.clone()) };

                fields.push((key, pattern));

                if !self.match_any(&[TokenKind::Comma]) {
                    break;
                }

                self.skip_newlines();
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after dict pattern")?;
        Ok(Pattern::Dict(fields))
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

    /// Convert an expression (List or Dict) to a pattern for destructuring assignment
    pub(crate) fn expr_to_pattern(&self, expr: &lugli_ast::Expr) -> Result<Pattern, ParseError> {
        match expr {
            lugli_ast::Expr::List {
                elements, ..
            } => {
                let mut patterns = Vec::new();
                for elem in elements {
                    patterns.push(self.expr_to_pattern(elem)?);
                }
                Ok(Pattern::List(patterns))
            }
            lugli_ast::Expr::Dict {
                pairs, ..
            } => {
                let mut fields = Vec::new();
                for (key_expr, value_expr) in pairs {
                    // Key must be a string literal or identifier
                    let key = match key_expr {
                        lugli_ast::Expr::Literal {
                            value: lugli_ast::LiteralValue::String(s), ..
                        } => s.clone(),
                        lugli_ast::Expr::Identifier {
                            name, ..
                        } => name.clone(),
                        _ => {
                            let span = self.get_expr_span(key_expr.id());
                            return Err(ParseError::Custom {
                                message: "Dict pattern keys must be identifiers or string literals".to_string(),
                                span,
                            });
                        }
                    };
                    let pattern = self.expr_to_pattern(value_expr)?;
                    fields.push((key, pattern));
                }
                Ok(Pattern::Dict(fields))
            }
            lugli_ast::Expr::Identifier {
                name, ..
            } => {
                if name == "_" {
                    Ok(Pattern::Wildcard)
                } else {
                    Ok(Pattern::Identifier(name.clone()))
                }
            }
            _ => {
                let span = self.get_expr_span(expr.id());
                Err(ParseError::Custom {
                    message: "Invalid pattern in destructuring assignment".to_string(),
                    span,
                })
            }
        }
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
