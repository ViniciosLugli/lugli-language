use lugli_lexer::TokenKind;
use lugli_ast::{Stmt, AstNode, Expr};
use lugli_common::Span;
use crate::error::ParseError;
use crate::Parser;

impl<'a> Parser<'a> {
    pub(crate) fn statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Let) | Some(TokenKind::Mut) | Some(TokenKind::Const) => {
                self.var_declaration()
            },
            Some(TokenKind::Fn) => {
                self.function_declaration()
            },
            Some(TokenKind::If) => {
                self.if_statement()
            },
            Some(TokenKind::While) => {
                self.while_statement()
            },
            Some(TokenKind::For) => {
                self.for_statement()
            },
            Some(TokenKind::Loop) => {
                self.loop_statement()
            },
            Some(TokenKind::Return) => {
                self.return_statement()
            },
            Some(TokenKind::Break) => {
                self.break_statement()
            },
            Some(TokenKind::Continue) => {
                self.continue_statement()
            },
            Some(TokenKind::LeftBrace) => {
                if self.is_dict_literal() {
                    self.expression_statement()
                } else {
                    self.block_statement()
                }
            },
            _ => self.expression_statement(),
        }
    }

    pub(crate) fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let is_const = self.match_any(&[TokenKind::Const]);
        let _is_mutable = self.match_any(&[TokenKind::Mut]);

        if !is_const && !_is_mutable {
            self.consume(&TokenKind::Let, "Expected 'let', 'mut', or 'const'")?;
        }

        let name = self.consume_identifier("Expected variable name")?;

        let initializer = if self.match_any(&[TokenKind::Equal]) {
            Some(self.expression()?)
        } else if is_const {
            return Err(ParseError::Custom {
                message: "Const variables must be initialized".to_string(),
                span: self.current_span(),
            });
        } else {
            None
        };

        self.consume_statement_terminator()?;

        let span = Span {
            start: self.previous_span().start,
            end: initializer.as_ref()
                .map(|init| init.span().end)
                .unwrap_or_else(|| self.previous_span().end),
        };

        Ok(Stmt::VarDecl {
            name,
            initializer,
            is_const,
            span,
        })
    }

    pub(crate) fn function_declaration(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Fn, "Expected 'fn'")?;

        let name = self.consume_identifier("Expected function name")?;

        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        self.skip_newlines();

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;
                params.push(param_name);

                if !self.match_any(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_newlines();
            }
        }

        self.skip_newlines();
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;

        let body = self.block_body()?;

        let end_span = self.previous_span();

        Ok(Stmt::FnDecl {
            name,
            params,
            body,
            span: self.merge_spans(start_span, end_span),
        })
    }

    pub(crate) fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::If, "Expected 'if'")?;

        let condition = self.expression()?;
        let then_branch = self.block_body()?;

        let mut elif_branches = Vec::new();
        while self.match_any(&[TokenKind::Elif]) {
            let elif_condition = self.expression()?;
            let elif_body = self.block_body()?;
            elif_branches.push((elif_condition, elif_body));
        }

        let else_branch = if self.match_any(&[TokenKind::Else]) {
            Some(self.block_body()?)
        } else {
            None
        };

        let end_span = else_branch.as_ref()
            .and_then(|stmts| stmts.last())
            .map(|stmt| stmt.span().end)
            .or_else(|| {
                elif_branches.last()
                    .and_then(|(_, stmts)| stmts.last())
                    .map(|stmt| stmt.span().end)
            })
            .unwrap_or_else(|| {
                then_branch.last()
                    .map(|stmt| stmt.span().end)
                    .unwrap_or(start_span.end)
            });

        Ok(Stmt::If {
            condition,
            then_branch,
            elif_branches,
            else_branch,
            span: Span {
                start: start_span.start,
                end: end_span,
            },
        })
    }

    pub(crate) fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::While, "Expected 'while'")?;

        let condition = self.expression()?;
        let body = self.block_body()?;

        let end_span = body.last()
            .map(|stmt| stmt.span().end)
            .unwrap_or(start_span.end);

        Ok(Stmt::While {
            condition,
            body,
            span: Span {
                start: start_span.start,
                end: end_span,
            },
        })
    }

    pub(crate) fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::For, "Expected 'for'")?;

        let variable = self.consume_identifier("Expected variable name")?;
        self.consume(&TokenKind::In, "Expected 'in' after for variable")?;

        let iterable = self.expression()?;
        let body = self.block_body()?;

        let end_span = body.last()
            .map(|stmt| stmt.span().end)
            .unwrap_or(start_span.end);

        Ok(Stmt::For {
            variable,
            iterable,
            body,
            span: Span {
                start: start_span.start,
                end: end_span,
            },
        })
    }

    pub(crate) fn loop_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Loop, "Expected 'loop'")?;

        let body = self.block_body()?;

        let end_span = body.last()
            .map(|stmt| stmt.span().end)
            .unwrap_or(start_span.end);

        Ok(Stmt::Loop {
            body,
            span: Span {
                start: start_span.start,
                end: end_span,
            },
        })
    }

    pub(crate) fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Return, "Expected 'return'")?;

        let value = if self.check(&TokenKind::Newline) || self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume_statement_terminator()?;

        let end_span = value.as_ref()
            .map(|expr| expr.span().end)
            .unwrap_or_else(|| self.previous_span().end);

        Ok(Stmt::Return {
            value,
            span: Span {
                start: start_span.start,
                end: end_span,
            },
        })
    }

    pub(crate) fn break_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Break, "Expected 'break'")?;
        self.consume_statement_terminator()?;

        Ok(Stmt::Break {
            span: Span {
                start: start_span.start,
                end: self.previous_span().end,
            },
        })
    }

    pub(crate) fn continue_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Continue, "Expected 'continue'")?;
        self.consume_statement_terminator()?;

        Ok(Stmt::Continue {
            span: Span {
                start: start_span.start,
                end: self.previous_span().end,
            },
        })
    }

    pub(crate) fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        let statements = self.block_body()?;
        let end_span = self.previous_span();

        Ok(Stmt::Block {
            statements,
            span: self.merge_spans(start_span, end_span),
        })
    }

    pub(crate) fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        if self.match_any(&[TokenKind::Equal, TokenKind::PlusEqual, TokenKind::MinusEqual,
                            TokenKind::StarEqual, TokenKind::SlashEqual, TokenKind::PercentEqual]) {
            let operator = self.previous().clone();
            let value = self.expression()?;
            self.consume_statement_terminator()?;

            // For compound assignments, convert to binary operation
            let final_value = if operator.kind != TokenKind::Equal {
                let binary_op = match operator.kind {
                    TokenKind::PlusEqual => TokenKind::Plus,
                    TokenKind::MinusEqual => TokenKind::Minus,
                    TokenKind::StarEqual => TokenKind::Star,
                    TokenKind::SlashEqual => TokenKind::Slash,
                    TokenKind::PercentEqual => TokenKind::Percent,
                    _ => unreachable!(),
                };

                let binary_operator = lugli_lexer::Token::new(binary_op, "".to_string(), operator.span);
                let span = self.merge_spans(*expr.span(), *value.span());
                Expr::Binary {
                    left: Box::new(expr.clone()),
                    operator: binary_operator,
                    right: Box::new(value),
                    span,
                }
            } else {
                value
            };

            let assignment_expr = match expr {
                Expr::Identifier { name, span: id_span } => {
                    let span = self.merge_spans(id_span, *final_value.span());
                    Expr::Set {
                        object: Box::new(Expr::Identifier { name: "global".to_string(), span: id_span }),
                        name,
                        value: Box::new(final_value),
                        span,
                    }
                },
                Expr::Get { object, name, span: get_span } => {
                    let span = self.merge_spans(get_span, *final_value.span());
                    Expr::Set {
                        object,
                        name,
                        value: Box::new(final_value),
                        span,
                    }
                },
                Expr::Index { object, index, span: index_span } => {
                    let span = self.merge_spans(index_span, *final_value.span());
                    Expr::Set {
                        object,
                        name: format!("[{}]", "index"), // Use a marker for index assignments
                        value: Box::new(Expr::List {
                            elements: vec![*index, final_value],
                            span,
                        }),
                        span,
                    }
                },
                _ => {
                    return Err(ParseError::Custom {
                        message: "Invalid assignment target".to_string(),
                        span: *expr.span(),
                    });
                }
            };

            let span = *assignment_expr.span();
            Ok(Stmt::Expression { expr: assignment_expr, span })
        } else {
            self.consume_statement_terminator()?;
            let span = *expr.span();
            Ok(Stmt::Expression { expr, span })
        }
    }

    pub(crate) fn block_body(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.consume_with_newlines(&TokenKind::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
            self.skip_newlines();

            if !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
                statements.push(self.statement()?);
            }
        }

        self.consume_closing(&TokenKind::RightBrace, "Expected '}'")?;

        Ok(statements)
    }
}