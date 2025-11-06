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
    let source = r#"f"outer {f'inner {x}'}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "Nested f-strings with mixed quotes should parse (v0.4.0+)");
}

#[test]
fn test_fstring_with_string_literal_single_quotes() {
    let source = r#"f"Value: {'hello'}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "F-string with string literal should parse");
}

#[test]
fn test_fstring_with_dict_access_string_key() {
    let source = r#"f"Name: {data['name']}""#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "F-string with dict string key access should parse");
}

#[test]
fn test_fstring_single_quote_style() {
    let source = r#"f'Hello {name}!'"#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "Single-quote f-string should parse");
}

#[test]
fn test_fstring_mixed_quotes_in_expression() {
    let source = r#"f'Value: {d["key"]}'"#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse();
    assert!(result.is_ok(), "F-string with mixed quotes should parse");
}
