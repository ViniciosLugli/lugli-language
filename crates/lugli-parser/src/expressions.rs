use crate::{Parser, error::ParseError};
use lugli_ast::{AstNode, Expr};
use lugli_lexer::TokenKind;

impl<'a> Parser<'a> {
    pub(crate) fn expression(&mut self) -> Result<Expr, ParseError> { self.or() }

    pub(crate) fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while self.match_any(&[TokenKind::Or]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.and()?;
            let span = self.merge_spans(*expr.span(), *right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    pub(crate) fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.match_any(&[TokenKind::And]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.equality()?;
            let span = self.merge_spans(*expr.span(), *right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    pub(crate) fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.match_any(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.comparison()?;
            let span = self.merge_spans(*expr.span(), *right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    pub(crate) fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.match_any(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.term()?;
            let span = self.merge_spans(*expr.span(), *right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    pub(crate) fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_any(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.factor()?;
            let span = self.merge_spans(*expr.span(), *right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    pub(crate) fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.power()?;

        while self.match_any(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent, TokenKind::IntegerDivision]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.power()?;
            let span = self.merge_spans(*expr.span(), *right.span());
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(expr)
    }

    pub(crate) fn power(&mut self) -> Result<Expr, ParseError> {
        let expr = self.unary()?;

        // Right-associative: 2 ** 3 ** 2 = 2 ** (3 ** 2) = 512
        if self.match_any(&[TokenKind::Power]) {
            let operator = self.previous().clone();
            self.skip_newlines();
            let right = self.power()?; // Recursive for right-associativity
            let span = self.merge_spans(*expr.span(), *right.span());
            return Ok(Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    pub(crate) fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_any(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let operand = self.unary()?;
            let span = self.merge_spans(operator.span, *operand.span());
            return Ok(Expr::Unary {
                operator,
                operand: Box::new(operand),
                span,
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
                let span = self.merge_spans(*expr.span(), self.previous_span());
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.match_any(&[TokenKind::Dot]) {
                let mut name = self.consume_identifier("Expected property name after '.'")?;

                // Check for ! suffix (mutating method indicator)
                if self.check(&TokenKind::Bang) && self.peek_next_kind() == Some(TokenKind::LeftParen) {
                    self.advance(); // consume !
                    name.push('!');
                }

                let span = self.merge_spans(*expr.span(), self.previous_span());
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                    span,
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

        let span = self.merge_spans(*callee.span(), self.previous_span());

        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
            span,
        })
    }
}
