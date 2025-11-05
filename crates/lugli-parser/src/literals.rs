use crate::{Parser, error::ParseError};
use lugli_ast::{Expr, ListComprehensionData, LiteralValue};
use lugli_lexer::TokenKind;

fn is_pascal_case(s: &str) -> bool {
    let first_char = match s.chars().next() {
        Some(c) => c,
        None => return false,
    };
    if !first_char.is_uppercase() {
        return false;
    }

    true
}

impl<'a> Parser<'a> {
    pub(crate) fn block_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::LeftBrace, "Expected '{'")?;
        self.skip_newlines();

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
            self.skip_newlines();

            if !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
                statements.push(self.statement()?);
            }
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after block")?;
        let end_span = self.previous_span();

        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::Block {
            id,
            statements,
        })
    }

    fn parse_dict_key(&mut self) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::String(id)) => {
                let s = self.scanner.pool().resolve(*id);
                let key = s[1..s.len() - 1].to_string();
                let span = self.current_span();
                self.advance();

                let node_id = self.span_map.alloc_id();
                self.span_map.insert(node_id, span);

                Ok(Expr::Literal {
                    id: node_id,
                    value: LiteralValue::String(key),
                })
            }
            Some(TokenKind::Identifier(id)) => {
                let key = self.scanner.pool().resolve(*id).to_string();
                let span = self.current_span();
                self.advance();

                let node_id = self.span_map.alloc_id();
                self.span_map.insert(node_id, span);

                Ok(Expr::Literal {
                    id: node_id,
                    value: LiteralValue::String(key),
                })
            }
            Some(TokenKind::Number(n)) => {
                // Support numeric keys (converted to strings for storage)
                let key = n.to_string();
                let span = self.current_span();
                self.advance();

                let node_id = self.span_map.alloc_id();
                self.span_map.insert(node_id, span);

                Ok(Expr::Literal {
                    id: node_id,
                    value: LiteralValue::String(key),
                })
            }
            _ => Err(self.expected_error("string, identifier, or number for dictionary key")),
        }
    }

    pub(crate) fn primary(&mut self) -> Result<Expr, ParseError> {
        if let Some(TokenKind::Number(value)) = self.peek_kind() {
            let value = *value;
            let span = self.current_span();
            self.advance();

            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Literal {
                id,
                value: LiteralValue::Number(value),
            });
        }

        if let Some(TokenKind::String(id)) = self.peek_kind() {
            let s = self.scanner.pool().resolve(*id);
            let unquoted_value = s[1..s.len() - 1].to_string();
            let span = self.current_span();
            self.advance();

            let node_id = self.span_map.alloc_id();
            self.span_map.insert(node_id, span);

            return Ok(Expr::Literal {
                id: node_id,
                value: LiteralValue::String(unquoted_value),
            });
        }

        if let Some(TokenKind::FString(id)) = self.peek_kind() {
            let s = self.scanner.pool().resolve(*id);
            let value = s.to_string();
            let span = self.current_span();
            self.advance();

            // Parse f-string interpolation
            let content = &value[2..value.len() - 1]; // Remove f" and "
            let parts = self.parse_fstring_content(content)?;

            let node_id = self.span_map.alloc_id();
            self.span_map.insert(node_id, span);

            return Ok(Expr::FString {
                id: node_id,
                parts: Box::new(parts),
            });
        }

        // Handle 'self' as a special identifier
        if self.check(&TokenKind::SelfKeyword) {
            let span = self.current_span();
            self.advance();

            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Identifier {
                id,
                name: "self".to_string(),
            });
        }

        if let Some(TokenKind::Identifier(id)) = self.peek_kind() {
            let span = self.current_span();
            let name = self.scanner.pool().resolve(*id).to_string();
            self.advance();

            // Check for associated function call (Type::function)
            if self.match_any(&[TokenKind::DoubleColon]) {
                let method_name = self.consume_identifier("Expected method name after '::'")?;
                let full_span = self.merge_spans(span, self.previous_span());

                let static_method_name = format!("{}_{}", name, method_name);

                let node_id = self.span_map.alloc_id();
                self.span_map.insert(node_id, full_span);

                return Ok(Expr::Identifier {
                    id: node_id,
                    name: static_method_name,
                });
            }

            // Check for struct literal (Type { field: value, ... })
            if self.check(&TokenKind::LeftBrace) && is_pascal_case(&name) {
                let struct_start_span = span;
                self.advance(); // consume '{'
                self.skip_newlines();

                let mut pairs = Vec::new();

                if !self.check(&TokenKind::RightBrace) {
                    loop {
                        let field_name = self.consume_identifier("Expected field name")?;
                        self.consume(&TokenKind::Colon, "Expected ':' after field name")?;
                        let value_expr = self.expression()?;

                        let field_span = self.current_span();
                        let key_id = self.span_map.alloc_id();
                        self.span_map.insert(key_id, field_span);

                        let key_expr = Expr::Literal {
                            id: key_id,
                            value: LiteralValue::String(field_name),
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

                // Create __struct_type__ field
                let type_key_id = self.span_map.alloc_id();
                self.span_map.insert(type_key_id, struct_start_span);
                let type_value_id = self.span_map.alloc_id();
                self.span_map.insert(type_value_id, struct_start_span);

                let mut struct_pairs = vec![(
                    Expr::Literal {
                        id: type_key_id,
                        value: LiteralValue::String("__struct_type__".to_string()),
                    },
                    Expr::Literal {
                        id: type_value_id,
                        value: LiteralValue::String(name),
                    },
                )];
                struct_pairs.extend(pairs);

                let dict_span = self.merge_spans(struct_start_span, end_span);
                let dict_id = self.span_map.alloc_id();
                self.span_map.insert(dict_id, dict_span);

                return Ok(Expr::Dict {
                    id: dict_id,
                    pairs: struct_pairs,
                });
            }

            let node_id = self.span_map.alloc_id();
            self.span_map.insert(node_id, span);

            return Ok(Expr::Identifier {
                id: node_id,
                name,
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
            let span = self.previous_span();
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Literal {
                id,
                value: LiteralValue::Boolean(true),
            });
        }

        if self.match_any(&[TokenKind::False]) {
            let span = self.previous_span();
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Literal {
                id,
                value: LiteralValue::Boolean(false),
            });
        }

        if self.match_any(&[TokenKind::Null]) {
            let span = self.previous_span();
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Literal {
                id,
                value: LiteralValue::Null,
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

        if self.check(&TokenKind::If) {
            return self.if_expression();
        }

        Err(self.unexpected_token_error("in expression context"))
    }

    pub(crate) fn if_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::If, "Expected 'if'")?;

        let condition = Box::new(self.expression()?);

        let then_branch = Box::new(if self.check(&TokenKind::LeftBrace) {
            self.block_expression()?
        } else {
            return Err(self.expected_error("'{' after if condition in expression context"));
        });

        let else_branch = if self.match_any(&[TokenKind::Else]) {
            if self.check(&TokenKind::If) {
                Some(Box::new(self.if_expression()?))
            } else if self.check(&TokenKind::LeftBrace) {
                Some(Box::new(self.block_expression()?))
            } else {
                return Err(self.expected_error("'{' or 'if' after 'else' in expression context"));
            }
        } else {
            None
        };

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::If {
            id,
            condition,
            then_branch,
            else_branch,
        })
    }

    pub(crate) fn match_expression(&mut self) -> Result<Expr, ParseError> {
        use lugli_ast::{MatchArm, Pattern};

        let start_span = self.current_span();
        self.consume(&TokenKind::Match, "Expected 'match'")?;

        let value = Box::new(self.expression()?);

        self.consume(&TokenKind::LeftBrace, "Expected '{' after match value")?;
        self.skip_newlines();

        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
            let pattern = if let Some(TokenKind::Identifier(id)) = self.peek_kind() {
                let name = self.scanner.pool().resolve(*id);
                if name == "_" {
                    self.advance();
                    Pattern::Wildcard
                } else {
                    let ident = name.to_string();
                    self.advance();
                    Pattern::Identifier(ident)
                }
            } else if let Some(TokenKind::Number(n)) = self.peek_kind() {
                let num = *n;
                self.advance();
                Pattern::Literal(LiteralValue::Number(num))
            } else if let Some(TokenKind::String(id)) = self.peek_kind() {
                let s = self.scanner.pool().resolve(*id);
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

            let guard = if self.match_any(&[TokenKind::If]) { Some(Box::new(self.expression()?)) } else { None };

            self.consume(&TokenKind::FatArrow, "Expected '=>' after pattern")?;

            let body = if self.check(&TokenKind::LeftBrace) {
                Box::new(self.block_expression()?)
            } else {
                Box::new(self.expression()?)
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            self.match_any(&[TokenKind::Comma]);
            self.skip_newlines();
        }

        self.consume(&TokenKind::RightBrace, "Expected '}' after match arms")?;
        let end_span = self.previous_span();

        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::Match {
            id,
            value,
            arms,
        })
    }

    pub(crate) fn list_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume_with_newlines(&TokenKind::LeftBracket, "Expected '['")?;

        if self.check(&TokenKind::RightBracket) {
            self.consume_closing(&TokenKind::RightBracket, "Expected ']'")?;
            let end_span = self.previous_span();
            let span = self.merge_spans(start_span, end_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::List {
                id,
                elements: Vec::new(),
            });
        }

        let first_element = self.expression()?;

        // Check for list comprehension: [expr for var in iterable if condition for var2 in iterable2 ...]
        if self.check(&TokenKind::For) {
            use lugli_ast::ComprehensionClause;
            let mut clauses = Vec::new();

            while self.check(&TokenKind::For) {
                self.advance(); // consume 'for'

                let variable = self.consume_identifier("Expected variable name after 'for'")?;

                self.consume(&TokenKind::In, "Expected 'in' after variable in list comprehension")?;

                let iterable = self.expression()?;

                let condition = if self.check(&TokenKind::If) {
                    self.advance(); // consume 'if'
                    Some(self.expression()?)
                } else {
                    None
                };

                clauses.push(ComprehensionClause {
                    variable,
                    iterable,
                    condition,
                });
            }

            self.consume_closing(&TokenKind::RightBracket, "Expected ']' after list comprehension")?;
            let end_span = self.previous_span();

            let span = self.merge_spans(start_span, end_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::ListComprehension {
                id,
                data: Box::new(ListComprehensionData {
                    element: first_element,
                    clauses,
                }),
            });
        }

        // Regular list literal
        let mut elements = vec![first_element];

        if self.match_any(&[TokenKind::Comma]) {
            self.skip_newlines();
            if !self.check(&TokenKind::RightBracket) {
                let rest = self.parse_delimited(&TokenKind::RightBracket, &TokenKind::Comma, |p| p.expression())?;
                elements.extend(rest);
            }
        }

        self.consume_closing(&TokenKind::RightBracket, "Expected ']' after list elements")?;
        let end_span = self.previous_span();

        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::List {
            id,
            elements,
        })
    }

    pub(crate) fn dict_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::LeftBrace, "Expected '{'")?;
        self.skip_newlines();

        let pairs = self.parse_delimited(&TokenKind::RightBrace, &TokenKind::Comma, |p| {
            let key_expr = p.parse_dict_key()?;
            p.consume(&TokenKind::Colon, "Expected ':' after dictionary key")?;
            let value_expr = p.expression()?;
            Ok((key_expr, value_expr))
        })?;

        self.skip_newlines();
        self.consume(&TokenKind::RightBrace, "Expected '}' after dictionary")?;
        let end_span = self.previous_span();

        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::Dict {
            id,
            pairs,
        })
    }

    pub(crate) fn function_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Fn, "Expected 'fn'")?;

        self.consume(&TokenKind::LeftParen, "Expected '(' after 'fn'")?;
        self.skip_newlines();

        let parsed_params = self.parse_params()?;
        let params: Vec<String> = parsed_params.iter().map(|p| p.name.clone()).collect();

        self.skip_newlines();
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;

        if self.match_any(&[TokenKind::Arrow]) {
            while !self.check(&TokenKind::LeftBrace) && !self.scanner.is_at_end() {
                self.advance();
            }
        }

        let body = self.block_body()?;

        let end_span = self.previous_span();

        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::Function {
            id,
            params,
            body,
        })
    }
}
