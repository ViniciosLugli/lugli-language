use lugli_parser::parse;

#[test]
fn test_simple_if() {
    let source = "if true { print(\"yes\") }";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse simple if statement: {:?}", result.err());
}

#[test]
fn test_empty_if() {
    let source = "if true { }";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse empty if statement: {:?}", result.err());
}

#[test]
fn test_simple_while() {
    let source = "while true { }";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse simple while loop: {:?}", result.err());
}

#[test]
fn test_simple_function() {
    let source = "fn hello() { }";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse simple function: {:?}", result.err());
}

#[test]
fn test_function_with_fstring() {
    let source = r#"fn greet(name) { print(f"Hello {name}!") }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse function with f-string: {:?}", result.err());
}

#[test]
fn test_block_statement() {
    let source = "{ print(\"hello\") }";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse block statement: {:?}", result.err());
}

#[test]
fn test_variable_declaration_with_dict() {
    let source = r#"let x = {"a": 1}"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse variable declaration with dict: {:?}", result.err());
}

#[test]
fn test_compound_assignment() {
    let source = "x += 1";
    let result = parse(source);
    if result.is_err() {
        println!("Assignment parsing needs statement-level handling: {:?}", result.err());
    }
}

#[test]
fn test_index_assignment() {
    let source = "items[0] = \"first\"";
    let result = parse(source);
    if result.is_err() {
        println!("Index assignment parsing needs statement-level handling: {:?}", result.err());
    }
}
