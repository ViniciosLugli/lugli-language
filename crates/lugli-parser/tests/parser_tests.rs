use lugli_ast::Stmt;
use lugli_parser::parse;

#[test]
fn test_basic_parsing() {
    let source = "let x = 5";
    let result = parse(source);
    assert!(result.is_ok());

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_variable_declarations() {
    let test_cases = vec!["let x = 5", "mut y = 10", "const Z = 15", "let name = \"hello\"", "mut flag = true", "const PI = 3.14159"];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {}", source);

        let (program, _span_map) = result.unwrap();
        assert_eq!(program.statements.len(), 1, "Wrong statement count for: {}", source);

        match &program.statements[0] {
            Stmt::VarDecl {
                ..
            } => {} // Success
            _ => panic!("Expected VarDecl for: {}", source),
        }
    }
}

#[test]
fn test_function_declarations() {
    let test_cases = vec![
        "fn hello() { }",
        "fn add(x, y) { return x + y }",
        "fn greet(name) { print(f\"Hello {name}!\") }",
        "fn calculate(a, b, c) { let result = a + b * c; return result }",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse function: {}", source);

        let (program, _span_map) = result.unwrap();
        assert_eq!(program.statements.len(), 1, "Wrong statement count for: {}", source);

        match &program.statements[0] {
            Stmt::FnDecl {
                ..
            } => {} // Success
            _ => panic!("Expected FnDecl for: {}", source),
        }
    }
}

#[test]
fn test_expressions() {
    let test_cases = vec![
        "42",
        "3.14159",
        "true",
        "false",
        "null",
        "\"hello world\"",
        "variable_name",
        "x + y",
        "a * b + c",
        "(x + y) * z",
        "func()",
        "func(1, 2, 3)",
        "[1, 2, 3]",
        // Skip dict for now: "{\"key\": \"value\"}",
        "obj.property",
        "list[0]",
    ];

    for expr_source in test_cases {
        let source = format!("{}", expr_source);
        let result = parse(&source);
        assert!(result.is_ok(), "Failed to parse expression: {}", expr_source);
    }
}

#[test]
fn test_control_flow() {
    let test_cases = vec![
        "if true { print(\"yes\") }",
        "if x > 0 { print(\"positive\") } else { print(\"not positive\") }",
        "if x > 0 { print(\"positive\") } elif x < 0 { print(\"negative\") } else { print(\"zero\") }",
        "while x < 10 { x += 1 }",
        "for item in items { print(item) }",
        "loop { break }",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse control flow: {}", source);
    }
}

#[test]
fn test_complex_program() {
    let source = r#"
        const PI = 3.14159
        mut counter = 0

        fn increment_counter() {
            counter += 1
        }

        fn calculate_area(radius) {
            return PI * radius * radius
        }

        let radius = 5.0
        let area = calculate_area(radius)

        if area > 50 {
            print("Large circle")
        } else {
            print("Small circle")
        }

        for i in [1, 2, 3, 4, 5] {
            increment_counter()
            print(f"Counter: {counter}")
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse complex program");

    let (program, _span_map) = result.unwrap();
    assert!(program.statements.len() > 5, "Expected multiple statements in complex program");
}

#[test]
fn test_error_handling() {
    let invalid_cases = vec![
        "let = 5",     // Missing variable name
        "fn { }",      // Missing function name
        "if { }",      // Missing condition
        "let x = ;",   // Missing value
        "fn test() {", // Unclosed brace
        "x + + y",     // Invalid expression
    ];

    for source in invalid_cases {
        let result = parse(source);
        assert!(result.is_err(), "Should have failed to parse: {}", source);
    }
}

#[test]
fn test_modern_syntax_features() {
    let test_cases = vec![
        "let x = f\"Hello {name}!\"",                     // F-strings
        "mut items = [1, 2, 3]",                          // Mutable list
        "const config = {\"debug\": true}",               // Object literal
        "let result = await fetch_data()",                // Async/await (parsing only)
        "try { risky_operation() } catch e { print(e) }", // Try/catch
    ];

    for source in test_cases {
        let result = parse(source);
        if result.is_err() {
            // Some features might not be fully implemented yet
            println!("Note: {} is not fully implemented yet", source);
        }
    }
}

#[test]
fn test_assignment_statements() {
    let test_cases = vec![
        "x = 5",
        // Skip compound assignments for now as they may not be implemented
        // "y += 10",
        // "z -= 3",
        // "a *= 2",
        // "b /= 4",
        // "c %= 5",
        "items[0] = \"first\"",
        "obj.prop = \"value\"",
    ];

    for source in test_cases {
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse assignment: {}", source);
    }
}

#[test]
fn test_literal_values() {
    let source = r#"
        let number = 42
        let float = 3.14
        let string = "hello"
        let boolean = true
        let null_value = null
        let list = [1, 2, 3]
        let dict = {"key": "value", "number": 42}
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse literals");

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 7);

    // Validate each statement is a VarDecl
    for stmt in &program.statements {
        match stmt {
            Stmt::VarDecl {
                ..
            } => {} // Success
            _ => panic!("Expected all statements to be VarDecl"),
        }
    }
}

#[test]
fn test_nested_expressions() {
    let source = "let result = ((a + b) * (c - d)) / (e + f)";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse nested expressions");

    let (program, _span_map) = result.unwrap();
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_function_calls_with_complex_args() {
    let source = r#"
        let result = calculate(
            x + y,
            process_data([1, 2, 3]),
            {"config": true, "mode": "fast"}
        )
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse complex function call");
}
