use lugli_ast::{AstNode, Expr, LiteralValue, Program, Stmt, Visitor, VisitorMut};
use lugli_common::Span;
use lugli_lexer::{Token, TokenKind};

mod ast_construction_tests {
    use super::*;

    #[test]
    fn test_complex_nested_program() {
        let span = Span {
            start: 0,
            end: 100,
        };

        // Build a complex program with nested structures
        let program = Program::new(
            vec![
                // fn fibonacci(n) { ... }
                Stmt::FnDecl {
                    name: "fibonacci".to_string(),
                    params: vec!["n".to_string()],
                    body: vec![Stmt::If {
                        condition: Expr::Binary {
                            left: Box::new(Expr::Identifier {
                                name: "n".to_string(),
                                span: span.clone(),
                            }),
                            operator: Token::new(TokenKind::LessEqual, "<=".to_string(), span.clone()),
                            right: Box::new(Expr::Literal {
                                value: LiteralValue::Number(1.0),
                                span: span.clone(),
                            }),
                            span: span.clone(),
                        },
                        then_branch: vec![Stmt::Return {
                            value: Some(Expr::Identifier {
                                name: "n".to_string(),
                                span: span.clone(),
                            }),
                            span: span.clone(),
                        }],
                        elif_branches: vec![],
                        else_branch: Some(vec![Stmt::Return {
                            value: Some(Expr::Binary {
                                left: Box::new(Expr::Call {
                                    callee: Box::new(Expr::Identifier {
                                        name: "fibonacci".to_string(),
                                        span: span.clone(),
                                    }),
                                    arguments: vec![Expr::Binary {
                                        left: Box::new(Expr::Identifier {
                                            name: "n".to_string(),
                                            span: span.clone(),
                                        }),
                                        operator: Token::new(TokenKind::Minus, "-".to_string(), span.clone()),
                                        right: Box::new(Expr::Literal {
                                            value: LiteralValue::Number(1.0),
                                            span: span.clone(),
                                        }),
                                        span: span.clone(),
                                    }],
                                    span: span.clone(),
                                }),
                                operator: Token::new(TokenKind::Plus, "+".to_string(), span.clone()),
                                right: Box::new(Expr::Call {
                                    callee: Box::new(Expr::Identifier {
                                        name: "fibonacci".to_string(),
                                        span: span.clone(),
                                    }),
                                    arguments: vec![Expr::Binary {
                                        left: Box::new(Expr::Identifier {
                                            name: "n".to_string(),
                                            span: span.clone(),
                                        }),
                                        operator: Token::new(TokenKind::Minus, "-".to_string(), span.clone()),
                                        right: Box::new(Expr::Literal {
                                            value: LiteralValue::Number(2.0),
                                            span: span.clone(),
                                        }),
                                        span: span.clone(),
                                    }],
                                    span: span.clone(),
                                }),
                                span: span.clone(),
                            }),
                            span: span.clone(),
                        }]),
                        span: span.clone(),
                    }],
                    span: span.clone(),
                },
                // let result = fibonacci(10)
                Stmt::VarDecl {
                    name: "result".to_string(),
                    initializer: Some(Expr::Call {
                        callee: Box::new(Expr::Identifier {
                            name: "fibonacci".to_string(),
                            span: span.clone(),
                        }),
                        arguments: vec![Expr::Literal {
                            value: LiteralValue::Number(10.0),
                            span: span.clone(),
                        }],
                        span: span.clone(),
                    }),
                    is_const: false,
                    span: span.clone(),
                },
            ],
            span.clone(),
        );

        // Verify structure
        assert_eq!(program.statements.len(), 2);
        assert_eq!(*program.span(), span);

        // Verify function declaration
        if let Stmt::FnDecl {
            name,
            params,
            body,
            ..
        } = &program.statements[0]
        {
            assert_eq!(name, "fibonacci");
            assert_eq!(params.len(), 1);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected function declaration");
        }

        // Verify variable declaration
        if let Stmt::VarDecl {
            name,
            initializer,
            ..
        } = &program.statements[1]
        {
            assert_eq!(name, "result");
            assert!(initializer.is_some());
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_complex_data_structures() {
        let span = Span {
            start: 0,
            end: 50,
        };

        // Create a complex dictionary with nested structures
        let complex_dict = Expr::Dict {
            pairs: vec![
                (
                    Expr::Literal {
                        value: LiteralValue::String("numbers".to_string()),
                        span: span.clone(),
                    },
                    Expr::List {
                        elements: vec![
                            Expr::Literal {
                                value: LiteralValue::Number(1.0),
                                span: span.clone(),
                            },
                            Expr::Literal {
                                value: LiteralValue::Number(2.0),
                                span: span.clone(),
                            },
                            Expr::Binary {
                                left: Box::new(Expr::Literal {
                                    value: LiteralValue::Number(3.0),
                                    span: span.clone(),
                                }),
                                operator: Token::new(TokenKind::Plus, "+".to_string(), span.clone()),
                                right: Box::new(Expr::Literal {
                                    value: LiteralValue::Number(4.0),
                                    span: span.clone(),
                                }),
                                span: span.clone(),
                            },
                        ],
                        span: span.clone(),
                    },
                ),
                (
                    Expr::Literal {
                        value: LiteralValue::String("metadata".to_string()),
                        span: span.clone(),
                    },
                    Expr::Dict {
                        pairs: vec![
                            (
                                Expr::Literal {
                                    value: LiteralValue::String("version".to_string()),
                                    span: span.clone(),
                                },
                                Expr::Literal {
                                    value: LiteralValue::String("1.0".to_string()),
                                    span: span.clone(),
                                },
                            ),
                            (
                                Expr::Literal {
                                    value: LiteralValue::String("author".to_string()),
                                    span: span.clone(),
                                },
                                Expr::Literal {
                                    value: LiteralValue::String("test".to_string()),
                                    span: span.clone(),
                                },
                            ),
                        ],
                        span: span.clone(),
                    },
                ),
            ],
            span: span.clone(),
        };

        // Verify structure
        assert_eq!(*complex_dict.span(), span);

        if let Expr::Dict {
            pairs, ..
        } = complex_dict
        {
            assert_eq!(pairs.len(), 2);

            // Check the numbers list
            if let (
                _,
                Expr::List {
                    elements, ..
                },
            ) = &pairs[0]
            {
                assert_eq!(elements.len(), 3);
            } else {
                panic!("Expected list in first pair");
            }

            // Check the nested metadata dict
            if let (
                _,
                Expr::Dict {
                    pairs: inner_pairs, ..
                },
            ) = &pairs[1]
            {
                assert_eq!(inner_pairs.len(), 2);
            } else {
                panic!("Expected dict in second pair");
            }
        } else {
            panic!("Expected Dict expression");
        }
    }

    #[test]
    fn test_control_flow_structures() {
        let span = Span {
            start: 0,
            end: 80,
        };

        // Create complex control flow: for loop with nested if-elif-else
        let for_loop = Stmt::For {
            variable: "item".to_string(),
            iterable: Expr::Identifier {
                name: "collection".to_string(),
                span: span.clone(),
            },
            body: vec![Stmt::If {
                condition: Expr::Binary {
                    left: Box::new(Expr::Identifier {
                        name: "item".to_string(),
                        span: span.clone(),
                    }),
                    operator: Token::new(TokenKind::Greater, ">".to_string(), span.clone()),
                    right: Box::new(Expr::Literal {
                        value: LiteralValue::Number(10.0),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                },
                then_branch: vec![Stmt::Expression {
                    expr: Expr::Call {
                        callee: Box::new(Expr::Identifier {
                            name: "print".to_string(),
                            span: span.clone(),
                        }),
                        arguments: vec![Expr::Literal {
                            value: LiteralValue::String("large".to_string()),
                            span: span.clone(),
                        }],
                        span: span.clone(),
                    },
                    span: span.clone(),
                }],
                elif_branches: vec![(
                    Expr::Binary {
                        left: Box::new(Expr::Identifier {
                            name: "item".to_string(),
                            span: span.clone(),
                        }),
                        operator: Token::new(TokenKind::Greater, ">".to_string(), span.clone()),
                        right: Box::new(Expr::Literal {
                            value: LiteralValue::Number(5.0),
                            span: span.clone(),
                        }),
                        span: span.clone(),
                    },
                    vec![Stmt::Expression {
                        expr: Expr::Call {
                            callee: Box::new(Expr::Identifier {
                                name: "print".to_string(),
                                span: span.clone(),
                            }),
                            arguments: vec![Expr::Literal {
                                value: LiteralValue::String("medium".to_string()),
                                span: span.clone(),
                            }],
                            span: span.clone(),
                        },
                        span: span.clone(),
                    }],
                )],
                else_branch: Some(vec![Stmt::Expression {
                    expr: Expr::Call {
                        callee: Box::new(Expr::Identifier {
                            name: "print".to_string(),
                            span: span.clone(),
                        }),
                        arguments: vec![Expr::Literal {
                            value: LiteralValue::String("small".to_string()),
                            span: span.clone(),
                        }],
                        span: span.clone(),
                    },
                    span: span.clone(),
                }]),
                span: span.clone(),
            }],
            span: span.clone(),
        };

        // Verify structure
        assert_eq!(*for_loop.span(), span);

        if let Stmt::For {
            variable,
            body,
            ..
        } = for_loop
        {
            assert_eq!(variable, "item");
            assert_eq!(body.len(), 1);

            // Check nested if statement
            if let Stmt::If {
                elif_branches,
                else_branch,
                ..
            } = &body[0]
            {
                assert_eq!(elif_branches.len(), 1);
                assert!(else_branch.is_some());
            } else {
                panic!("Expected If statement in for loop body");
            }
        } else {
            panic!("Expected For statement");
        }
    }
}

mod visitor_pattern_tests {
    use super::*;

    struct ExpressionCounter {
        pub literal_count: usize,
        pub identifier_count: usize,
        pub binary_count: usize,
        pub call_count: usize,
        pub total_expr_count: usize,
    }

    impl Visitor<()> for ExpressionCounter {
        fn visit_expr(&mut self, expr: &Expr) -> () {
            self.total_expr_count += 1;

            match expr {
                Expr::Literal {
                    ..
                } => self.literal_count += 1,
                Expr::Identifier {
                    ..
                } => self.identifier_count += 1,
                Expr::Binary {
                    ..
                } => self.binary_count += 1,
                Expr::Call {
                    ..
                } => self.call_count += 1,
                _ => {}
            }

            self.walk_expr(expr)
        }
    }

    #[test]
    fn test_visitor_counting() {
        let span = Span {
            start: 0,
            end: 20,
        };

        // Create program with mixed expressions
        let program = Program::new(
            vec![Stmt::VarDecl {
                name: "result".to_string(),
                initializer: Some(Expr::Binary {
                    left: Box::new(Expr::Call {
                        callee: Box::new(Expr::Identifier {
                            name: "add".to_string(),
                            span: span.clone(),
                        }),
                        arguments: vec![
                            Expr::Literal {
                                value: LiteralValue::Number(1.0),
                                span: span.clone(),
                            },
                            Expr::Literal {
                                value: LiteralValue::Number(2.0),
                                span: span.clone(),
                            },
                        ],
                        span: span.clone(),
                    }),
                    operator: Token::new(TokenKind::Star, "*".to_string(), span.clone()),
                    right: Box::new(Expr::Identifier {
                        name: "factor".to_string(),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                }),
                is_const: false,
                span: span.clone(),
            }],
            span,
        );

        let mut counter = ExpressionCounter {
            literal_count: 0,
            identifier_count: 0,
            binary_count: 0,
            call_count: 0,
            total_expr_count: 0,
        };

        counter.visit_program(&program);

        // Expected: 2 literals (1.0, 2.0), 2 identifiers (add, factor), 1 binary (*), 1 call
        // (add(...))
        assert_eq!(counter.literal_count, 2);
        assert_eq!(counter.identifier_count, 2);
        assert_eq!(counter.binary_count, 1);
        assert_eq!(counter.call_count, 1);
        assert_eq!(counter.total_expr_count, 6); // Total of all expressions
    }

    struct IdentifierCollector {
        pub identifiers: Vec<String>,
    }

    impl Visitor<()> for IdentifierCollector {
        fn visit_expr(&mut self, expr: &Expr) -> () {
            if let Expr::Identifier {
                name, ..
            } = expr
            {
                self.identifiers.push(name.clone());
            }
            self.walk_expr(expr)
        }
    }

    #[test]
    fn test_identifier_collection() {
        let span = Span {
            start: 0,
            end: 30,
        };

        let program = Program::new(
            vec![Stmt::FnDecl {
                name: "calculate".to_string(),
                params: vec!["x".to_string(), "y".to_string()],
                body: vec![Stmt::Return {
                    value: Some(Expr::Binary {
                        left: Box::new(Expr::Identifier {
                            name: "x".to_string(),
                            span: span.clone(),
                        }),
                        operator: Token::new(TokenKind::Plus, "+".to_string(), span.clone()),
                        right: Box::new(Expr::Binary {
                            left: Box::new(Expr::Identifier {
                                name: "y".to_string(),
                                span: span.clone(),
                            }),
                            operator: Token::new(TokenKind::Star, "*".to_string(), span.clone()),
                            right: Box::new(Expr::Identifier {
                                name: "z".to_string(),
                                span: span.clone(),
                            }),
                            span: span.clone(),
                        }),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                }],
                span: span.clone(),
            }],
            span,
        );

        let mut collector = IdentifierCollector {
            identifiers: Vec::new(),
        };

        collector.visit_program(&program);

        // Should collect identifiers: x, y, z
        assert_eq!(collector.identifiers.len(), 3);
        assert!(collector.identifiers.contains(&"x".to_string()));
        assert!(collector.identifiers.contains(&"y".to_string()));
        assert!(collector.identifiers.contains(&"z".to_string()));
    }

    struct NameReplacer {
        pub old_name: String,
        pub new_name: String,
    }

    impl VisitorMut<()> for NameReplacer {
        fn visit_expr_mut(&mut self, expr: &mut Expr) -> () {
            if let Expr::Identifier {
                name, ..
            } = expr
            {
                if *name == self.old_name {
                    *name = self.new_name.clone();
                }
            }
            self.walk_expr_mut(expr)
        }
    }

    #[test]
    fn test_mutable_visitor() {
        let span = Span {
            start: 0,
            end: 15,
        };

        let mut program = Program::new(
            vec![Stmt::Expression {
                expr: Expr::Binary {
                    left: Box::new(Expr::Identifier {
                        name: "old_var".to_string(),
                        span: span.clone(),
                    }),
                    operator: Token::new(TokenKind::Plus, "+".to_string(), span.clone()),
                    right: Box::new(Expr::Identifier {
                        name: "old_var".to_string(),
                        span: span.clone(),
                    }),
                    span: span.clone(),
                },
                span: span.clone(),
            }],
            span,
        );

        let mut replacer = NameReplacer {
            old_name: "old_var".to_string(),
            new_name: "new_var".to_string(),
        };

        replacer.visit_program_mut(&mut program);

        // Verify that identifiers were replaced
        if let Stmt::Expression {
            expr, ..
        } = &program.statements[0]
        {
            if let Expr::Binary {
                left,
                right,
                ..
            } = expr
            {
                if let Expr::Identifier {
                    name: left_name, ..
                } = left.as_ref()
                {
                    assert_eq!(left_name, "new_var");
                }
                if let Expr::Identifier {
                    name: right_name, ..
                } = right.as_ref()
                {
                    assert_eq!(right_name, "new_var");
                }
            }
        }
    }
}

mod span_tracking_tests {
    use super::*;

    #[test]
    fn test_comprehensive_span_tracking() {
        let span1 = Span {
            start: 0,
            end: 5,
        };
        let span2 = Span {
            start: 6,
            end: 10,
        };
        let span3 = Span {
            start: 11,
            end: 20,
        };

        // Create expressions with different spans
        let expr1 = Expr::Literal {
            value: LiteralValue::Number(42.0),
            span: span1.clone(),
        };

        let expr2 = Expr::Identifier {
            name: "variable".to_string(),
            span: span2.clone(),
        };

        let binary_expr = Expr::Binary {
            left: Box::new(expr1),
            operator: Token::new(TokenKind::Plus, "+".to_string(), span2.clone()),
            right: Box::new(expr2),
            span: span3.clone(),
        };

        // Verify spans are preserved
        assert_eq!(*binary_expr.span(), span3);

        if let Expr::Binary {
            left,
            right,
            ..
        } = binary_expr
        {
            assert_eq!(*left.span(), span1);
            assert_eq!(*right.span(), span2);
        }
    }

    #[test]
    fn test_nested_span_consistency() {
        let outer_span = Span {
            start: 0,
            end: 50,
        };
        let inner_spans = vec![
            Span {
                start: 5,
                end: 10,
            },
            Span {
                start: 15,
                end: 25,
            },
            Span {
                start: 30,
                end: 45,
            },
        ];

        let nested_call = Expr::Call {
            callee: Box::new(Expr::Identifier {
                name: "outer".to_string(),
                span: inner_spans[0].clone(),
            }),
            arguments: vec![Expr::Call {
                callee: Box::new(Expr::Identifier {
                    name: "inner".to_string(),
                    span: inner_spans[1].clone(),
                }),
                arguments: vec![Expr::Literal {
                    value: LiteralValue::Number(123.0),
                    span: inner_spans[2].clone(),
                }],
                span: inner_spans[1].clone(),
            }],
            span: outer_span.clone(),
        };

        // Verify span hierarchy
        assert_eq!(*nested_call.span(), outer_span);

        if let Expr::Call {
            callee,
            arguments,
            ..
        } = nested_call
        {
            assert_eq!(*callee.span(), inner_spans[0]);
            assert_eq!(arguments.len(), 1);

            if let Expr::Call {
                arguments: inner_args, ..
            } = &arguments[0]
            {
                assert_eq!(inner_args.len(), 1);
                assert_eq!(*inner_args[0].span(), inner_spans[2]);
            }
        }
    }
}

mod ast_node_interface_tests {
    use super::*;

    #[test]
    fn test_ast_node_trait_consistency() {
        let span = Span {
            start: 10,
            end: 20,
        };

        let program = Program::new(
            vec![Stmt::VarDecl {
                name: "test".to_string(),
                initializer: None,
                is_const: false,
                span: span.clone(),
            }],
            span.clone(),
        );

        // Test AstNode trait implementation
        assert_eq!(*program.span(), span);

        // Test that span is consistent across all statements
        for stmt in &program.statements {
            assert_eq!(*stmt.span(), span);
        }
    }

    #[test]
    fn test_deep_nesting_performance() {
        let span = Span {
            start: 0,
            end: 100,
        };
        let depth = 50;

        // Create deeply nested binary expressions: ((((1 + 2) + 3) + 4) + ... )
        let mut expr = Expr::Literal {
            value: LiteralValue::Number(1.0),
            span: span.clone(),
        };

        for i in 2..=depth {
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: Token::new(TokenKind::Plus, "+".to_string(), span.clone()),
                right: Box::new(Expr::Literal {
                    value: LiteralValue::Number(i as f64),
                    span: span.clone(),
                }),
                span: span.clone(),
            };
        }

        // Should handle deep nesting without issues
        assert_eq!(*expr.span(), span);

        // Count depth using visitor
        struct DepthCounter {
            max_depth: usize,
            current_depth: usize,
        }

        impl Visitor<()> for DepthCounter {
            fn visit_expr(&mut self, expr: &Expr) -> () {
                self.current_depth += 1;
                if self.current_depth > self.max_depth {
                    self.max_depth = self.current_depth;
                }
                self.walk_expr(expr);
                self.current_depth -= 1;
            }
        }

        let mut counter = DepthCounter {
            max_depth: 0,
            current_depth: 0,
        };

        counter.visit_expr(&expr);
        assert_eq!(counter.max_depth, depth); // Should be exactly depth expressions deep
    }
}
