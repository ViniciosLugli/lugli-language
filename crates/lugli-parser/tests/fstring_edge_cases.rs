use lugli_parser::Parser;

#[test]
fn test_fstring_basic() {
    let source = r#"f"Hello {name}!""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "Basic f-string should parse");
}

#[test]
fn test_fstring_escaped_braces() {
    let source = r#"f"{{ escaped }} braces""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "Escaped braces should parse");
}

#[test]
fn test_fstring_multiple_expressions() {
    let source = r#"f"{a} + {b} = {a + b}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "Multiple expressions should parse");
}

#[test]
fn test_fstring_complex_expression() {
    let source = r#"f"Result: {list[0].upper()}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "Complex expressions should parse");
}

#[test]
fn test_fstring_unterminated() {
    let source = r#"f"unterminated {expr""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_err(), "Unterminated expression should error");
}

#[test]
fn test_fstring_unmatched_close_brace() {
    let source = r#"f"unmatched }""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_err(), "Unmatched close brace should error");
}

#[test]
fn test_fstring_empty_expression() {
    let source = r#"f"value: {}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    // Empty expressions should probably error, but depends on spec
    // For now, just document behavior
    let _ = result;
}

#[test]
fn test_fstring_nested_single_level() {
    // Known limitation in v0.3.0: nested f-strings not supported
    let source = r#"f"outer {f"inner {x}"}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    // Currently fails - nested f-strings are a v0.4.0 feature
    assert!(result.is_err(), "Nested f-strings not yet supported (v0.4.0 planned)");
}

// Note: Deep nesting test commented out as it may cause stack overflow
// This is a known limitation documented in v0.3.0
// #[test]
// fn test_fstring_deep_nesting() {
//     let nested = "f\"".to_string() + &"f\"".repeat(15) + "x" + &"}\"".repeat(15);
//     let mut parser = Parser::new(&nested).unwrap();
//     let result = parser.parse();
//     // Should ideally error with depth limit, but currently may succeed or overflow
// }
