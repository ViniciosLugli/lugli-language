use lugli_common::Value;
use lugli_vm::compile_and_run;
use lugli_ast::{Program, Stmt, Expr, LiteralValue};
use lugli_lexer::Token;
use lugli_common::Span;

// Helper function for creating spans
fn dummy_span() -> Span {
    Span { start: 0, end: 0 }
}

// Helper function for value equality testing
fn assert_value_eq(actual: &Value, expected: &Value) {
    assert!(actual.equals(expected), "Values not equal: {:?} != {:?}", actual, expected);
}

#[test]
fn test_compiler_literal_expressions() {
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::Literal {
                    value: LiteralValue::Number(42.0),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_compiler_binary_expressions() {
    // Test: 10 + 20 * 2 (should be 50 due to precedence)
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: LiteralValue::Number(10.0),
                        span: dummy_span(),
                    }),
                    operator: Token::new(lugli_lexer::TokenKind::Plus, "+".to_string(), dummy_span()),
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Literal {
                            value: LiteralValue::Number(20.0),
                            span: dummy_span(),
                        }),
                        operator: Token::new(lugli_lexer::TokenKind::Star, "*".to_string(), dummy_span()),
                        right: Box::new(Expr::Literal {
                            value: LiteralValue::Number(2.0),
                            span: dummy_span(),
                        }),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::Number(50.0));
}

#[test]
fn test_compiler_variable_declaration_and_access() {
    let program = Program {
        statements: vec![
            // let x = 42
            Stmt::VarDecl {
                name: "x".to_string(),
                initializer: Some(Expr::Literal {
                    value: LiteralValue::Number(42.0),
                    span: dummy_span(),
                }),
                is_const: false,
                span: dummy_span(),
            },
            // x (access variable)
            Stmt::Expression {
                expr: Expr::Identifier {
                    name: "x".to_string(),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_compiler_dictionary_creation() {
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::Dict {
                    pairs: vec![
                        (
                            Expr::Literal {
                                value: LiteralValue::String("name".to_string()),
                                span: dummy_span(),
                            },
                            Expr::Literal {
                                value: LiteralValue::String("Alice".to_string()),
                                span: dummy_span(),
                            }
                        ),
                        (
                            Expr::Literal {
                                value: LiteralValue::String("age".to_string()),
                                span: dummy_span(),
                            },
                            Expr::Literal {
                                value: LiteralValue::Number(30.0),
                                span: dummy_span(),
                            }
                        ),
                    ],
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();

    if let Value::Dict(dict) = result {
        assert_eq!(dict.len(), 2);
        assert_value_eq(dict.get("name").unwrap(), &Value::String("Alice".to_string()));
        assert_value_eq(dict.get("age").unwrap(), &Value::Number(30.0));
    } else {
        panic!("Expected Dict, got {:?}", result);
    }
}

#[test]
fn test_compiler_list_creation() {
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::List {
                    elements: vec![
                        Expr::Literal {
                            value: LiteralValue::Number(1.0),
                            span: dummy_span(),
                        },
                        Expr::Literal {
                            value: LiteralValue::Number(2.0),
                            span: dummy_span(),
                        },
                        Expr::Literal {
                            value: LiteralValue::Number(3.0),
                            span: dummy_span(),
                        },
                    ],
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();

    if let Value::List(list) = result {
        let list_ref = list.borrow();
        assert_eq!(list_ref.len(), 3);
        assert_value_eq(&list_ref[0], &Value::Number(1.0));
        assert_value_eq(&list_ref[1], &Value::Number(2.0));
        assert_value_eq(&list_ref[2], &Value::Number(3.0));
    } else {
        panic!("Expected List, got {:?}", result);
    }
}

#[test]
fn test_compiler_property_access() {
    let program = Program {
        statements: vec![
            // let person = {"name": "Alice"}
            Stmt::VarDecl {
                name: "person".to_string(),
                initializer: Some(Expr::Dict {
                    pairs: vec![
                        (
                            Expr::Literal {
                                value: LiteralValue::String("name".to_string()),
                                span: dummy_span(),
                            },
                            Expr::Literal {
                                value: LiteralValue::String("Alice".to_string()),
                                span: dummy_span(),
                            }
                        ),
                    ],
                    span: dummy_span(),
                }),
                is_const: false,
                span: dummy_span(),
            },
            // person.name
            Stmt::Expression {
                expr: Expr::Get {
                    object: Box::new(Expr::Identifier {
                        name: "person".to_string(),
                        span: dummy_span(),
                    }),
                    name: "name".to_string(),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::String("Alice".to_string()));
}

#[test]
fn test_compiler_global_variable_assignment() {
    let program = Program {
        statements: vec![
            // let x = 10
            Stmt::VarDecl {
                name: "x".to_string(),
                initializer: Some(Expr::Literal {
                    value: LiteralValue::Number(10.0),
                    span: dummy_span(),
                }),
                is_const: false,
                span: dummy_span(),
            },
            // x = 20 (this assigns to local variable x)
            Stmt::Expression {
                expr: Expr::Set {
                    object: Box::new(Expr::Identifier {
                        name: "global".to_string(),
                        span: dummy_span(),
                    }),
                    name: "x".to_string(),
                    value: Box::new(Expr::Literal {
                        value: LiteralValue::Number(20.0),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
                span: dummy_span(),
            },
            // x (access the local variable)
            Stmt::Expression {
                expr: Expr::Identifier {
                    name: "x".to_string(),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::Number(20.0));
}

#[test]
fn test_compiler_property_assignment() {
    let program = Program {
        statements: vec![
            // let person = {"name": "Alice"}
            Stmt::VarDecl {
                name: "person".to_string(),
                initializer: Some(Expr::Dict {
                    pairs: vec![
                        (
                            Expr::Literal {
                                value: LiteralValue::String("name".to_string()),
                                span: dummy_span(),
                            },
                            Expr::Literal {
                                value: LiteralValue::String("Alice".to_string()),
                                span: dummy_span(),
                            }
                        ),
                    ],
                    span: dummy_span(),
                }),
                is_const: false,
                span: dummy_span(),
            },
            // person.name = "Bob"
            Stmt::Expression {
                expr: Expr::Set {
                    object: Box::new(Expr::Identifier {
                        name: "person".to_string(),
                        span: dummy_span(),
                    }),
                    name: "name".to_string(),
                    value: Box::new(Expr::Literal {
                        value: LiteralValue::String("Bob".to_string()),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
                span: dummy_span(),
            },
            // person.name
            Stmt::Expression {
                expr: Expr::Get {
                    object: Box::new(Expr::Identifier {
                        name: "person".to_string(),
                        span: dummy_span(),
                    }),
                    name: "name".to_string(),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::String("Bob".to_string()));
}

#[test]
fn test_compiler_return_statement() {
    let program = Program {
        statements: vec![
            Stmt::Return {
                value: Some(Expr::Literal {
                    value: LiteralValue::String("Hello, World!".to_string()),
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::String("Hello, World!".to_string()));
}

#[test]
fn test_compiler_complex_nested_structure() {
    // Test: {"data": [1, 2, {"nested": true}]}
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::Dict {
                    pairs: vec![
                        (
                            Expr::Literal {
                                value: LiteralValue::String("data".to_string()),
                                span: dummy_span(),
                            },
                            Expr::List {
                                elements: vec![
                                    Expr::Literal {
                                        value: LiteralValue::Number(1.0),
                                        span: dummy_span(),
                                    },
                                    Expr::Literal {
                                        value: LiteralValue::Number(2.0),
                                        span: dummy_span(),
                                    },
                                    Expr::Dict {
                                        pairs: vec![
                                            (
                                                Expr::Literal {
                                                    value: LiteralValue::String("nested".to_string()),
                                                    span: dummy_span(),
                                                },
                                                Expr::Literal {
                                                    value: LiteralValue::Boolean(true),
                                                    span: dummy_span(),
                                                }
                                            ),
                                        ],
                                        span: dummy_span(),
                                    },
                                ],
                                span: dummy_span(),
                            }
                        ),
                    ],
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();

    if let Value::Dict(main_dict) = result {
        if let Value::List(data_list) = main_dict.get("data").unwrap() {
            let data = data_list.borrow();
            assert_eq!(data.len(), 3);
            assert_value_eq(&data[0], &Value::Number(1.0));
            assert_value_eq(&data[1], &Value::Number(2.0));

            if let Value::Dict(nested_dict) = &data[2] {
                assert_value_eq(nested_dict.get("nested").unwrap(), &Value::Bool(true));
            } else {
                panic!("Expected nested dict");
            }
        } else {
            panic!("Expected data to be List");
        }
    } else {
        panic!("Expected main result to be Dict");
    }
}

#[test]
fn test_compiler_error_undefined_variable() {
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::Identifier {
                    name: "undefined_var".to_string(),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("undefined_var"));
}

#[test]
fn test_compiler_multiple_variable_declarations() {
    let program = Program {
        statements: vec![
            // let a = 10
            Stmt::VarDecl {
                name: "a".to_string(),
                initializer: Some(Expr::Literal {
                    value: LiteralValue::Number(10.0),
                    span: dummy_span(),
                }),
                is_const: false,
                span: dummy_span(),
            },
            // let b = 20
            Stmt::VarDecl {
                name: "b".to_string(),
                initializer: Some(Expr::Literal {
                    value: LiteralValue::Number(20.0),
                    span: dummy_span(),
                }),
                is_const: false,
                span: dummy_span(),
            },
            // a + b
            Stmt::Expression {
                expr: Expr::Binary {
                    left: Box::new(Expr::Identifier {
                        name: "a".to_string(),
                        span: dummy_span(),
                    }),
                    operator: Token::new(lugli_lexer::TokenKind::Plus, "+".to_string(), dummy_span()),
                    right: Box::new(Expr::Identifier {
                        name: "b".to_string(),
                        span: dummy_span(),
                    }),
                    span: dummy_span(),
                },
                span: dummy_span(),
            }
        ],
        span: dummy_span(),
    };

    let result = compile_and_run(&program).unwrap();
    assert_value_eq(&result, &Value::Number(30.0));
}