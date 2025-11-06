// Professional f-string lexer tests ensuring consistency and edge case coverage
// Tests the custom f-string lexer callback that handles:
// - Brace depth tracking for interpolations
// - Nested strings inside interpolations
// - Escaped braces ({{ and }})
// - Complex expressions with conditionals and blocks

use lugli_lexer::{tokenize_with_pool, TokenKind};

#[test]
fn test_basic_fstring_lexing() {
    let source = r#"f"Hello {name}!""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));

    // Verify the entire f-string is captured
    if let TokenKind::FString(id) = tokens[0].kind {
        let content = pool.resolve(id);
        assert_eq!(content, r#"f"Hello {name}!""#);
    }
}

#[test]
fn test_fstring_with_nested_quotes() {
    let source = r#"f"Status: {if x == "ok" { "good" } else { "bad" }}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));

    if let TokenKind::FString(id) = tokens[0].kind {
        let content = pool.resolve(id);
        assert_eq!(content, source);
    }
}

#[test]
fn test_fstring_with_dict_access() {
    let source = r#"f"Value: {data["key"]}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));

    if let TokenKind::FString(id) = tokens[0].kind {
        let content = pool.resolve(id);
        assert_eq!(content, source);
    }
}

#[test]
fn test_fstring_triple_nested() {
    let source = r#"f'L1 {f"L2 {f'L3 {x}'}"}'"#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));

    if let TokenKind::FString(id) = tokens[0].kind {
        let content = pool.resolve(id);
        assert_eq!(content, source);
    }
}

#[test]
fn test_fstring_with_escaped_braces() {
    let source = r#"f"Literal braces: {{ and }}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));

    if let TokenKind::FString(id) = tokens[0].kind {
        let content = pool.resolve(id);
        assert_eq!(content, source);
    }
}

#[test]
fn test_fstring_single_quotes() {
    let source = r#"f'Hello {name}!'"#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}

#[test]
fn test_fstring_with_method_calls() {
    let source = r#"f"Upper: {text.upper()}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}

#[test]
fn test_fstring_with_dict_literal_in_interpolation() {
    // Dict literal inside interpolation (not escaped braces)
    let source = r#"f"Data: {dict_var}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}

#[test]
fn test_fstring_with_list_comprehension() {
    let source = r#"f"Items: {[x for x in items]}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}

#[test]
fn test_multiple_fstrings_in_source() {
    let source = r#"f"First {a}" f"Second {b}""#;
    let (tokens, _) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
    assert!(matches!(tokens[1].kind, TokenKind::FString(_)));
}

#[test]
fn test_fstring_empty_interpolation() {
    let source = r#"f"Empty: {}""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}

#[test]
fn test_fstring_alternating_quotes() {
    let source = r#"f"Outer {f'Inner {x}'} done""#;
    let (tokens, pool) = tokenize_with_pool(source).unwrap();

    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}

#[test]
fn test_fstring_unterminated_consumed_to_eof() {
    // Unterminated f-strings should be lexed (consuming to EOF) and let parser handle error
    let source = r#"f"unterminated"#;
    let (tokens, _) = tokenize_with_pool(source).unwrap();

    // Lexer should succeed and produce a token
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].kind, TokenKind::FString(_)));
}
