// Compiler tests - validates AST to bytecode compilation
//
// Tests direct AST → VM execution by manually constructing AST nodes:
// - Literal expressions (numbers, strings, booleans)
// - Binary operations
// - Variable declarations and access
// - Struct definitions and methods
// - Nested structs and complex expressions
//
// Note: Uses manual AST construction rather than parsing from source

mod helpers;

use helpers::assert_value_eq;
use lugli_ast::{CallData, Expr, IfData, LiteralValue, NodeId, Program, SpanMap, Stmt};
use lugli_common::{Span, Value};
use lugli_lexer::TokenKind;
use lugli_vm::compile_and_run;

// Helper function for creating spans
fn dummy_span() -> Span {
    Span { start: 0, end: 0 }
}

// Helper function for creating NodeIds
fn make_id(n: usize) -> NodeId {
    NodeId::new(n)
}

// Helper to create a SpanMap with dummy spans for all nodes
fn make_span_map(count: usize) -> SpanMap {
    let mut map = SpanMap::new();
    for i in 0..count {
        let id = map.alloc_id();
        map.insert(id, dummy_span());
        assert_eq!(id, make_id(i));
    }
    map
}

#[test]
fn test_compiler_literal_expressions() {
    let program = Program {
        statements: vec![Stmt::Expression {
            id: make_id(0),
            expr: Expr::Literal {
                id: make_id(1),
                value: LiteralValue::Number(42.0),
            },
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(2);
    let result = compile_and_run(&program, span_map).unwrap();
    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_compiler_binary_expressions() {
    // Test: 10 + 20 * 2 (should be 50 due to precedence)
    let program = Program {
        statements: vec![Stmt::Expression {
            id: make_id(0),
            expr: Expr::Binary {
                id: make_id(1),
                left: Box::new(Expr::Literal {
                    id: make_id(2),
                    value: LiteralValue::Number(10.0),
                }),
                operator: TokenKind::Plus,
                right: Box::new(Expr::Binary {
                    id: make_id(3),
                    left: Box::new(Expr::Literal {
                        id: make_id(4),
                        value: LiteralValue::Number(20.0),
                    }),
                    operator: TokenKind::Star,
                    right: Box::new(Expr::Literal {
                        id: make_id(5),
                        value: LiteralValue::Number(2.0),
                    }),
                }),
            },
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(6);
    let result = compile_and_run(&program, span_map).unwrap();
    assert_value_eq(&result, &Value::Number(50.0));
}

#[test]
fn test_compiler_variable_declaration_and_access() {
    let program = Program {
        statements: vec![
            // let x = 42
            Stmt::VarDecl {
                id: make_id(0),
                name: "x".to_string(),
                initializer: Some(Expr::Literal {
                    id: make_id(1),
                    value: LiteralValue::Number(42.0),
                }),
                is_const: false,
            },
            // x (access variable)
            Stmt::Expression {
                id: make_id(2),
                expr: Expr::Identifier {
                    id: make_id(3),
                    name: "x".to_string(),
                },
            },
        ],
        span: dummy_span(),
    };

    let span_map = make_span_map(4);
    let result = compile_and_run(&program, span_map).unwrap();
    assert_value_eq(&result, &Value::Number(42.0));
}

#[test]
fn test_compiler_dictionary_creation() {
    let program = Program {
        statements: vec![Stmt::Expression {
            id: make_id(0),
            expr: Expr::Dict {
                id: make_id(1),
                pairs: vec![
                    (
                        Expr::Literal {
                            id: make_id(2),
                            value: LiteralValue::String("name".to_string()),
                        },
                        Expr::Literal {
                            id: make_id(3),
                            value: LiteralValue::String("Alice".to_string()),
                        },
                    ),
                    (
                        Expr::Literal {
                            id: make_id(4),
                            value: LiteralValue::String("age".to_string()),
                        },
                        Expr::Literal {
                            id: make_id(5),
                            value: LiteralValue::Number(30.0),
                        },
                    ),
                ],
            },
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(6);
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run(&bytecode).unwrap();

    if let Value::Dict(dict) = result {
        let mut pool = bytecode.string_pool.borrow_mut();
        let name_id = pool.intern("name");
        let age_id = pool.intern("age");
        let alice_id = pool.intern("Alice");

        let dict_ref = dict.borrow();
        assert_eq!(dict_ref.len(), 2);
        assert_value_eq(dict_ref.get(&name_id).unwrap(), &Value::String(alice_id));
        assert_value_eq(dict_ref.get(&age_id).unwrap(), &Value::Number(30.0));
    } else {
        panic!("Expected Dict, got {:?}", result);
    }
}

#[test]
fn test_compiler_list_creation() {
    let program = Program {
        statements: vec![Stmt::Expression {
            id: make_id(0),
            expr: Expr::List {
                id: make_id(1),
                elements: vec![
                    Expr::Literal {
                        id: make_id(2),
                        value: LiteralValue::Number(1.0),
                    },
                    Expr::Literal {
                        id: make_id(3),
                        value: LiteralValue::Number(2.0),
                    },
                    Expr::Literal {
                        id: make_id(4),
                        value: LiteralValue::Number(3.0),
                    },
                ],
            },
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(5);
    let result = compile_and_run(&program, span_map).unwrap();

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
                id: make_id(0),
                name: "person".to_string(),
                initializer: Some(Expr::Dict {
                    id: make_id(1),
                    pairs: vec![(
                        Expr::Literal {
                            id: make_id(2),
                            value: LiteralValue::String("name".to_string()),
                        },
                        Expr::Literal {
                            id: make_id(3),
                            value: LiteralValue::String("Alice".to_string()),
                        },
                    )],
                }),
                is_const: false,
            },
            // person.name
            Stmt::Expression {
                id: make_id(4),
                expr: Expr::Get {
                    id: make_id(5),
                    object: Box::new(Expr::Identifier {
                        id: make_id(6),
                        name: "person".to_string(),
                    }),
                    name: "name".to_string(),
                },
            },
        ],
        span: dummy_span(),
    };

    let span_map = make_span_map(7);
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run(&bytecode).unwrap();
    let mut pool = bytecode.string_pool.borrow_mut();
    let alice_id = pool.intern("Alice");
    assert_value_eq(&result, &Value::String(alice_id));
}

#[test]
fn test_compiler_global_variable_assignment() {
    let program = Program {
        statements: vec![
            // let x = 10
            Stmt::VarDecl {
                id: make_id(0),
                name: "x".to_string(),
                initializer: Some(Expr::Literal {
                    id: make_id(1),
                    value: LiteralValue::Number(10.0),
                }),
                is_const: false,
            },
            // x = 20 (this assigns to local variable x)
            Stmt::Expression {
                id: make_id(2),
                expr: Expr::Set {
                    id: make_id(3),
                    object: Box::new(Expr::Identifier {
                        id: make_id(4),
                        name: "global".to_string(),
                    }),
                    name: "x".to_string(),
                    value: Box::new(Expr::Literal {
                        id: make_id(5),
                        value: LiteralValue::Number(20.0),
                    }),
                },
            },
            // x (access the local variable)
            Stmt::Expression {
                id: make_id(6),
                expr: Expr::Identifier {
                    id: make_id(7),
                    name: "x".to_string(),
                },
            },
        ],
        span: dummy_span(),
    };

    let span_map = make_span_map(8);
    let result = compile_and_run(&program, span_map).unwrap();
    assert_value_eq(&result, &Value::Number(20.0));
}

#[test]
fn test_compiler_property_assignment() {
    let program = Program {
        statements: vec![
            // let person = {"name": "Alice"}
            Stmt::VarDecl {
                id: make_id(0),
                name: "person".to_string(),
                initializer: Some(Expr::Dict {
                    id: make_id(1),
                    pairs: vec![(
                        Expr::Literal {
                            id: make_id(2),
                            value: LiteralValue::String("name".to_string()),
                        },
                        Expr::Literal {
                            id: make_id(3),
                            value: LiteralValue::String("Alice".to_string()),
                        },
                    )],
                }),
                is_const: false,
            },
            // person.name = "Bob"
            Stmt::Expression {
                id: make_id(4),
                expr: Expr::Set {
                    id: make_id(5),
                    object: Box::new(Expr::Identifier {
                        id: make_id(6),
                        name: "person".to_string(),
                    }),
                    name: "name".to_string(),
                    value: Box::new(Expr::Literal {
                        id: make_id(7),
                        value: LiteralValue::String("Bob".to_string()),
                    }),
                },
            },
            // person.name
            Stmt::Expression {
                id: make_id(8),
                expr: Expr::Get {
                    id: make_id(9),
                    object: Box::new(Expr::Identifier {
                        id: make_id(10),
                        name: "person".to_string(),
                    }),
                    name: "name".to_string(),
                },
            },
        ],
        span: dummy_span(),
    };

    let span_map = make_span_map(11);
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run(&bytecode);

    if let Err(e) = &result {
        println!("{}", lugli_vm::debug::disassemble(&bytecode, "test_compiler_property_assignment"));
        panic!(
            "Test failed with error: {}\nBytecode:\n{}",
            e,
            lugli_vm::debug::disassemble(&bytecode, "test_compiler_property_assignment")
        );
    }

    let mut pool = bytecode.string_pool.borrow_mut();
    let bob_id = pool.intern("Bob");
    assert_value_eq(&result.unwrap(), &Value::String(bob_id));
}

#[test]
fn test_compiler_return_statement() {
    let program = Program {
        statements: vec![Stmt::Return {
            id: make_id(0),
            value: Some(Expr::Literal {
                id: make_id(1),
                value: LiteralValue::String("Hello, World!".to_string()),
            }),
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(2);
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run(&bytecode).unwrap();
    let mut pool = bytecode.string_pool.borrow_mut();
    let hello_id = pool.intern("Hello, World!");
    assert_value_eq(&result, &Value::String(hello_id));
}

#[test]
fn test_compiler_complex_nested_structure() {
    // Test: {"data": [1, 2, {"nested": true}]}
    let program = Program {
        statements: vec![Stmt::Expression {
            id: make_id(0),
            expr: Expr::Dict {
                id: make_id(1),
                pairs: vec![(
                    Expr::Literal {
                        id: make_id(2),
                        value: LiteralValue::String("data".to_string()),
                    },
                    Expr::List {
                        id: make_id(3),
                        elements: vec![
                            Expr::Literal {
                                id: make_id(4),
                                value: LiteralValue::Number(1.0),
                            },
                            Expr::Literal {
                                id: make_id(5),
                                value: LiteralValue::Number(2.0),
                            },
                            Expr::Dict {
                                id: make_id(6),
                                pairs: vec![(
                                    Expr::Literal {
                                        id: make_id(7),
                                        value: LiteralValue::String("nested".to_string()),
                                    },
                                    Expr::Literal {
                                        id: make_id(8),
                                        value: LiteralValue::Boolean(true),
                                    },
                                )],
                            },
                        ],
                    },
                )],
            },
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(9);
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run(&bytecode).unwrap();

    if let Value::Dict(main_dict) = result {
        let mut pool = bytecode.string_pool.borrow_mut();
        let data_id = pool.intern("data");
        let nested_id = pool.intern("nested");

        if let Value::List(data_list) = main_dict.borrow().get(&data_id).unwrap() {
            let data = data_list.borrow();
            assert_eq!(data.len(), 3);
            assert_value_eq(&data[0], &Value::Number(1.0));
            assert_value_eq(&data[1], &Value::Number(2.0));

            if let Value::Dict(nested_dict) = &data[2] {
                assert_value_eq(nested_dict.borrow().get(&nested_id).unwrap(), &Value::Bool(true));
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
        statements: vec![Stmt::Expression {
            id: make_id(0),
            expr: Expr::Identifier {
                id: make_id(1),
                name: "undefined_var".to_string(),
            },
        }],
        span: dummy_span(),
    };

    let span_map = make_span_map(2);
    let result = compile_and_run(&program, span_map);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("undefined_var"));
}

#[test]
fn test_compiler_multiple_variable_declarations() {
    let program = Program {
        statements: vec![
            // let a = 10
            Stmt::VarDecl {
                id: make_id(0),
                name: "a".to_string(),
                initializer: Some(Expr::Literal {
                    id: make_id(1),
                    value: LiteralValue::Number(10.0),
                }),
                is_const: false,
            },
            // let b = 20
            Stmt::VarDecl {
                id: make_id(2),
                name: "b".to_string(),
                initializer: Some(Expr::Literal {
                    id: make_id(3),
                    value: LiteralValue::Number(20.0),
                }),
                is_const: false,
            },
            // a + b
            Stmt::Expression {
                id: make_id(4),
                expr: Expr::Binary {
                    id: make_id(5),
                    left: Box::new(Expr::Identifier {
                        id: make_id(6),
                        name: "a".to_string(),
                    }),
                    operator: TokenKind::Plus,
                    right: Box::new(Expr::Identifier {
                        id: make_id(7),
                        name: "b".to_string(),
                    }),
                },
            },
        ],
        span: dummy_span(),
    };

    let span_map = make_span_map(8);
    let result = compile_and_run(&program, span_map).unwrap();
    assert_value_eq(&result, &Value::Number(30.0));
}
