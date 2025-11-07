use lugli_ast::{Expr, LiteralValue, Stmt};
use lugli_parser::parse;

#[test]
fn test_struct_shorthand_basic() {
    let source = r#"
        let x = 10
        let y = 20
        let p = Point { x, y }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse struct with shorthand: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 3);

    match &program.statements[2] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                // Should have 3 fields: __struct_type__, x, y
                assert_eq!(pairs.len(), 3);

                // Check __struct_type__ = "Point"
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Literal { value: LiteralValue::String(val), .. }) = &pairs[0] {
                    assert_eq!(key, "__struct_type__");
                    assert_eq!(val, "Point");
                } else {
                    panic!("Expected __struct_type__ field");
                }

                // Check x field (shorthand desugared to x: x identifier)
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = &pairs[1] {
                    assert_eq!(key, "x");
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected x field with identifier value");
                }

                // Check y field (shorthand desugared to y: y identifier)
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = &pairs[2] {
                    assert_eq!(key, "y");
                    assert_eq!(name, "y");
                } else {
                    panic!("Expected y field with identifier value");
                }
            } else {
                panic!("Expected Dict expression for struct");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_mixed_shorthand_longhand() {
    let source = r#"
        let name = "Alice"
        let age = 25
        let p = Person { name, age: 30, email: "alice@example.com" }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse mixed shorthand/longhand: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 3);

    match &program.statements[2] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                // Should have 4 fields: __struct_type__, name, age, email
                assert_eq!(pairs.len(), 4);

                // Check name field (shorthand)
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = &pairs[1] {
                    assert_eq!(key, "name");
                    assert_eq!(name, "name");
                } else {
                    panic!("Expected name field with shorthand");
                }

                // Check age field (longhand with literal)
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Literal { value: LiteralValue::Number(val), .. }) = &pairs[2] {
                    assert_eq!(key, "age");
                    assert_eq!(*val, 30.0);
                } else {
                    panic!("Expected age field with longhand literal");
                }

                // Check email field (longhand with literal)
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Literal { value: LiteralValue::String(val), .. }) = &pairs[3] {
                    assert_eq!(key, "email");
                    assert_eq!(val, "alice@example.com");
                } else {
                    panic!("Expected email field with longhand string");
                }
            } else {
                panic!("Expected Dict expression for struct");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_all_shorthand() {
    let source = r#"
        let x = 1
        let y = 2
        let z = 3
        let vec = Vec3 { x, y, z }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse all shorthand: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 4);

    match &program.statements[3] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                assert_eq!(pairs.len(), 4); // __struct_type__ + x, y, z

                for (i, pair) in pairs.iter().enumerate().skip(1).take(3) {
                    if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = pair {
                        assert_eq!(key, name, "Field {} should use shorthand", i);
                    } else {
                        panic!("Expected shorthand for field {}", i);
                    }
                }
            } else {
                panic!("Expected Dict expression");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_shorthand_with_trailing_comma() {
    let source = r#"
        let width = 800
        let height = 600
        let size = Size { width, height, }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse with trailing comma: {:?}", result.err());
}

#[test]
fn test_struct_shorthand_multiline() {
    let source = r#"
        let name = "Bob"
        let age = 30
        let email = "bob@example.com"
        let user = User {
            name,
            age,
            email
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse multiline shorthand: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 4);

    match &program.statements[3] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                assert_eq!(pairs.len(), 4); // __struct_type__ + name, age, email

                let fields = ["name", "age", "email"];
                for (i, field) in fields.iter().enumerate() {
                    if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = &pairs[i + 1] {
                        assert_eq!(key, field);
                        assert_eq!(name, field);
                    } else {
                        panic!("Expected shorthand for {}", field);
                    }
                }
            } else {
                panic!("Expected Dict expression");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_shorthand_with_expression_fields() {
    let source = r#"
        let x = 10
        let p = Point { x, y: x + 5 }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse shorthand with expression: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    match &program.statements[1] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                assert_eq!(pairs.len(), 3); // __struct_type__, x, y

                // Check x is shorthand
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = &pairs[1] {
                    assert_eq!(key, "x");
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected x to be shorthand");
                }

                // Check y is longhand with binary expression
                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Binary { .. }) = &pairs[2] {
                    assert_eq!(key, "y");
                } else {
                    panic!("Expected y to be longhand with expression");
                }
            } else {
                panic!("Expected Dict expression");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_empty_still_works() {
    let source = "let empty = Empty {}";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse empty struct: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                assert_eq!(pairs.len(), 1); // Just __struct_type__
            } else {
                panic!("Expected Dict expression");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_shorthand_nested() {
    let source = r#"
        let x = 10
        let y = 20
        let inner = Point { x, y }
        let outer = Container { inner, value: 42 }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse nested with shorthand: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 4);

    // Check outer struct uses inner as shorthand
    match &program.statements[3] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                assert_eq!(pairs.len(), 3); // __struct_type__, inner, value

                if let (Expr::Literal { value: LiteralValue::String(key), .. }, Expr::Identifier { name, .. }) = &pairs[1] {
                    assert_eq!(key, "inner");
                    assert_eq!(name, "inner");
                } else {
                    panic!("Expected inner to be shorthand");
                }
            } else {
                panic!("Expected Dict expression");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}

#[test]
fn test_struct_shorthand_with_method_access() {
    let source = r#"
        let obj = {"field": 10}
        let value = obj.field
        let wrapper = Wrapper { value }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse shorthand after method access: {:?}", result.err());
}

#[test]
fn test_regular_dict_not_affected() {
    // Regular dict literals should still require colons
    let source = r#"let dict = {"x": 10, "y": 20}"#;
    let result = parse(source);
    assert!(result.is_ok(), "Regular dict should still work: {:?}", result.err());

    let (program, _span_map) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { initializer, .. } => {
            if let Some(Expr::Dict { pairs, .. }) = initializer {
                // Regular dict, no __struct_type__
                assert_eq!(pairs.len(), 2);
            } else {
                panic!("Expected Dict expression");
            }
        }
        _ => panic!("Expected VarDecl"),
    }
}
