use lugli_ast::{Expr, LiteralValue, Stmt};
use lugli_parser::Parser;

fn extract_string_literal(source: &str) -> Option<String> {
    let mut parser = Parser::new(source).ok()?;
    let (ast, _span_map) = parser.parse().ok()?;

    if let Some(Stmt::VarDecl {
        initializer: Some(expr), ..
    }) = ast.statements.first()
        && let Expr::Literal {
            value: LiteralValue::String(s), ..
        } = expr
    {
        return Some(s.clone());
    }
    None
}

#[test]
fn test_newline_escape() {
    let value = extract_string_literal(r#"let s = "hello\nworld""#).expect("Should parse");
    assert_eq!(value, "hello\nworld");
    assert!(value.contains('\n'));
}

#[test]
fn test_tab_escape() {
    let value = extract_string_literal(r#"let s = "hello\tworld""#).expect("Should parse");
    assert_eq!(value, "hello\tworld");
    assert!(value.contains('\t'));
}

#[test]
fn test_carriage_return_escape() {
    let value = extract_string_literal(r#"let s = "hello\rworld""#).expect("Should parse");
    assert_eq!(value, "hello\rworld");
    assert!(value.contains('\r'));
}

#[test]
fn test_backslash_escape() {
    let value = extract_string_literal(r#"let s = "path\\to\\file""#).expect("Should parse");
    assert_eq!(value, "path\\to\\file");
    assert_eq!(value.matches('\\').count(), 2);
}

#[test]
fn test_quote_escape() {
    let value = extract_string_literal(r#"let s = "He said \"hello\"""#).expect("Should parse");
    assert_eq!(value, "He said \"hello\"");
    assert!(value.contains('"'));
}

#[test]
fn test_single_quote_escape() {
    let value = extract_string_literal(r#"let s = "It\'s working""#).expect("Should parse");
    assert_eq!(value, "It's working");
    assert!(value.contains('\''));
}

#[test]
fn test_multiple_escapes() {
    let value = extract_string_literal(r#"let s = "line1\nline2\tindented\nline3""#).expect("Should parse");
    assert_eq!(value, "line1\nline2\tindented\nline3");
    assert!(value.contains('\n'));
    assert!(value.contains('\t'));
}

#[test]
fn test_unknown_escape_preserved() {
    let value = extract_string_literal(r#"let s = "test\xunknown""#).expect("Should parse");
    assert_eq!(value, "test\\xunknown");
}

#[test]
fn test_trailing_backslash() {
    let value = extract_string_literal(r#"let s = "test\\""#).expect("Should parse");
    assert_eq!(value, "test\\");
}

#[test]
fn test_empty_string() {
    let value = extract_string_literal(r#"let s = """#).expect("Should parse");
    assert_eq!(value, "");
}

#[test]
fn test_windows_path() {
    let value = extract_string_literal(r#"let path = "C:\\Users\\Alice\\Documents\\file.txt""#).expect("Should parse");
    assert_eq!(value, "C:\\Users\\Alice\\Documents\\file.txt");
    assert_eq!(value.matches('\\').count(), 4);
}

#[test]
fn test_json_like_string() {
    let value = extract_string_literal(r#"let json = "{\"name\": \"Alice\", \"age\": 30}""#).expect("Should parse");
    assert_eq!(value, "{\"name\": \"Alice\", \"age\": 30}");
}

#[test]
fn test_escape_sequences_comprehensive() {
    let value = extract_string_literal(r#"let s = "\n\t\r\\""#).expect("Should parse");
    assert_eq!(value.len(), 4);
    assert_eq!(&value[0..1], "\n");
    assert_eq!(&value[1..2], "\t");
    assert_eq!(&value[2..3], "\r");
    assert_eq!(&value[3..4], "\\");
}
