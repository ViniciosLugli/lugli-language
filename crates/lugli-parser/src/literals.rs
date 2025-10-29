use crate::{Parser, error::ParseError};
use lugli_ast::{Expr, LiteralValue};
use lugli_lexer::TokenKind;

/// Check if a string is PascalCase (starts with uppercase, has lowercase letters)
/// This distinguishes struct names (PascalCase) from constants (ALL_CAPS)
fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first_char = s.chars().next().unwrap();
    if !first_char.is_uppercase() {
        return false;
    }

    // Accept both PascalCase and ALL_CAPS for struct names
    // Reject camelCase (starts with lowercase)
    true
}

impl<'a> Parser<'a> {
    fn parse_dict_key(&mut self) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::String(s)) => {
                let key = s[1..s.len() - 1].to_string();
                let span = self.current_span();
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::String(key),
                    span,
                })
            }
            Some(TokenKind::Identifier(s)) => {
                let key = s.clone();
                let span = self.current_span();
                self.advance();
                Ok(Expr::Literal {
                    value: LiteralValue::String(key),
                    span,
                })
            }
            _ => Err(self.expected_error("string or identifier for dictionary key")),
        }
    }

    pub(crate) fn primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(TokenKind::Number(value)) = self.peek_kind() {
            let value = *value;
            let span = self.current_span();
            self.advance();
            return Ok(Expr::Literal {
                value: LiteralValue::Number(value),
                span,
            });
        }

        if let Some(TokenKind::String(value)) = self.peek_kind() {
            let unquoted_value = value[1..value.len() - 1].to_string();
            let span = self.current_span();
            self.advance();
            return Ok(Expr::Literal {
                value: LiteralValue::String(unquoted_value),
                span,
            });
        }

        if let Some(TokenKind::FString(value)) = self.peek_kind() {
            // Clone the value to avoid borrow checker issues
            let value = value.clone();
            let span = self.current_span();
            self.advance();

            // Parse f-string interpolation
            let content = &value[2..value.len() - 1]; // Remove f" and "
            let parts = self.parse_fstring_content(content)?;

            return Ok(Expr::FString {
                parts,
                span,
            });
        }

        // Handle 'self' as a special identifier
        if self.check(&TokenKind::SelfKeyword) {
            let span = self.current_span();
            self.advance();
            return Ok(Expr::Identifier {
                name: "self".to_string(),
                span,
            });
        }

        if let Some(TokenKind::Identifier(name)) = self.peek_kind() {
            let span = self.current_span();
            let name = name.clone();
            self.advance();

            // Check for associated function call (Type::function)
            if self.match_any(&[TokenKind::DoubleColon]) {
                let method_name = self.consume_identifier("Expected method name after '::'")?;
                let full_span = self.merge_spans(span, self.previous_span());

                // For static methods, we'll look up the function as TypeName_methodName
                // This matches how impl blocks will store their methods
                let static_method_name = format!("{}_{}", name, method_name);

                return Ok(Expr::Identifier {
                    name: static_method_name,
                    span: full_span,
                });
            }

            // Check for struct literal (Type { field: value, ... })
            // Only treat as struct literal if the identifier is PascalCase (not ALL_CAPS constants)
            if self.check(&TokenKind::LeftBrace) && is_pascal_case(&name) {
                let struct_start_span = span;
                self.advance(); // consume '{'
                self.skip_newlines();

                let mut pairs = Vec::new();

                if !self.check(&TokenKind::RightBrace) {
                    loop {
                        // For struct literals, field names are identifiers
                        let field_name = self.consume_identifier("Expected field name")?;
                        self.consume(&TokenKind::Colon, "Expected ':' after field name")?;
                        let value_expr = self.expression()?;

                        // Convert field name to string literal for dict representation
                        let key_expr = Expr::Literal {
                            value: LiteralValue::String(field_name),
                            span: self.current_span(),
                        };

                        pairs.push((key_expr, value_expr));

                        if !self.match_any(&[TokenKind::Comma]) {
                            break;
                        }
                        self.skip_newlines();
                        if self.check(&TokenKind::RightBrace) {
                            break;
                        }
                    }
                }

                self.skip_newlines();
                self.consume(&TokenKind::RightBrace, "Expected '}' after struct fields")?;
                let end_span = self.previous_span();

                // For now, represent struct literals as dict construction with type metadata
                // The struct type name is stored as a special "__type__" field
                let mut struct_pairs = vec![(
                    Expr::Literal {
                        value: LiteralValue::String("__struct_type__".to_string()),
                        span: struct_start_span,
                    },
                    Expr::Literal {
                        value: LiteralValue::String(name),
                        span: struct_start_span,
                    },
                )];
                struct_pairs.extend(pairs);

                return Ok(Expr::Dict {
                    pairs: struct_pairs,
                    span: self.merge_spans(struct_start_span, end_span),
                });
            }

            return Ok(Expr::Identifier {
                name,
                span,
            });
        }

        if self.match_any(&[TokenKind::LeftParen]) {
            self.skip_newlines();
            let expr = self.expression()?;
            self.skip_newlines();
            self.consume(&TokenKind::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        if self.match_any(&[TokenKind::True]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Boolean(true),
                span: self.previous_span(),
            });
        }

        if self.match_any(&[TokenKind::False]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Boolean(false),
                span: self.previous_span(),
            });
        }

        if self.match_any(&[TokenKind::Null]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Null,
                span: self.previous_span(),
            });
        }

        if self.check(&TokenKind::LeftBracket) {
            return self.list_literal();
        }

        if self.check(&TokenKind::LeftBrace) {
            return self.dict_literal();
        }

        if self.check(&TokenKind::Fn) {
            return self.function_literal();
        }

        if self.check(&TokenKind::Match) {
            return self.match_expression();
        }

        Err(self.unexpected_token_error("in expression context"))
    }

    pub(crate) fn match_expression(&mut self) -> Result<Expr, ParseError> {
        use lugli_ast::{MatchArm, Pattern};

        let start_span = self.current_span();
        self.consume(&TokenKind::Match, "Expected 'match'")?;

        // Parse the value to match on
        let value = Box::new(self.expression()?);

        self.consume(&TokenKind::LeftBrace, "Expected '{' after match value")?;
        self.skip_newlines();

        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
            // Parse pattern
            let pattern = if let Some(TokenKind::Identifier(name)) = self.peek_kind() {
                if name == "_" {
                    self.advance();
                    Pattern::Wildcard
                } else {
                    let ident = name.clone();
                    self.advance();
                    Pattern::Identifier(ident)
                }
            } else if let Some(TokenKind::Number(n)) = self.peek_kind() {
                let num = *n;
                self.advance();
                Pattern::Literal(LiteralValue::Number(num))
            } else if let Some(TokenKind::String(s)) = self.peek_kind() {
                let str_val = s[1..s.len() - 1].to_string();
                self.advance();
                Pattern::Literal(LiteralValue::String(str_val))
            } else if self.match_any(&[TokenKind::True]) {
                Pattern::Literal(LiteralValue::Boolean(true))
            } else if self.match_any(&[TokenKind::False]) {
                Pattern::Literal(LiteralValue::Boolean(false))
            } else if self.match_any(&[TokenKind::Null]) {
                Pattern::Literal(LiteralValue::Null)
            } else {
                return Err(self.expected_error("pattern"));
            };

            // Optional guard (if condition)
            let guard = if self.match_any(&[TokenKind::If]) { Some(Box::new(self.expression()?)) } else { None };

            // Arrow
            self.consume(&TokenKind::FatArrow, "Expected '=>' after pattern")?;

            // Body expression
            let body = Box::new(self.expression()?);

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            // Optional comma
            self.match_any(&[TokenKind::Comma]);
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after match arms")?;
        let end_span = self.previous_span();

        Ok(Expr::Match {
            value,
            arms,
            span: self.merge_spans(start_span, end_span),
        })
    }

    pub(crate) fn list_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume_with_newlines(&TokenKind::LeftBracket, "Expected '['")?;

        if self.check(&TokenKind::RightBracket) {
            self.consume_closing(&TokenKind::RightBracket, "Expected ']'")?;
            let end_span = self.previous_span();
            return Ok(Expr::List {
                elements: Vec::new(),
                span: self.merge_spans(start_span, end_span),
            });
        }

        // Parse first element
        let first_element = self.expression()?;

        // Check for list comprehension: [expr for var in iterable if condition]
        if self.check(&TokenKind::For) {
            self.advance(); // consume 'for'

            let variable = self.consume_identifier("Expected variable name after 'for'")?;

            self.consume(&TokenKind::In, "Expected 'in' after variable in list comprehension")?;

            let iterable = self.expression()?;

            // Optional condition: if condition
            let condition = if self.check(&TokenKind::If) {
                self.advance(); // consume 'if'
                Some(Box::new(self.expression()?))
            } else {
                None
            };

            self.consume_closing(&TokenKind::RightBracket, "Expected ']' after list comprehension")?;
            let end_span = self.previous_span();

            return Ok(Expr::ListComprehension {
                element: Box::new(first_element),
                variable,
                iterable: Box::new(iterable),
                condition,
                span: self.merge_spans(start_span, end_span),
            });
        }

        // Regular list literal
        let mut elements = vec![first_element];

        while self.match_any_with_newlines(&[TokenKind::Comma]) {
            // Handle trailing comma - if we see the closing bracket after comma, break
            if self.check(&TokenKind::RightBracket) {
                break;
            }
            elements.push(self.expression()?);
        }

        self.consume_closing(&TokenKind::RightBracket, "Expected ']' after list elements")?;
        let end_span = self.previous_span();

        Ok(Expr::List {
            elements,
            span: self.merge_spans(start_span, end_span),
        })
    }

    pub(crate) fn dict_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::LeftBrace, "Expected '{'")?;
        self.skip_newlines();

        let mut pairs = Vec::new();

        if !self.check(&TokenKind::RightBrace) {
            loop {
                let key_expr = self.parse_dict_key()?;
                self.consume(&TokenKind::Colon, "Expected ':' after dictionary key")?;
                let value_expr = self.expression()?;

                pairs.push((key_expr, value_expr));

                if !self.match_any(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_newlines();
                // Handle trailing comma - if we see the closing brace after comma, break
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }
        }

        self.skip_newlines();
        self.consume(&TokenKind::RightBrace, "Expected '}' after dictionary")?;
        let end_span = self.previous_span();

        Ok(Expr::Dict {
            pairs,
            span: self.merge_spans(start_span, end_span),
        })
    }

    pub(crate) fn function_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Fn, "Expected 'fn'")?;

        self.consume(&TokenKind::LeftParen, "Expected '(' after 'fn'")?;
        self.skip_newlines();

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;

                // Skip type hints if present
                if self.match_any(&[TokenKind::Colon]) {
                    while !self.check(&TokenKind::Comma) && !self.check(&TokenKind::RightParen) {
                        self.advance();
                    }
                }

                params.push(param_name);

                if !self.match_any(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_newlines();
            }
        }

        self.skip_newlines();
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;

        // Skip return type hint if present
        if self.match_any(&[TokenKind::Arrow]) {
            while !self.check(&TokenKind::LeftBrace) && !self.scanner.is_at_end() {
                self.advance();
            }
        }

        let body = self.block_body()?;

        let end_span = self.previous_span();

        Ok(Expr::Function {
            params,
            body,
            span: self.merge_spans(start_span, end_span),
        })
    }
}
