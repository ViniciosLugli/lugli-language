use crate::{Parser, error::ParseError};
use lugli_ast::{Expr, IfData, Stmt, StructDeclData};
use lugli_lexer::TokenKind;

impl<'a> Parser<'a> {
    pub(crate) fn statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Let) | Some(TokenKind::Const) | Some(TokenKind::Mut) => self.var_declaration(),
            Some(TokenKind::Fn) => self.function_declaration(),
            Some(TokenKind::Struct) => self.struct_declaration(),
            Some(TokenKind::Impl) => self.impl_block(),
            Some(TokenKind::Import) => self.import_statement(),
            Some(TokenKind::From) => self.from_import_statement(),
            Some(TokenKind::Export) => self.export_statement(),
            Some(TokenKind::If) => self.if_statement(),
            Some(TokenKind::While) => self.while_statement(),
            Some(TokenKind::For) => self.for_statement(),
            Some(TokenKind::Loop) => self.loop_statement(),
            Some(TokenKind::Return) => self.return_statement(),
            Some(TokenKind::Break) => self.break_statement(),
            Some(TokenKind::Continue) => self.continue_statement(),
            Some(TokenKind::LeftBrace) => {
                if self.is_dict_literal() {
                    self.expression_statement()
                } else {
                    self.block_statement()
                }
            }
            _ => self.expression_statement(),
        }
    }

    pub(crate) fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        let is_const = self.match_any(&[TokenKind::Const]);
        let mut _is_mutable = false;

        if !is_const {
            if self.match_any(&[TokenKind::Mut]) {
                _is_mutable = true;
            } else {
                self.consume(&TokenKind::Let, "Expected 'let', 'mut', or 'const'")?;
                _is_mutable = self.match_any(&[TokenKind::Mut]);
            }
        }

        let name = self.consume_identifier("Expected variable name")?;

        // Parse optional type hint
        let type_hint = if self.match_any(&[TokenKind::Colon]) { Some(self.parse_type_hint()?) } else { None };

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

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::VarDecl {
            id,
            name,
            type_hint,
            initializer,
            is_const,
        })
    }

    pub(crate) fn function_declaration(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Fn, "Expected 'fn'")?;

        // Allow keywords as method names in struct definitions
        let mut name = self.consume_identifier_or_keyword("Expected function name")?;

        if self.match_any(&[TokenKind::Bang]) {
            name.push('!');
        }

        if self.match_any(&[TokenKind::Question]) {
            name.push('?');
        }

        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        self.skip_newlines();

        let parsed_params = self.parse_params()?;
        let params: Vec<(String, Option<lugli_ast::TypeHint>)> = parsed_params.iter().map(|p| (p.name.clone(), p.type_hint.clone())).collect();

        self.skip_newlines();
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse optional return type hint
        let return_type = if self.match_any(&[TokenKind::Arrow]) { Some(self.parse_type_hint()?) } else { None };

        let body = self.block_body()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::FnDecl {
            id,
            name,
            params,
            return_type,
            body,
        })
    }

    pub(crate) fn struct_declaration(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Struct, "Expected 'struct'")?;

        let name = self.consume_identifier("Expected struct name")?;

        self.consume_with_newlines(&TokenKind::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
            self.skip_newlines();

            if self.check(&TokenKind::Fn) {
                methods.push(self.function_declaration()?);
            } else if !self.check(&TokenKind::RightBrace) {
                let field_name = self.consume_identifier("Expected field name")?;

                // Parse optional type hint and default value
                let (type_hint, default_value) = if self.check(&TokenKind::Colon) {
                    self.advance(); // consume colon
                    let is_type_hint = matches!(self.peek_kind(), Some(TokenKind::Identifier(_)));
                    if is_type_hint {
                        let hint = Some(self.parse_type_hint()?);
                        let default = if self.match_any(&[TokenKind::Equal]) { Some(self.expression()?) } else { None };
                        (hint, default)
                    } else {
                        (None, Some(self.expression()?))
                    }
                } else if self.match_any(&[TokenKind::Equal]) {
                    (None, Some(self.expression()?))
                } else {
                    (None, None)
                };

                fields.push(lugli_ast::StructField {
                    name: field_name,
                    type_hint,
                    default: default_value,
                });

                self.match_any(&[TokenKind::Comma, TokenKind::Newline]);
            }
        }

        self.consume_closing(&TokenKind::RightBrace, "Expected '}' after struct definition")?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::StructDecl {
            id,
            data: Box::new(StructDeclData {
                name,
                fields,
                methods,
            }),
        })
    }

    pub(crate) fn impl_block(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Impl, "Expected 'impl'")?;

        let struct_name = self.consume_identifier("Expected struct name")?;

        self.consume_with_newlines(&TokenKind::LeftBrace, "Expected '{' after struct name")?;

        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.scanner.is_at_end() {
            self.skip_newlines();

            if self.check(&TokenKind::Fn) && !self.scanner.is_at_end() {
                methods.push(self.function_declaration()?);
            } else if !self.check(&TokenKind::RightBrace) {
                return Err(ParseError::Custom {
                    message: "Expected method declaration in impl block".to_string(),
                    span: self.current_span(),
                });
            }
        }

        self.consume_closing(&TokenKind::RightBrace, "Expected '}' after impl block")?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::StructDecl {
            id,
            data: Box::new(StructDeclData {
                name: struct_name,
                fields: Vec::new(),
                methods,
            }),
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

        let else_branch = if self.match_any(&[TokenKind::Else]) { Some(self.block_body()?) } else { None };

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::If {
            id,
            data: Box::new(IfData {
                condition,
                then_branch,
                elif_branches,
                else_branch,
            }),
        })
    }

    pub(crate) fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::While, "Expected 'while'")?;

        let condition = self.expression()?;
        let body = self.block_body()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::While {
            id,
            condition,
            body,
        })
    }

    pub(crate) fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::For, "Expected 'for'")?;

        let variable = self.consume_identifier("Expected variable name")?;
        self.consume(&TokenKind::In, "Expected 'in' after for variable")?;

        let iterable = self.expression()?;
        let body = self.block_body()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::For {
            id,
            variable,
            iterable,
            body,
        })
    }

    pub(crate) fn loop_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Loop, "Expected 'loop'")?;

        let body = self.block_body()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Loop {
            id,
            body,
        })
    }

    pub(crate) fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Return, "Expected 'return'")?;

        let value = if self.check(&TokenKind::Newline) || self.check(&TokenKind::Semicolon) { None } else { Some(self.expression()?) };

        self.consume_statement_terminator()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Return {
            id,
            value,
        })
    }

    pub(crate) fn break_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Break, "Expected 'break'")?;
        self.consume_statement_terminator()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Break {
            id,
        })
    }

    pub(crate) fn continue_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Continue, "Expected 'continue'")?;
        self.consume_statement_terminator()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Continue {
            id,
        })
    }

    pub(crate) fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        let statements = self.block_body()?;
        let end_span = self.previous_span();

        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Block {
            id,
            statements,
        })
    }

    pub(crate) fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        if self.match_any(&[
            TokenKind::Equal,
            TokenKind::PlusEqual,
            TokenKind::MinusEqual,
            TokenKind::StarEqual,
            TokenKind::SlashEqual,
            TokenKind::PercentEqual,
        ]) {
            let operator = self.previous().clone();
            let value = self.expression()?;
            self.consume_statement_terminator()?;

            let final_value = if operator.kind != TokenKind::Equal {
                let binary_op = match operator.kind {
                    TokenKind::PlusEqual => TokenKind::Plus,
                    TokenKind::MinusEqual => TokenKind::Minus,
                    TokenKind::StarEqual => TokenKind::Star,
                    TokenKind::SlashEqual => TokenKind::Slash,
                    TokenKind::PercentEqual => TokenKind::Percent,
                    _ => {
                        return Err(ParseError::Custom {
                            message: format!("Unexpected operator in assignment: {:?}. Expected one of +=, -=, *=, /=, %=", operator.kind),
                            span: operator.span,
                        });
                    }
                };

                let expr_span = self.get_expr_span(expr.id());
                let value_span = self.get_expr_span(value.id());
                let bin_span = self.merge_spans(expr_span, value_span);
                let bin_id = self.span_map.alloc_id();
                self.span_map.insert(bin_id, bin_span);

                Expr::Binary {
                    id: bin_id,
                    left: Box::new(expr.clone()),
                    operator: binary_op,
                    right: Box::new(value),
                }
            } else {
                value
            };

            let final_value_span = self.get_expr_span(final_value.id());

            let assignment_expr = match expr.clone() {
                Expr::Identifier {
                    name,
                    id,
                } => {
                    let id_span = self.get_expr_span(id);
                    let span = self.merge_spans(id_span, final_value_span);
                    let set_id = self.span_map.alloc_id();
                    self.span_map.insert(set_id, span);

                    let global_id = self.span_map.alloc_id();
                    self.span_map.insert(global_id, id_span);

                    Expr::Set {
                        id: set_id,
                        object: Box::new(Expr::Identifier {
                            id: global_id,
                            name: "global".to_string(),
                        }),
                        name,
                        value: Box::new(final_value),
                    }
                }
                Expr::Get {
                    object,
                    name,
                    id,
                } => {
                    let get_span = self.get_expr_span(id);
                    let span = self.merge_spans(get_span, final_value_span);
                    let set_id = self.span_map.alloc_id();
                    self.span_map.insert(set_id, span);

                    Expr::Set {
                        id: set_id,
                        object,
                        name,
                        value: Box::new(final_value),
                    }
                }
                Expr::Index {
                    object,
                    index,
                    id,
                } => {
                    let index_span = self.get_expr_span(id);
                    let span = self.merge_spans(index_span, final_value_span);
                    let set_id = self.span_map.alloc_id();
                    self.span_map.insert(set_id, span);

                    let list_id = self.span_map.alloc_id();
                    self.span_map.insert(list_id, span);

                    Expr::Set {
                        id: set_id,
                        object,
                        name: "[index]".to_string(),
                        value: Box::new(Expr::List {
                            id: list_id,
                            elements: vec![(*index).clone(), final_value.clone()],
                        }),
                    }
                }
                _ => {
                    let expr_span = self.get_expr_span(expr.id());
                    return Err(ParseError::Custom {
                        message: "Invalid assignment target".to_string(),
                        span: expr_span,
                    });
                }
            };

            let assignment_span = self.get_expr_span(assignment_expr.id());
            let stmt_id = self.span_map.alloc_id();
            self.span_map.insert(stmt_id, assignment_span);

            Ok(Stmt::Expression {
                id: stmt_id,
                expr: assignment_expr.clone(),
            })
        } else {
            self.consume_statement_terminator()?;
            let expr_span = self.get_expr_span(expr.id());
            let stmt_id = self.span_map.alloc_id();
            self.span_map.insert(stmt_id, expr_span);

            Ok(Stmt::Expression {
                id: stmt_id,
                expr,
            })
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

    pub(crate) fn import_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Import, "Expected 'import'")?;

        let mut module_path = vec![self.consume_identifier("Expected module name")?];

        while self.match_any(&[TokenKind::Dot]) {
            module_path.push(self.consume_identifier("Expected module component")?);
        }

        let alias = if self.match_any(&[TokenKind::As]) { Some(self.consume_identifier("Expected alias name")?) } else { None };

        self.consume_statement_terminator()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Import {
            id,
            module_path,
            items: None,
            alias,
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn from_import_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::From, "Expected 'from'")?;

        let mut module_path = vec![self.consume_identifier("Expected module name")?];

        while self.match_any(&[TokenKind::Dot]) {
            module_path.push(self.consume_identifier("Expected module component")?);
        }

        self.consume(&TokenKind::Import, "Expected 'import' after module path")?;

        let items = if self.match_any(&[TokenKind::Star]) {
            None
        } else {
            let mut import_items = vec![self.consume_identifier("Expected item name")?];

            while self.match_any(&[TokenKind::Comma]) {
                self.skip_newlines();
                if self.scanner.is_at_end() || self.check(&TokenKind::Newline) {
                    break;
                }
                import_items.push(self.consume_identifier("Expected item name")?);
            }

            Some(import_items)
        };

        self.consume_statement_terminator()?;

        let end_span = self.previous_span();
        let span = self.merge_spans(start_span, end_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Import {
            id,
            module_path,
            items,
            alias: None,
        })
    }

    pub(crate) fn export_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.consume(&TokenKind::Export, "Expected 'export'")?;

        let item = Box::new(self.statement()?);

        let item_span = self.get_expr_span(item.id());
        let span = self.merge_spans(start_span, item_span);
        let id = self.span_map.alloc_id();
        self.span_map.insert(id, span);

        Ok(Stmt::Export {
            id,
            item,
        })
    }
}
