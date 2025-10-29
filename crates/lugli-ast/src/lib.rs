use lugli_common::Span;

pub mod expr;
pub mod stmt;
pub mod visitor;

pub use expr::{Expr, FStringPart, LiteralValue, MatchArm, Pattern};
pub use stmt::Stmt;
pub use visitor::{Visitor, VisitorMut};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

impl Program {
    pub fn new(statements: Vec<Stmt>, span: Span) -> Self {
        Self {
            statements,
            span,
        }
    }
}

pub trait AstNode {
    fn span(&self) -> &Span;
}

impl AstNode for Program {
    fn span(&self) -> &Span { &self.span }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_creation() {
        let span = Span {
            start: 0,
            end: 10,
        };
        let program = Program::new(vec![], span.clone());

        assert_eq!(program.statements.len(), 0);
        assert_eq!(program.span(), &span);
    }

    #[test]
    fn test_literal_expressions() {
        let span = Span {
            start: 0,
            end: 5,
        };

        // Test number literal
        let number_expr = Expr::Literal {
            value: LiteralValue::Number(42.0),
            span: span.clone(),
        };
        assert_eq!(number_expr.span(), &span);

        // Test string literal
        let string_expr = Expr::Literal {
            value: LiteralValue::String("hello".to_string()),
            span: span.clone(),
        };
        assert_eq!(string_expr.span(), &span);

        // Test boolean literal
        let bool_expr = Expr::Literal {
            value: LiteralValue::Boolean(true),
            span: span.clone(),
        };
        assert_eq!(bool_expr.span(), &span);

        // Test null literal
        let null_expr = Expr::Literal {
            value: LiteralValue::Null,
            span: span.clone(),
        };
        assert_eq!(null_expr.span(), &span);
    }

    #[test]
    fn test_identifier_expression() {
        let span = Span {
            start: 0,
            end: 5,
        };
        let ident_expr = Expr::Identifier {
            name: "variable".to_string(),
            span: span.clone(),
        };

        assert_eq!(ident_expr.span(), &span);
        if let Expr::Identifier {
            name, ..
        } = ident_expr
        {
            assert_eq!(name, "variable");
        } else {
            panic!("Expected Identifier expression");
        }
    }

    #[test]
    fn test_binary_expression() {
        let span = Span {
            start: 0,
            end: 5,
        };
        let left = Box::new(Expr::Literal {
            value: LiteralValue::Number(1.0),
            span: span.clone(),
        });
        let right = Box::new(Expr::Literal {
            value: LiteralValue::Number(2.0),
            span: span.clone(),
        });

        // Create a mock token for the operator
        let operator = lugli_lexer::Token::new(lugli_lexer::TokenKind::Plus, span.clone());

        let binary_expr = Expr::Binary {
            left,
            operator,
            right,
            span: span.clone(),
        };

        assert_eq!(binary_expr.span(), &span);
    }

    #[test]
    fn test_function_call_expression() {
        let span = Span {
            start: 0,
            end: 10,
        };
        let callee = Box::new(Expr::Identifier {
            name: "function".to_string(),
            span: span.clone(),
        });
        let arguments = vec![
            Expr::Literal {
                value: LiteralValue::Number(1.0),
                span: span.clone(),
            },
            Expr::Literal {
                value: LiteralValue::String("hello".to_string()),
                span: span.clone(),
            },
        ];

        let call_expr = Expr::Call {
            callee,
            arguments,
            span: span.clone(),
        };

        assert_eq!(call_expr.span(), &span);
        if let Expr::Call {
            arguments, ..
        } = call_expr
        {
            assert_eq!(arguments.len(), 2);
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_list_expression() {
        let span = Span {
            start: 0,
            end: 10,
        };
        let elements = vec![
            Expr::Literal {
                value: LiteralValue::Number(1.0),
                span: span.clone(),
            },
            Expr::Literal {
                value: LiteralValue::Number(2.0),
                span: span.clone(),
            },
            Expr::Literal {
                value: LiteralValue::Number(3.0),
                span: span.clone(),
            },
        ];

        let list_expr = Expr::List {
            elements,
            span: span.clone(),
        };

        assert_eq!(list_expr.span(), &span);
        if let Expr::List {
            elements, ..
        } = list_expr
        {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected List expression");
        }
    }

    #[test]
    fn test_dict_expression() {
        let span = Span {
            start: 0,
            end: 15,
        };
        let pairs = vec![
            (
                Expr::Literal {
                    value: LiteralValue::String("key1".to_string()),
                    span: span.clone(),
                },
                Expr::Literal {
                    value: LiteralValue::Number(1.0),
                    span: span.clone(),
                },
            ),
            (
                Expr::Literal {
                    value: LiteralValue::String("key2".to_string()),
                    span: span.clone(),
                },
                Expr::Literal {
                    value: LiteralValue::String("value2".to_string()),
                    span: span.clone(),
                },
            ),
        ];

        let dict_expr = Expr::Dict {
            pairs,
            span: span.clone(),
        };

        assert_eq!(dict_expr.span(), &span);
        if let Expr::Dict {
            pairs, ..
        } = dict_expr
        {
            assert_eq!(pairs.len(), 2);
        } else {
            panic!("Expected Dict expression");
        }
    }

    #[test]
    fn test_var_declaration_statement() {
        let span = Span {
            start: 0,
            end: 10,
        };
        let initializer = Some(Expr::Literal {
            value: LiteralValue::Number(42.0),
            span: span.clone(),
        });

        let var_decl = Stmt::VarDecl {
            name: "x".to_string(),
            initializer,
            is_const: false,
            span: span.clone(),
        };

        assert_eq!(var_decl.span(), &span);
        if let Stmt::VarDecl {
            name,
            is_const,
            ..
        } = var_decl
        {
            assert_eq!(name, "x");
            assert_eq!(is_const, false);
        } else {
            panic!("Expected VarDecl statement");
        }
    }

    #[test]
    fn test_function_declaration_statement() {
        let span = Span {
            start: 0,
            end: 20,
        };
        let params = vec!["x".to_string(), "y".to_string()];
        let body = vec![Stmt::Return {
            value: Some(Expr::Identifier {
                name: "x".to_string(),
                span: span.clone(),
            }),
            span: span.clone(),
        }];

        let fn_decl = Stmt::FnDecl {
            name: "add".to_string(),
            params,
            body,
            span: span.clone(),
        };

        assert_eq!(fn_decl.span(), &span);
        if let Stmt::FnDecl {
            name,
            params,
            body,
            ..
        } = fn_decl
        {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected FnDecl statement");
        }
    }

    #[test]
    fn test_if_statement() {
        let span = Span {
            start: 0,
            end: 30,
        };
        let condition = Expr::Literal {
            value: LiteralValue::Boolean(true),
            span: span.clone(),
        };
        let then_branch = vec![Stmt::Expression {
            expr: Expr::Literal {
                value: LiteralValue::String("then".to_string()),
                span: span.clone(),
            },
            span: span.clone(),
        }];
        let else_branch = Some(vec![Stmt::Expression {
            expr: Expr::Literal {
                value: LiteralValue::String("else".to_string()),
                span: span.clone(),
            },
            span: span.clone(),
        }]);

        let if_stmt = Stmt::If {
            condition,
            then_branch,
            elif_branches: vec![],
            else_branch,
            span: span.clone(),
        };

        assert_eq!(if_stmt.span(), &span);
        if let Stmt::If {
            then_branch,
            else_branch,
            ..
        } = if_stmt
        {
            assert_eq!(then_branch.len(), 1);
            assert!(else_branch.is_some());
            assert_eq!(else_branch.unwrap().len(), 1);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_visitor_pattern() {
        struct TestVisitor {
            expr_count: usize,
            stmt_count: usize,
        }

        impl Visitor<()> for TestVisitor {
            fn visit_expr(&mut self, expr: &Expr) -> () {
                self.expr_count += 1;
                self.walk_expr(expr)
            }

            fn visit_stmt(&mut self, stmt: &Stmt) -> () {
                self.stmt_count += 1;
                self.walk_stmt(stmt)
            }
        }

        let span = Span {
            start: 0,
            end: 10,
        };
        let program = Program::new(
            vec![
                Stmt::VarDecl {
                    name: "x".to_string(),
                    initializer: Some(Expr::Literal {
                        value: LiteralValue::Number(42.0),
                        span: span.clone(),
                    }),
                    is_const: false,
                    span: span.clone(),
                },
                Stmt::Expression {
                    expr: Expr::Binary {
                        left: Box::new(Expr::Identifier {
                            name: "x".to_string(),
                            span: span.clone(),
                        }),
                        operator: lugli_lexer::Token::new(lugli_lexer::TokenKind::Plus, span.clone()),
                        right: Box::new(Expr::Literal {
                            value: LiteralValue::Number(1.0),
                            span: span.clone(),
                        }),
                        span: span.clone(),
                    },
                    span: span.clone(),
                },
            ],
            span,
        );

        let mut visitor = TestVisitor {
            expr_count: 0,
            stmt_count: 0,
        };

        visitor.visit_program(&program);

        // Should visit 2 statements and 4 expressions (42.0, x, 1.0, binary)
        assert_eq!(visitor.stmt_count, 2);
        assert_eq!(visitor.expr_count, 4);
    }
}
