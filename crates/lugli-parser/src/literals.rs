use lugli_lexer::TokenKind;
use lugli_ast::{Expr, LiteralValue};
use crate::error::ParseError;
use crate::Parser;

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
            let unquoted_value = value[1..value.len() - 1].to_string();
            let span = self.current_span();
            self.advance();
            // For now, treat f-strings as regular strings - proper interpolation will be handled later
            return Ok(Expr::Literal {
                value: LiteralValue::String(unquoted_value),
                span,
            });
        }

        if let Some(TokenKind::Identifier(name)) = self.peek_kind() {
            let span = self.current_span();
            let name = name.clone();
            self.advance();
            return Ok(Expr::Identifier { name, span });
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

        Err(self.unexpected_token_error("in expression context"))
    }

    pub(crate) fn list_literal(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.consume_with_newlines(&TokenKind::LeftBracket, "Expected '['")?;

        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                elements.push(self.expression()?);
                if !self.match_any_with_newlines(&[TokenKind::Comma]) {
                    break;
                }
                // Handle trailing comma - if we see the closing bracket after comma, break
                if self.check(&TokenKind::RightBracket) {
                    break;
                }
            }
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
}