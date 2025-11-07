#![allow(clippy::clone_on_copy)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::needless_return)]

use lugli_ast::{AstNode, CallData, Expr, IfData, LiteralValue, NodeId, Pattern, Program, Stmt, Visitor, VisitorMut};
use lugli_common::Span;
use lugli_lexer::TokenKind;

fn make_id(n: usize) -> NodeId { NodeId::new(n) }

mod ast_construction_tests {
    use super::*;

    #[test]
    fn test_complex_nested_program() {
        let span = Span {
            start: 0,
            end: 100,
        };
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        // Build a complex program with nested structures
        let program = Program::new(
            vec![
                // fn fibonacci(n) { ... }
                Stmt::FnDecl {
                    id: next_id(),
                    name: "fibonacci".to_string(),
                    params: vec![("n".to_string(), None)],
                    param_defaults: vec![None],
                    return_type: None,
                    body: vec![Stmt::If {
                        id: next_id(),
                        data: Box::new(IfData {
                            condition: Expr::Binary {
                                id: next_id(),
                                left: Box::new(Expr::Identifier {
                                    id: next_id(),
                                    name: "n".to_string(),
                                }),
                                operator: TokenKind::LessEqual,
                                right: Box::new(Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::Number(1.0),
                                }),
                            },
                            then_branch: vec![Stmt::Return {
                                id: next_id(),
                                value: Some(Expr::Identifier {
                                    id: next_id(),
                                    name: "n".to_string(),
                                }),
                            }],
                            elif_branches: vec![],
                            else_branch: Some(vec![Stmt::Return {
                                id: next_id(),
                                value: Some(Expr::Binary {
                                    id: next_id(),
                                    left: Box::new(Expr::Call {
                                        id: next_id(),
                                        data: Box::new(CallData {
                                            callee: Expr::Identifier {
                                                id: next_id(),
                                                name: "fibonacci".to_string(),
                                            },
                                            arguments: vec![Expr::Binary {
                                                id: next_id(),
                                                left: Box::new(Expr::Identifier {
                                                    id: next_id(),
                                                    name: "n".to_string(),
                                                }),
                                                operator: TokenKind::Minus,
                                                right: Box::new(Expr::Literal {
                                                    id: next_id(),
                                                    value: LiteralValue::Number(1.0),
                                                }),
                                            }],
                                        }),
                                    }),
                                    operator: TokenKind::Plus,
                                    right: Box::new(Expr::Call {
                                        id: next_id(),
                                        data: Box::new(CallData {
                                            callee: Expr::Identifier {
                                                id: next_id(),
                                                name: "fibonacci".to_string(),
                                            },
                                            arguments: vec![Expr::Binary {
                                                id: next_id(),
                                                left: Box::new(Expr::Identifier {
                                                    id: next_id(),
                                                    name: "n".to_string(),
                                                }),
                                                operator: TokenKind::Minus,
                                                right: Box::new(Expr::Literal {
                                                    id: next_id(),
                                                    value: LiteralValue::Number(2.0),
                                                }),
                                            }],
                                        }),
                                    }),
                                }),
                            }]),
                        }),
                    }],
                },
                // let result = fibonacci(10)
                Stmt::VarDecl {
                    id: next_id(),
                    pattern: Pattern::Identifier("result".to_string()),
                    type_hint: None,
                    initializer: Some(Expr::Call {
                        id: next_id(),
                        data: Box::new(CallData {
                            callee: Expr::Identifier {
                                id: next_id(),
                                name: "fibonacci".to_string(),
                            },
                            arguments: vec![Expr::Literal {
                                id: next_id(),
                                value: LiteralValue::Number(10.0),
                            }],
                        }),
                    }),
                    is_const: false,
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
            pattern,
            initializer,
            ..
        } = &program.statements[1]
        {
            assert_eq!(pattern, &Pattern::Identifier("result".to_string()));
            assert!(initializer.is_some());
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_complex_data_structures() {
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        // Create a complex dictionary with nested structures
        let complex_dict = Expr::Dict {
            id: next_id(),
            pairs: vec![
                (
                    Expr::Literal {
                        id: next_id(),
                        value: LiteralValue::String("numbers".to_string()),
                    },
                    Expr::List {
                        id: next_id(),
                        elements: vec![
                            Expr::Literal {
                                id: next_id(),
                                value: LiteralValue::Number(1.0),
                            },
                            Expr::Literal {
                                id: next_id(),
                                value: LiteralValue::Number(2.0),
                            },
                            Expr::Binary {
                                id: next_id(),
                                left: Box::new(Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::Number(3.0),
                                }),
                                operator: TokenKind::Plus,
                                right: Box::new(Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::Number(4.0),
                                }),
                            },
                        ],
                    },
                ),
                (
                    Expr::Literal {
                        id: next_id(),
                        value: LiteralValue::String("metadata".to_string()),
                    },
                    Expr::Dict {
                        id: next_id(),
                        pairs: vec![
                            (
                                Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::String("version".to_string()),
                                },
                                Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::String("1.0".to_string()),
                                },
                            ),
                            (
                                Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::String("author".to_string()),
                                },
                                Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::String("test".to_string()),
                                },
                            ),
                        ],
                    },
                ),
            ],
        };

        // Verify structure - Dict nodes don't have span() after NodeId migration
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
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        // Create complex control flow: for loop with nested if-elif-else
        let for_loop = Stmt::For {
            id: next_id(),
            pattern: Pattern::Identifier("item".to_string()),
            iterable: Expr::Identifier {
                id: next_id(),
                name: "collection".to_string(),
            },
            body: vec![Stmt::If {
                id: next_id(),
                data: Box::new(IfData {
                    condition: Expr::Binary {
                        id: next_id(),
                        left: Box::new(Expr::Identifier {
                            id: next_id(),
                            name: "item".to_string(),
                        }),
                        operator: TokenKind::Greater,
                        right: Box::new(Expr::Literal {
                            id: next_id(),
                            value: LiteralValue::Number(10.0),
                        }),
                    },
                    then_branch: vec![Stmt::Expression {
                        id: next_id(),
                        expr: Expr::Call {
                            id: next_id(),
                            data: Box::new(CallData {
                                callee: Expr::Identifier {
                                    id: next_id(),
                                    name: "print".to_string(),
                                },
                                arguments: vec![Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::String("large".to_string()),
                                }],
                            }),
                        },
                    }],
                    elif_branches: vec![(
                        Expr::Binary {
                            id: next_id(),
                            left: Box::new(Expr::Identifier {
                                id: next_id(),
                                name: "item".to_string(),
                            }),
                            operator: TokenKind::Greater,
                            right: Box::new(Expr::Literal {
                                id: next_id(),
                                value: LiteralValue::Number(5.0),
                            }),
                        },
                        vec![Stmt::Expression {
                            id: next_id(),
                            expr: Expr::Call {
                                id: next_id(),
                                data: Box::new(CallData {
                                    callee: Expr::Identifier {
                                        id: next_id(),
                                        name: "print".to_string(),
                                    },
                                    arguments: vec![Expr::Literal {
                                        id: next_id(),
                                        value: LiteralValue::String("medium".to_string()),
                                    }],
                                }),
                            },
                        }],
                    )],
                    else_branch: Some(vec![Stmt::Expression {
                        id: next_id(),
                        expr: Expr::Call {
                            id: next_id(),
                            data: Box::new(CallData {
                                callee: Expr::Identifier {
                                    id: next_id(),
                                    name: "print".to_string(),
                                },
                                arguments: vec![Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::String("small".to_string()),
                                }],
                            }),
                        },
                    }]),
                }),
            }],
        };

        // Verify structure - For nodes don't have span() after NodeId migration
        if let Stmt::For {
            pattern,
            body,
            ..
        } = for_loop
        {
            assert_eq!(pattern, Pattern::Identifier("item".to_string()));
            assert_eq!(body.len(), 1);

            // Check nested if statement
            if let Stmt::If {
                data, ..
            } = &body[0]
            {
                assert_eq!(data.elif_branches.len(), 1);
                assert!(data.else_branch.is_some());
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
        fn visit_expr(&mut self, expr: &Expr) {
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
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        // Create program with mixed expressions
        let program = Program::new(
            vec![Stmt::VarDecl {
                id: next_id(),
                pattern: Pattern::Identifier("result".to_string()),
                type_hint: None,
                initializer: Some(Expr::Binary {
                    id: next_id(),
                    left: Box::new(Expr::Call {
                        id: next_id(),
                        data: Box::new(CallData {
                            callee: Expr::Identifier {
                                id: next_id(),
                                name: "add".to_string(),
                            },
                            arguments: vec![
                                Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::Number(1.0),
                                },
                                Expr::Literal {
                                    id: next_id(),
                                    value: LiteralValue::Number(2.0),
                                },
                            ],
                        }),
                    }),
                    operator: TokenKind::Star,
                    right: Box::new(Expr::Identifier {
                        id: next_id(),
                        name: "factor".to_string(),
                    }),
                }),
                is_const: false,
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
        fn visit_expr(&mut self, expr: &Expr) {
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
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        let program = Program::new(
            vec![Stmt::FnDecl {
                id: next_id(),
                name: "calculate".to_string(),
                params: vec![("x".to_string(), None), ("y".to_string(), None)],
                param_defaults: vec![None, None],
                return_type: None,
                body: vec![Stmt::Return {
                    id: next_id(),
                    value: Some(Expr::Binary {
                        id: next_id(),
                        left: Box::new(Expr::Identifier {
                            id: next_id(),
                            name: "x".to_string(),
                        }),
                        operator: TokenKind::Plus,
                        right: Box::new(Expr::Binary {
                            id: next_id(),
                            left: Box::new(Expr::Identifier {
                                id: next_id(),
                                name: "y".to_string(),
                            }),
                            operator: TokenKind::Star,
                            right: Box::new(Expr::Identifier {
                                id: next_id(),
                                name: "z".to_string(),
                            }),
                        }),
                    }),
                }],
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
        fn visit_expr_mut(&mut self, expr: &mut Expr) {
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
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        let mut program = Program::new(
            vec![Stmt::Expression {
                id: next_id(),
                expr: Expr::Binary {
                    id: next_id(),
                    left: Box::new(Expr::Identifier {
                        id: next_id(),
                        name: "old_var".to_string(),
                    }),
                    operator: TokenKind::Plus,
                    right: Box::new(Expr::Identifier {
                        id: next_id(),
                        name: "old_var".to_string(),
                    }),
                },
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
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        // Create expressions - NodeIds are used for tracking instead of inline spans
        let expr1 = Expr::Literal {
            id: next_id(),
            value: LiteralValue::Number(42.0),
        };

        let expr2 = Expr::Identifier {
            id: next_id(),
            name: "variable".to_string(),
        };

        let binary_expr = Expr::Binary {
            id: next_id(),
            left: Box::new(expr1),
            operator: TokenKind::Plus,
            right: Box::new(expr2),
        };

        // Verify NodeIds are assigned
        match &binary_expr {
            Expr::Binary {
                id,
                left,
                right,
                ..
            } => {
                assert_eq!(*id, make_id(2));
                match left.as_ref() {
                    Expr::Literal {
                        id, ..
                    } => assert_eq!(*id, make_id(0)),
                    _ => panic!("Expected Literal"),
                }
                match right.as_ref() {
                    Expr::Identifier {
                        id, ..
                    } => assert_eq!(*id, make_id(1)),
                    _ => panic!("Expected Identifier"),
                }
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_nested_span_consistency() {
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        let nested_call = Expr::Call {
            id: next_id(),
            data: Box::new(CallData {
                callee: Expr::Identifier {
                    id: next_id(),
                    name: "outer".to_string(),
                },
                arguments: vec![Expr::Call {
                    id: next_id(),
                    data: Box::new(CallData {
                        callee: Expr::Identifier {
                            id: next_id(),
                            name: "inner".to_string(),
                        },
                        arguments: vec![Expr::Literal {
                            id: next_id(),
                            value: LiteralValue::Number(123.0),
                        }],
                    }),
                }],
            }),
        };

        // Verify NodeId hierarchy
        match &nested_call {
            Expr::Call {
                id,
                data,
                ..
            } => {
                assert_eq!(*id, make_id(0));
                match &data.callee {
                    Expr::Identifier {
                        id, ..
                    } => assert_eq!(*id, make_id(1)),
                    _ => panic!("Expected Identifier"),
                }
                assert_eq!(data.arguments.len(), 1);

                match &data.arguments[0] {
                    Expr::Call {
                        id,
                        data,
                        ..
                    } => {
                        assert_eq!(*id, make_id(2));
                        match &data.arguments[0] {
                            Expr::Literal {
                                id, ..
                            } => assert_eq!(*id, make_id(4)),
                            _ => panic!("Expected Literal"),
                        }
                    }
                    _ => panic!("Expected Call"),
                }
            }
            _ => panic!("Expected Call"),
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
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        let program = Program::new(
            vec![Stmt::VarDecl {
                id: next_id(),
                pattern: Pattern::Identifier("test".to_string()),
                type_hint: None,
                initializer: None,
                is_const: false,
            }],
            span.clone(),
        );

        // Test AstNode trait implementation (Program still has span)
        assert_eq!(*program.span(), span);

        // Test that all statements have NodeIds
        for stmt in &program.statements {
            // Just verify id exists - NodeId is opaque type
            let _ = stmt.id();
        }
    }

    #[test]
    fn test_deep_nesting_performance() {
        let depth = 50;
        let mut id_counter = 0;
        let mut next_id = || {
            let id = id_counter;
            id_counter += 1;
            make_id(id)
        };

        // Create deeply nested binary expressions: ((((1 + 2) + 3) + 4) + ... )
        let mut expr = Expr::Literal {
            id: next_id(),
            value: LiteralValue::Number(1.0),
        };

        for i in 2..=depth {
            expr = Expr::Binary {
                id: next_id(),
                left: Box::new(expr),
                operator: TokenKind::Plus,
                right: Box::new(Expr::Literal {
                    id: next_id(),
                    value: LiteralValue::Number(i as f64),
                }),
            };
        }

        // Should handle deep nesting without issues - verify NodeId is assigned
        let _ = expr.id();

        // Count depth using visitor
        struct DepthCounter {
            max_depth: usize,
            current_depth: usize,
        }

        impl Visitor<()> for DepthCounter {
            fn visit_expr(&mut self, expr: &Expr) {
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
