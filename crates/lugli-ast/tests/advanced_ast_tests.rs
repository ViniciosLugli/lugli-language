#![allow(clippy::clone_on_copy)]

use lugli_ast::{
    CallData, ComprehensionClause, Expr, FStringPart, IfData, ListComprehensionData, LiteralValue,
    MatchArm, NodeId, Pattern, Program, Stmt, StructDeclData, StructField, Visitor,
};
use lugli_common::Span;
use lugli_lexer::TokenKind;

fn make_id(n: usize) -> NodeId {
    NodeId::new(n)
}

#[test]
fn test_pattern_matching_ast_construction() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // Create a match expression with multiple arms
    let match_expr = Expr::Match {
        id: next_id(),
        value: Box::new(Expr::Identifier {
            id: next_id(),
            name: "status".to_string(),
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(LiteralValue::String("success".to_string())),
                guard: None,
                body: Box::new(Expr::Literal {
                    id: next_id(),
                    value: LiteralValue::Number(1.0),
                }),
            },
            MatchArm {
                pattern: Pattern::Literal(LiteralValue::String("error".to_string())),
                guard: None,
                body: Box::new(Expr::Literal {
                    id: next_id(),
                    value: LiteralValue::Number(0.0),
                }),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::Literal {
                    id: next_id(),
                    value: LiteralValue::Number(-1.0),
                }),
            },
        ],
    };

    // Verify structure
    if let Expr::Match { value, arms, .. } = &match_expr {
        if let Expr::Identifier { name, .. } = value.as_ref() {
            assert_eq!(name, "status");
        } else {
            panic!("Expected identifier in match value");
        }

        assert_eq!(arms.len(), 3);
        assert!(matches!(arms[0].pattern, Pattern::Literal(_)));
        assert!(matches!(arms[1].pattern, Pattern::Literal(_)));
        assert!(matches!(arms[2].pattern, Pattern::Wildcard));
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_match_expression_with_guards() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // Match with guard condition
    let match_expr = Expr::Match {
        id: next_id(),
        value: Box::new(Expr::Identifier {
            id: next_id(),
            name: "code".to_string(),
        }),
        arms: vec![MatchArm {
            pattern: Pattern::Identifier("code".to_string()),
            guard: Some(Box::new(Expr::Binary {
                id: next_id(),
                left: Box::new(Expr::Identifier {
                    id: next_id(),
                    name: "code".to_string(),
                }),
                operator: TokenKind::Greater,
                right: Box::new(Expr::Literal {
                    id: next_id(),
                    value: LiteralValue::Number(400.0),
                }),
            })),
            body: Box::new(Expr::Literal {
                id: next_id(),
                value: LiteralValue::String("client_error".to_string()),
            }),
        }],
    };

    // Verify guard exists
    if let Expr::Match { arms, .. } = &match_expr {
        assert!(arms[0].guard.is_some());
        if let Some(guard) = &arms[0].guard {
            assert!(matches!(guard.as_ref(), Expr::Binary { .. }));
        }
    } else {
        panic!("Expected Match expression");
    }
}

#[test]
fn test_list_comprehension_ast() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // [x * 2 for x in numbers if x > 5]
    let comprehension = Expr::ListComprehension {
        id: next_id(),
        data: Box::new(ListComprehensionData {
            element: Expr::Binary {
                id: next_id(),
                left: Box::new(Expr::Identifier {
                    id: next_id(),
                    name: "x".to_string(),
                }),
                operator: TokenKind::Star,
                right: Box::new(Expr::Literal {
                    id: next_id(),
                    value: LiteralValue::Number(2.0),
                }),
            },
            clauses: vec![ComprehensionClause {
                pattern: Pattern::Identifier("x".to_string()),
                iterable: Expr::Identifier {
                    id: next_id(),
                    name: "numbers".to_string(),
                },
                condition: Some(Expr::Binary {
                    id: next_id(),
                    left: Box::new(Expr::Identifier {
                        id: next_id(),
                        name: "x".to_string(),
                    }),
                    operator: TokenKind::Greater,
                    right: Box::new(Expr::Literal {
                        id: next_id(),
                        value: LiteralValue::Number(5.0),
                    }),
                }),
            }],
        }),
    };

    // Verify structure
    if let Expr::ListComprehension { data, .. } = &comprehension {
        assert!(matches!(data.element, Expr::Binary { .. }));
        assert_eq!(data.clauses.len(), 1);
        assert_eq!(
            data.clauses[0].pattern,
            Pattern::Identifier("x".to_string())
        );
        assert!(data.clauses[0].condition.is_some());
    } else {
        panic!("Expected ListComprehension");
    }
}

#[test]
fn test_fstring_ast_construction() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // f"Hello, {name}! You are {age} years old."
    let fstring = Expr::FString {
        id: next_id(),
        parts: Box::new(vec![
            FStringPart::Text("Hello, ".to_string()),
            FStringPart::Expression(Box::new(Expr::Identifier {
                id: next_id(),
                name: "name".to_string(),
            })),
            FStringPart::Text("! You are ".to_string()),
            FStringPart::Expression(Box::new(Expr::Identifier {
                id: next_id(),
                name: "age".to_string(),
            })),
            FStringPart::Text(" years old.".to_string()),
        ]),
    };

    // Verify structure
    if let Expr::FString { parts, .. } = &fstring {
        assert_eq!(parts.len(), 5);
        assert!(matches!(parts[0], FStringPart::Text(_)));
        assert!(matches!(parts[1], FStringPart::Expression(_)));
        assert!(matches!(parts[2], FStringPart::Text(_)));
        assert!(matches!(parts[3], FStringPart::Expression(_)));
        assert!(matches!(parts[4], FStringPart::Text(_)));
    } else {
        panic!("Expected FString");
    }
}

#[test]
fn test_struct_declaration_ast() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // struct Point { x, y, z = 0 }
    let struct_decl = Stmt::StructDecl {
        id: next_id(),
        data: Box::new(StructDeclData {
            name: "Point".to_string(),
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    type_hint: None,
                    default: None,
                },
                StructField {
                    name: "y".to_string(),
                    type_hint: None,
                    default: None,
                },
                StructField {
                    name: "z".to_string(),
                    type_hint: None,
                    default: Some(Expr::Literal {
                        id: next_id(),
                        value: LiteralValue::Number(0.0),
                    }),
                },
            ],
            methods: vec![Stmt::FnDecl {
                id: next_id(),
                name: "distance".to_string(),
                params: vec![("self".to_string(), None)],
                return_type: None,
                body: vec![Stmt::Return {
                    id: next_id(),
                    value: Some(Expr::Literal {
                        id: next_id(),
                        value: LiteralValue::Number(0.0),
                    }),
                }],
            }],
        }),
    };

    // Verify structure
    if let Stmt::StructDecl { data, .. } = &struct_decl {
        assert_eq!(data.name, "Point");
        assert_eq!(data.fields.len(), 3);
        assert_eq!(data.methods.len(), 1);

        // Check default value on z field
        assert!(data.fields[2].default.is_some());
    } else {
        panic!("Expected StructDecl");
    }
}

#[test]
fn test_complex_property_access_chain() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // user.profile.name
    let chain = Expr::Get {
        id: next_id(),
        object: Box::new(Expr::Get {
            id: next_id(),
            object: Box::new(Expr::Identifier {
                id: next_id(),
                name: "user".to_string(),
            }),
            name: "profile".to_string(),
        }),
        name: "name".to_string(),
    };

    // Count Get expressions
    struct GetCounter {
        pub count: usize,
    }

    impl Visitor<()> for GetCounter {
        fn visit_expr(&mut self, expr: &Expr) {
            if matches!(expr, Expr::Get { .. }) {
                self.count += 1;
            }
            self.walk_expr(expr);
        }
    }

    let mut counter = GetCounter { count: 0 };
    counter.visit_expr(&chain);

    assert_eq!(counter.count, 2); // Two Get expressions in chain
}

#[test]
fn test_loop_statements_ast() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // loop { if condition { break } }
    let loop_stmt = Stmt::Loop {
        id: next_id(),
        body: vec![Stmt::If {
            id: next_id(),
            data: Box::new(IfData {
                condition: Expr::Identifier {
                    id: next_id(),
                    name: "condition".to_string(),
                },
                then_branch: vec![Stmt::Break { id: next_id() }],
                elif_branches: vec![],
                else_branch: None,
            }),
        }],
    };

    // while condition { continue }
    let while_stmt = Stmt::While {
        id: next_id(),
        condition: Expr::Identifier {
            id: next_id(),
            name: "condition".to_string(),
        },
        body: vec![Stmt::Continue { id: next_id() }],
    };

    // for item in collection { print(item) }
    let for_stmt = Stmt::For {
        id: next_id(),
        pattern: Pattern::Identifier("item".to_string()),
        iterable: Expr::Identifier {
            id: next_id(),
            name: "collection".to_string(),
        },
        body: vec![Stmt::Expression {
            id: next_id(),
            expr: Expr::Call {
                id: next_id(),
                data: Box::new(CallData {
                    callee: Expr::Identifier {
                        id: next_id(),
                        name: "print".to_string(),
                    },
                    arguments: vec![Expr::Identifier {
                        id: next_id(),
                        name: "item".to_string(),
                    }],
                }),
            },
        }],
    };

    // Verify all loop types construct correctly
    assert!(matches!(loop_stmt, Stmt::Loop { .. }));
    assert!(matches!(while_stmt, Stmt::While { .. }));
    assert!(matches!(for_stmt, Stmt::For { .. }));
}

#[test]
fn test_nested_control_flow() {
    let span = Span { start: 0, end: 100 };
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // Complex nested control flow with for/if/match
    let program = Program::new(
        vec![Stmt::For {
            id: next_id(),
            pattern: Pattern::Identifier("status".to_string()),
            iterable: Expr::Identifier {
                id: next_id(),
                name: "statuses".to_string(),
            },
            body: vec![Stmt::Expression {
                id: next_id(),
                expr: Expr::Match {
                    id: next_id(),
                    value: Box::new(Expr::Identifier {
                        id: next_id(),
                        name: "status".to_string(),
                    }),
                    arms: vec![
                        MatchArm {
                            pattern: Pattern::Literal(LiteralValue::String("ok".to_string())),
                            guard: None,
                            body: Box::new(Expr::Call {
                                id: next_id(),
                                data: Box::new(CallData {
                                    callee: Expr::Identifier {
                                        id: next_id(),
                                        name: "handle_success".to_string(),
                                    },
                                    arguments: vec![],
                                }),
                            }),
                        },
                        MatchArm {
                            pattern: Pattern::Wildcard,
                            guard: None,
                            body: Box::new(Expr::Call {
                                id: next_id(),
                                data: Box::new(CallData {
                                    callee: Expr::Identifier {
                                        id: next_id(),
                                        name: "handle_error".to_string(),
                                    },
                                    arguments: vec![],
                                }),
                            }),
                        },
                    ],
                },
            }],
        }],
        span,
    );

    // Verify structure
    assert_eq!(program.statements.len(), 1);
    if let Stmt::For { body, .. } = &program.statements[0] {
        assert_eq!(body.len(), 1);
        if let Stmt::Expression { expr, .. } = &body[0] {
            if let Expr::Match { arms, .. } = expr {
                assert_eq!(arms.len(), 2);
            } else {
                panic!("Expected Match expression");
            }
        } else {
            panic!("Expected Expression statement");
        }
    } else {
        panic!("Expected For statement");
    }
}

#[test]
fn test_complex_comprehension_with_multiple_clauses() {
    let mut id_counter = 0;
    let mut next_id = || {
        let id = id_counter;
        id_counter += 1;
        make_id(id)
    };

    // [(x, y) for x in range1 for y in range2 if x < y]
    let comprehension = Expr::ListComprehension {
        id: next_id(),
        data: Box::new(ListComprehensionData {
            element: Expr::List {
                id: next_id(),
                elements: vec![
                    Expr::Identifier {
                        id: next_id(),
                        name: "x".to_string(),
                    },
                    Expr::Identifier {
                        id: next_id(),
                        name: "y".to_string(),
                    },
                ],
            },
            clauses: vec![
                ComprehensionClause {
                    pattern: Pattern::Identifier("x".to_string()),
                    iterable: Expr::Identifier {
                        id: next_id(),
                        name: "range1".to_string(),
                    },
                    condition: None,
                },
                ComprehensionClause {
                    pattern: Pattern::Identifier("y".to_string()),
                    iterable: Expr::Identifier {
                        id: next_id(),
                        name: "range2".to_string(),
                    },
                    condition: Some(Expr::Binary {
                        id: next_id(),
                        left: Box::new(Expr::Identifier {
                            id: next_id(),
                            name: "x".to_string(),
                        }),
                        operator: TokenKind::Less,
                        right: Box::new(Expr::Identifier {
                            id: next_id(),
                            name: "y".to_string(),
                        }),
                    }),
                },
            ],
        }),
    };

    // Verify structure
    if let Expr::ListComprehension { data, .. } = &comprehension {
        assert!(matches!(data.element, Expr::List { .. }));
        assert_eq!(data.clauses.len(), 2);
        assert!(data.clauses[0].condition.is_none());
        assert!(data.clauses[1].condition.is_some());
    } else {
        panic!("Expected ListComprehension");
    }
}
