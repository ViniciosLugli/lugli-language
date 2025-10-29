use crate::{Parser, error::ParseError};
use lugli_ast::{CallData, Expr};
use lugli_lexer::TokenKind;

impl<'a> Parser<'a> {
    pub(crate) fn expression(&mut self) -> Result<Expr, ParseError> { self.or() }

    pub(crate) fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_any(&[TokenKind::Or]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.and()?;

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

    pub(crate) fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_any(&[TokenKind::And]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.equality()?;

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

    pub(crate) fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_any(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.comparison()?;

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

    pub(crate) fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_any(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.term()?;

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

    pub(crate) fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_any(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.factor()?;

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

    pub(crate) fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.power()?;

        while self.match_any(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent, TokenKind::IntegerDivision]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.power()?;

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

    pub(crate) fn power(&mut self) -> Result<Expr, ParseError> {
        let expr = self.unary()?;

        // Right-associative: 2 ** 3 ** 2 = 2 ** (3 ** 2) = 512
        if self.match_any(&[TokenKind::Power]) {
            let operator_kind = self.previous().kind.clone();
            self.skip_newlines();
            let right = self.power()?; // Recursive for right-associativity

            let left_span = self.span_map.get(expr.id()).unwrap();
            let right_span = self.span_map.get(right.id()).unwrap();
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

            let operand_span = self.span_map.get(operand.id()).unwrap();
            let span = self.merge_spans(operator_span, operand_span);
            let id = self.span_map.alloc_id();
            self.span_map.insert(id, span);

            return Ok(Expr::Unary {
                id,
                operator: operator_kind,
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

                let object_span = self.span_map.get(expr.id()).unwrap();
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
                let mut name = self.consume_identifier("Expected property name after '.'")?;

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

                let object_span = self.span_map.get(expr.id()).unwrap();
                let end_span = self.previous_span();
                let span = self.merge_spans(object_span, end_span);
                let id = self.span_map.alloc_id();
                self.span_map.insert(id, span);

                expr = Expr::Get {
                    id,
                    object: Box::new(expr),
                    name,
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

        let callee_span = self.span_map.get(callee.id()).unwrap();
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
