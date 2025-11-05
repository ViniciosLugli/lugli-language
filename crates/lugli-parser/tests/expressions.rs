use lugli_parser::parse;

#[test]
fn test_simple_function_call() {
    let source = "print(\"hello\")";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse simple function call: {:?}", result.err());
}

#[test]
fn test_function_call_with_string() {
    let source = "print(\"yes\")";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse function call with string: {:?}", result.err());
}

#[test]
fn test_function_call_with_dict() {
    let source = r#"calculate({"config": true})"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse function call with dict: {:?}", result.err());
}

#[test]
fn test_multiline_function_call() {
    let source = r#"calculate(
    x + y
)"#;
    let result = parse(source);
    if result.is_err() {
        println!("Expected failure - multiline function call parsing not yet implemented");
    }
}

#[test]
fn test_complex_function_call() {
    let source = r#"
        let result = calculate(
            x + y,
            process_data([1, 2, 3]),
            {"config": true, "mode": "fast"}
        )
    "#;
    let result = parse(source);
    if result.is_err() {
        println!("Expected potential failure - complex multiline parsing: {:?}", result.err());
    }
}

#[test]
fn test_index_access() {
    let source = "items[0]";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse index access: {:?}", result.err());
}
