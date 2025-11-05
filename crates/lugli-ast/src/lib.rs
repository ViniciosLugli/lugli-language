//! Abstract Syntax Tree definitions for Lugli.
//!
//! Provides immutable AST nodes for expressions and statements, with visitor pattern support
//! for traversal and transformation. All nodes include unique IDs for source location tracking.

use lugli_common::Span;

pub mod expr;
pub mod span_map;
pub mod stmt;
pub mod type_hint;
pub mod visitor;

pub use expr::{CallData, Expr, FStringPart, ListComprehensionData, LiteralValue, MatchArm, Pattern};
pub use span_map::{NodeId, SpanMap};
pub use stmt::{IfData, Stmt, StructDeclData, StructField};
pub use type_hint::TypeHint;
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
    use lugli_lexer::TokenKind;

    fn make_id(n: u32) -> NodeId { NodeId::new(n as usize) }

    #[test]
    fn test_program_creation() {
        let span = Span {
            start: 0,
            end: 10,
        };
        let program = Program::new(vec![], span);

        assert_eq!(program.statements.len(), 0);
        assert_eq!(program.span(), &span);
    }

    #[test]
    fn test_literal_expressions() {
        let number_expr = Expr::Literal {
            id: make_id(0),
            value: LiteralValue::Number(42.0),
        };
        assert_eq!(number_expr.id(), make_id(0));

        let string_expr = Expr::Literal {
            id: make_id(1),
            value: LiteralValue::String("hello".to_string()),
        };
        assert_eq!(string_expr.id(), make_id(1));

        let bool_expr = Expr::Literal {
            id: make_id(2),
            value: LiteralValue::Boolean(true),
        };
        assert_eq!(bool_expr.id(), make_id(2));

        let null_expr = Expr::Literal {
            id: make_id(3),
            value: LiteralValue::Null,
        };
        assert_eq!(null_expr.id(), make_id(3));
    }

    #[test]
    fn test_identifier_expression() {
        let ident_expr = Expr::Identifier {
            id: make_id(0),
            name: "variable".to_string(),
        };

        assert_eq!(ident_expr.id(), make_id(0));
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
        let left = Box::new(Expr::Literal {
            id: make_id(0),
            value: LiteralValue::Number(1.0),
        });
        let right = Box::new(Expr::Literal {
            id: make_id(1),
            value: LiteralValue::Number(2.0),
        });

        let binary_expr = Expr::Binary {
            id: make_id(2),
            left,
            operator: TokenKind::Plus,
            right,
        };

        assert_eq!(binary_expr.id(), make_id(2));
    }

    #[test]
    fn test_function_call_expression() {
        let callee = Expr::Identifier {
            id: make_id(0),
            name: "function".to_string(),
        };
        let arguments = vec![
            Expr::Literal {
                id: make_id(1),
                value: LiteralValue::Number(1.0),
            },
            Expr::Literal {
                id: make_id(2),
                value: LiteralValue::String("hello".to_string()),
            },
        ];

        let call_expr = Expr::Call {
            id: make_id(3),
            data: Box::new(CallData {
                callee,
                arguments,
            }),
        };

        assert_eq!(call_expr.id(), make_id(3));
        if let Expr::Call {
            data, ..
        } = call_expr
        {
            assert_eq!(data.arguments.len(), 2);
        } else {
            panic!("Expected Call expression");
        }
    }

    #[test]
    fn test_list_expression() {
        let elements = vec![
            Expr::Literal {
                id: make_id(0),
                value: LiteralValue::Number(1.0),
            },
            Expr::Literal {
                id: make_id(1),
                value: LiteralValue::Number(2.0),
            },
            Expr::Literal {
                id: make_id(2),
                value: LiteralValue::Number(3.0),
            },
        ];

        let list_expr = Expr::List {
            id: make_id(3),
            elements,
        };

        assert_eq!(list_expr.id(), make_id(3));
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
        let pairs = vec![
            (
                Expr::Literal {
                    id: make_id(0),
                    value: LiteralValue::String("key1".to_string()),
                },
                Expr::Literal {
                    id: make_id(1),
                    value: LiteralValue::Number(1.0),
                },
            ),
            (
                Expr::Literal {
                    id: make_id(2),
                    value: LiteralValue::String("key2".to_string()),
                },
                Expr::Literal {
                    id: make_id(3),
                    value: LiteralValue::String("value2".to_string()),
                },
            ),
        ];

        let dict_expr = Expr::Dict {
            id: make_id(4),
            pairs,
        };

        assert_eq!(dict_expr.id(), make_id(4));
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
        let initializer = Some(Expr::Literal {
            id: make_id(0),
            value: LiteralValue::Number(42.0),
        });

        let var_decl = Stmt::VarDecl {
            id: make_id(1),
            name: "x".to_string(),
            type_hint: None,
            initializer,
            is_const: false,
        };

        assert_eq!(var_decl.id(), make_id(1));
        if let Stmt::VarDecl {
            name,
            is_const,
            ..
        } = var_decl
        {
            assert_eq!(name, "x");
            assert!(!is_const);
        } else {
            panic!("Expected VarDecl statement");
        }
    }

    #[test]
    fn test_function_declaration_statement() {
        let params = vec![("x".to_string(), None), ("y".to_string(), None)];
        let body = vec![Stmt::Return {
            id: make_id(0),
            value: Some(Expr::Identifier {
                id: make_id(1),
                name: "x".to_string(),
            }),
        }];

        let fn_decl = Stmt::FnDecl {
            id: make_id(2),
            name: "add".to_string(),
            params,
            return_type: None,
            body,
        };

        assert_eq!(fn_decl.id(), make_id(2));
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
        let condition = Expr::Literal {
            id: make_id(0),
            value: LiteralValue::Boolean(true),
        };
        let then_branch = vec![Stmt::Expression {
            id: make_id(1),
            expr: Expr::Literal {
                id: make_id(2),
                value: LiteralValue::String("then".to_string()),
            },
        }];
        let else_branch = Some(vec![Stmt::Expression {
            id: make_id(3),
            expr: Expr::Literal {
                id: make_id(4),
                value: LiteralValue::String("else".to_string()),
            },
        }]);

        let if_stmt = Stmt::If {
            id: make_id(5),
            data: Box::new(IfData {
                condition,
                then_branch,
                elif_branches: vec![],
                else_branch,
            }),
        };

        assert_eq!(if_stmt.id(), make_id(5));
        if let Stmt::If {
            data, ..
        } = if_stmt
        {
            assert_eq!(data.then_branch.len(), 1);
            assert!(data.else_branch.is_some());
            assert_eq!(data.else_branch.unwrap().len(), 1);
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
            fn visit_expr(&mut self, expr: &Expr) {
                self.expr_count += 1;
                self.walk_expr(expr)
            }

            fn visit_stmt(&mut self, stmt: &Stmt) {
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
                    id: make_id(0),
                    name: "x".to_string(),
                    type_hint: None,
                    initializer: Some(Expr::Literal {
                        id: make_id(1),
                        value: LiteralValue::Number(42.0),
                    }),
                    is_const: false,
                },
                Stmt::Expression {
                    id: make_id(2),
                    expr: Expr::Binary {
                        id: make_id(3),
                        left: Box::new(Expr::Identifier {
                            id: make_id(4),
                            name: "x".to_string(),
                        }),
                        operator: TokenKind::Plus,
                        right: Box::new(Expr::Literal {
                            id: make_id(5),
                            value: LiteralValue::Number(1.0),
                        }),
                    },
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
