use crate::{Parser, error::ParseError};
use lugli_ast::{CallData, Expr};
use lugli_lexer::TokenKind;

impl<'a> Parser<'a> {
    pub(crate) fn expression(&mut self) -> Result<Expr, ParseError> { self.or() }

    fn parse_binary_left_associative(
        &mut self,
        operators: &[TokenKind],
        next_precedence: fn(&mut Self) -> Result<Expr, ParseError>,
    ) -> Result<Expr, ParseError> {
        let mut expr = next_precedence(self)?;

        while self.match_any(operators) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = next_precedence(self)?;

            let left_span = self.get_expr_span(expr.id());
            let right_span = self.get_expr_span(right.id());
            let span = self.merge_spans(left_span, right_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            expr = Expr::Binary {
                id,
                left: Box::new(expr),
                operator: operator_kind,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn or(&mut self) -> Result<Expr, ParseError> { self.parse_binary_left_associative(&[TokenKind::Or], Self::and) }

    pub(crate) fn and(&mut self) -> Result<Expr, ParseError> { self.parse_binary_left_associative(&[TokenKind::And], Self::equality) }

    pub(crate) fn equality(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_left_associative(&[TokenKind::BangEqual, TokenKind::EqualEqual], Self::comparison)
    }

    pub(crate) fn comparison(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_left_associative(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual], Self::term)
    }

    pub(crate) fn term(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_left_associative(&[TokenKind::Minus, TokenKind::Plus], Self::factor)
    }

    pub(crate) fn factor(&mut self) -> Result<Expr, ParseError> {
        self.parse_binary_left_associative(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent, TokenKind::IntegerDivision], Self::power)
    }

    pub(crate) fn power(&mut self) -> Result<Expr, ParseError> {
        let expr = self.unary()?;

        // Right-associative: 2 ** 3 ** 2 = 2 ** (3 ** 2) = 512
        if self.match_any(&[TokenKind::Power]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.power()?; // Recursive for right-associativity

            let left_span = self.get_expr_span(expr.id());
            let right_span = self.get_expr_span(right.id());
            let span = self.merge_spans(left_span, right_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Binary {
                id,
                left: Box::new(expr),
                operator: operator_kind,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    pub(crate) fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_any(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator_kind = self.previous().kind.clone();
            let operator_span = self.previous().span;
            let operand = self.unary()?;

            let operand_span = self.get_expr_span(operand.id());
            let span = self.merge_spans(operator_span, operand_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Unary {
                id,
                operator: operator_kind,
                operand: Box::new(operand),
            });
        }

        // Pre-increment: ++x
        if self.match_any(&[TokenKind::Increment]) {
            let operator_span = self.previous().span;
            let operand = self.unary()?;

            let operand_span = self.get_expr_span(operand.id());
            let span = self.merge_spans(operator_span, operand_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::PreIncrement {
                id,
                operand: Box::new(operand),
            });
        }

        // Pre-decrement: --x
        if self.match_any(&[TokenKind::Decrement]) {
            let operator_span = self.previous().span;
            let operand = self.unary()?;

            let operand_span = self.get_expr_span(operand.id());
            let span = self.merge_spans(operator_span, operand_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::PreDecrement {
                id,
                operand: Box::new(operand),
            });
        }

        self.call()
    }

    pub(crate) fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_any(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_any(&[TokenKind::LeftBracket]) {
                let index = self.expression()?;
                self.consume(&TokenKind::RightBracket, "Expected ']' after index")?;

                let object_span = self.get_expr_span(expr.id());
                let end_span = self.previous_span();
                let span = self.merge_spans(object_span, end_span);
                let id = self.span_map.alloc_id();
                self.span_map.insert(id, span);

                expr = Expr::Index {
                    id,
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_any(&[TokenKind::Dot]) {
                // Allow keywords as property/method names after dot
                let mut name = self.consume_identifier_or_keyword("Expected property name after '.'")?;

                // Check for ! suffix (mutating method indicator)
                if self.check(&TokenKind::Bang) && self.peek_next_kind() == Some(TokenKind::LeftParen) {
                    self.advance(); // consume !
                    name.push('!');
                }

                // Check for ? suffix (query method indicator)
                if self.check(&TokenKind::Question) && self.peek_next_kind() == Some(TokenKind::LeftParen) {
                    self.advance(); // consume ?
                    name.push('?');
                }

                let object_span = self.get_expr_span(expr.id());
                let end_span = self.previous_span();
                let span = self.merge_spans(object_span, end_span);
                let id = self.span_map.alloc_id();
                self.span_map.insert(id, span);

                expr = Expr::Get {
                    id,
                    object: Box::new(expr),
                    name,
                };
            } else if self.match_any(&[TokenKind::Increment]) {
                // Post-increment: x++
                let expr_span = self.get_expr_span(expr.id());
                let operator_span = self.previous().span;
                let span = self.merge_spans(expr_span, operator_span);
                let id = self.span_map.alloc_id();
                self.span_map.insert(id, span);

                expr = Expr::PostIncrement {
                    id,
                    operand: Box::new(expr),
                };
            } else if self.match_any(&[TokenKind::Decrement]) {
                // Post-decrement: x--
                let expr_span = self.get_expr_span(expr.id());
                let operator_span = self.previous().span;
                let span = self.merge_spans(expr_span, operator_span);
                let id = self.span_map.alloc_id();
                self.span_map.insert(id, span);

                expr = Expr::PostDecrement {
                    id,
                    operand: Box::new(expr),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    pub(crate) fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();

        self.skip_newlines();

        if !self.check(&TokenKind::RightParen) {
            loop {
                if self.check(&TokenKind::RightParen) {
                    break;
                }

                arguments.push(self.expression()?);

                if !self.match_any_with_newlines(&[TokenKind::Comma]) {
                    break;
                }
                // Handle trailing comma - if we see the closing paren after comma, break
                if self.check(&TokenKind::RightParen) {
                    break;
                }
            }
        }

        self.consume_closing(&TokenKind::RightParen, "Expected ')' after arguments")?;

        let callee_span = self.get_expr_span(callee.id());
        let end_span = self.previous_span();
        let span = self.merge_spans(callee_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Expr::Call {
            id,
            data: Box::new(CallData {
                callee,
                arguments,
            }),
        })
    }
}
