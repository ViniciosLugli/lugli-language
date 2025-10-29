use lugli_lexer::{TokenKind, tokenize};

#[test]
fn test_import_keyword() {
    let tokens = tokenize("import").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Import);
}

#[test]
fn test_from_keyword() {
    let tokens = tokenize("from").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::From);
}

#[test]
fn test_export_keyword() {
    let tokens = tokenize("export").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Export);
}

#[test]
fn test_as_keyword() {
    let tokens = tokenize("as").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::As);
}

#[test]
fn test_simple_import_statement() {
    let tokens = tokenize("import math").expect("Should tokenize");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Import);
    match &tokens[1].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "math"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_from_import_statement() {
    let tokens = tokenize("from math import pi").expect("Should tokenize");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::From);
    match &tokens[1].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "math"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[2].kind, TokenKind::Import);
    match &tokens[3].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "pi"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_import_with_alias() {
    let tokens = tokenize("import math as m").expect("Should tokenize");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::Import);
    match &tokens[1].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "math"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[2].kind, TokenKind::As);
    match &tokens[3].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "m"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_export_function() {
    let tokens = tokenize("export fn foo() {}").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::Export);
    assert_eq!(tokens[1].kind, TokenKind::Fn);
}

#[test]
fn test_module_path_with_dots() {
    let tokens = tokenize("import stdlib.collections.list").expect("Should tokenize");
    // import, stdlib, ., collections, ., list, EOF
    assert_eq!(tokens[0].kind, TokenKind::Import);
    match &tokens[1].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "stdlib"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[2].kind, TokenKind::Dot);
    match &tokens[3].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "collections"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[4].kind, TokenKind::Dot);
    match &tokens[5].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "list"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_relative_import_path() {
    let tokens = tokenize("from ./utils import helper").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::From);
    assert_eq!(tokens[1].kind, TokenKind::Dot);
    assert_eq!(tokens[2].kind, TokenKind::Slash);
    match &tokens[3].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "utils"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[4].kind, TokenKind::Import);
}

#[test]
fn test_export_named_items() {
    let tokens = tokenize("export { foo, bar }").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::Export);
    assert_eq!(tokens[1].kind, TokenKind::LeftBrace);
    match &tokens[2].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "foo"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[3].kind, TokenKind::Comma);
    match &tokens[4].kind {
        TokenKind::Identifier(name) => assert_eq!(name, "bar"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[5].kind, TokenKind::RightBrace);
}

#[test]
fn test_wildcard_import() {
    let tokens = tokenize("from math import *").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::From);
    assert_eq!(tokens[2].kind, TokenKind::Import);
    assert_eq!(tokens[3].kind, TokenKind::Star);
}
