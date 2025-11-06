use lugli_ast::{Expr, Pattern, Stmt};
use lugli_parser::Parser;

fn parse(source: &str) -> Result<(lugli_ast::Program, lugli_ast::SpanMap), lugli_parser::error::ParseError> {
    let mut parser = Parser::new(source)?;
    parser.parse()
}

#[test]
fn test_list_destructuring_simple() {
    let test_cases = vec![
        "let [x, y] = [1, 2]",
        "let [a, b, c] = items",
        "mut [first, second] = pair",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);

        let (program, _) = result.unwrap();
        match &program.statements[0] {
            Stmt::VarDecl { pattern: Pattern::List(patterns), .. } => {
                assert!(patterns.len() >= 2, "Expected at least 2 patterns in: {}", source);
            }
            _ => panic!("Expected VarDecl with List pattern for: {}", source),
        }
    }
}

#[test]
fn test_multiple_assignment_shorthand() {
    let test_cases = vec![
        ("let x, y = 10, 20", 2),
        ("let a, b, c = 1, 2, 3", 3),
        ("mut x, y, z, w = 1, 2, 3, 4", 4),
    ];

    for (source, expected_count) in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);

        let (program, _) = result.unwrap();
        match &program.statements[0] {
            Stmt::VarDecl {
                pattern: Pattern::List(patterns),
                initializer: Some(Expr::List { elements, .. }),
                ..
            } => {
                assert_eq!(patterns.len(), expected_count, "Wrong pattern count in: {}", source);
                assert_eq!(elements.len(), expected_count, "Wrong element count in: {}", source);
            }
            _ => panic!("Expected VarDecl with List pattern and List initializer for: {}", source),
        }
    }
}

#[test]
fn test_dict_destructuring_simple() {
    let test_cases = vec![
        "let {name, age} = person",
        "let {x, y} = point",
        "mut {key, value} = pair",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);

        let (program, _) = result.unwrap();
        match &program.statements[0] {
            Stmt::VarDecl { pattern: Pattern::Dict(fields), .. } => {
                assert!(fields.len() >= 2, "Expected at least 2 fields in: {}", source);
            }
            _ => panic!("Expected VarDecl with Dict pattern for: {}", source),
        }
    }
}

#[test]
fn test_dict_destructuring_with_renaming() {
    let source = "let {x: a, y: b} = point";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse dict with renaming");

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { pattern: Pattern::Dict(fields), .. } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "x");
            assert_eq!(fields[1].0, "y");

            // Check renaming
            match &fields[0].1 {
                Pattern::Identifier(name) => assert_eq!(name, "a"),
                _ => panic!("Expected identifier pattern for renamed field"),
            }
        }
        _ => panic!("Expected VarDecl with Dict pattern"),
    }
}

#[test]
fn test_dict_destructuring_shorthand() {
    let source = "let {name, age} = person";
    let result = parse(source);
    assert!(result.is_ok());

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { pattern: Pattern::Dict(fields), .. } => {
            // Shorthand {name} means {name: name}
            assert_eq!(fields[0].0, "name");
            match &fields[0].1 {
                Pattern::Identifier(n) => assert_eq!(n, "name"),
                _ => panic!("Expected identifier pattern"),
            }
        }
        _ => panic!("Expected VarDecl with Dict pattern"),
    }
}

#[test]
fn test_nested_destructuring() {
    let source = "let [x, [y, z]] = [1, [2, 3]]";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse nested destructuring");

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { pattern: Pattern::List(patterns), .. } => {
            assert_eq!(patterns.len(), 2);

            // Second element should be a nested list pattern
            match &patterns[1] {
                Pattern::List(nested) => {
                    assert_eq!(nested.len(), 2);
                }
                _ => panic!("Expected nested list pattern"),
            }
        }
        _ => panic!("Expected VarDecl with nested List pattern"),
    }
}

#[test]
fn test_for_loop_list_destructuring() {
    let test_cases = vec![
        "for [k, v] in pairs { }",
        "for [index, item] in enumerate(list) { }",
        "for [x, y, z] in coords { }",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);

        let (program, _) = result.unwrap();
        match &program.statements[0] {
            Stmt::For { pattern: Pattern::List(patterns), .. } => {
                assert!(patterns.len() >= 2, "Expected at least 2 patterns in: {}", source);
            }
            _ => panic!("Expected For with List pattern for: {}", source),
        }
    }
}

#[test]
fn test_for_loop_dict_destructuring() {
    let source = "for {key, value} in items { }";
    let result = parse(source);
    assert!(result.is_ok());

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::For { pattern: Pattern::Dict(fields), .. } => {
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected For with Dict pattern"),
    }
}

#[test]
fn test_const_destructuring() {
    let source = "const [PI, E] = [3.14, 2.71]";
    let result = parse(source);
    assert!(result.is_ok());

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl {
            pattern: Pattern::List(_),
            is_const: true,
            ..
        } => {} // Success
        _ => panic!("Expected const VarDecl with List pattern"),
    }
}

#[test]
fn test_wildcard_in_destructuring() {
    let source = "let [x, _, z] = [1, 2, 3]";
    let result = parse(source);
    assert!(result.is_ok());

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { pattern: Pattern::List(patterns), .. } => {
            assert_eq!(patterns.len(), 3);
            match &patterns[1] {
                Pattern::Wildcard => {} // Success
                _ => panic!("Expected wildcard pattern at index 1"),
            }
        }
        _ => panic!("Expected VarDecl with List pattern"),
    }
}

#[test]
fn test_mixed_destructuring() {
    let source = "let [a, {x, y}, b] = [1, point, 2]";
    let result = parse(source);
    assert!(result.is_ok());

    let (program, _) = result.unwrap();
    match &program.statements[0] {
        Stmt::VarDecl { pattern: Pattern::List(patterns), .. } => {
            assert_eq!(patterns.len(), 3);

            // Middle element should be dict pattern
            match &patterns[1] {
                Pattern::Dict(fields) => {
                    assert_eq!(fields.len(), 2);
                }
                _ => panic!("Expected dict pattern at index 1"),
            }
        }
        _ => panic!("Expected VarDecl with mixed patterns"),
    }
}

#[test]
fn test_destructuring_error_const_without_init() {
    let source = "const [x, y]";
    let result = parse(source);
    assert!(result.is_err(), "Should fail: const requires initialization");
}

#[test]
fn test_single_variable_still_works() {
    // Ensure backward compatibility
    let test_cases = vec![
        "let x = 5",
        "mut y = 10",
        "const Z = 15",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);

        let (program, _) = result.unwrap();
        match &program.statements[0] {
            Stmt::VarDecl { pattern: Pattern::Identifier(_), .. } => {} // Success
            _ => panic!("Expected VarDecl with Identifier pattern for: {}", source),
        }
    }
}
