use lugli_lexer::{TokenKind, tokenize_with_pool};

#[test]
fn test_import_keyword() {
    let (tokens, _pool) = tokenize_with_pool("import").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Import);
}

#[test]
fn test_from_keyword() {
    let (tokens, _pool) = tokenize_with_pool("from").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::From);
}

#[test]
fn test_export_keyword() {
    let (tokens, _pool) = tokenize_with_pool("export").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Export);
}

#[test]
fn test_as_keyword() {
    let (tokens, _pool) = tokenize_with_pool("as").expect("Should tokenize");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::As);
}

#[test]
fn test_simple_import_statement() {
    let (tokens, pool) = tokenize_with_pool("import math").expect("Should tokenize");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Import);
    match &tokens[1].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "math"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_from_import_statement() {
    let (tokens, pool) = tokenize_with_pool("from math import pi").expect("Should tokenize");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::From);
    match &tokens[1].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "math"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[2].kind, TokenKind::Import);
    match &tokens[3].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "pi"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_import_with_alias() {
    let (tokens, pool) = tokenize_with_pool("import math as m").expect("Should tokenize");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].kind, TokenKind::Import);
    match &tokens[1].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "math"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[2].kind, TokenKind::As);
    match &tokens[3].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "m"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_export_function() {
    let (tokens, _pool) = tokenize_with_pool("export fn foo() {}").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::Export);
    assert_eq!(tokens[1].kind, TokenKind::Fn);
}

#[test]
fn test_module_path_with_dots() {
    let (tokens, pool) = tokenize_with_pool("import stdlib.collections.list").expect("Should tokenize");
    // import, stdlib, ., collections, ., list, EOF
    assert_eq!(tokens[0].kind, TokenKind::Import);
    match &tokens[1].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "stdlib"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[2].kind, TokenKind::Dot);
    match &tokens[3].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "collections"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[4].kind, TokenKind::Dot);
    match &tokens[5].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "list"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_relative_import_path() {
    let (tokens, pool) = tokenize_with_pool("from ./utils import helper").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::From);
    assert_eq!(tokens[1].kind, TokenKind::Dot);
    assert_eq!(tokens[2].kind, TokenKind::Slash);
    match &tokens[3].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "utils"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[4].kind, TokenKind::Import);
}

#[test]
fn test_export_named_items() {
    let (tokens, pool) = tokenize_with_pool("export { foo, bar }").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::Export);
    assert_eq!(tokens[1].kind, TokenKind::LeftBrace);
    match &tokens[2].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "foo"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[3].kind, TokenKind::Comma);
    match &tokens[4].kind {
        TokenKind::Identifier(id) => assert_eq!(pool.resolve(*id), "bar"),
        _ => panic!("Expected identifier"),
    }
    assert_eq!(tokens[5].kind, TokenKind::RightBrace);
}

#[test]
fn test_wildcard_import() {
    let (tokens, _pool) = tokenize_with_pool("from math import *").expect("Should tokenize");
    assert_eq!(tokens[0].kind, TokenKind::From);
    assert_eq!(tokens[2].kind, TokenKind::Import);
    assert_eq!(tokens[3].kind, TokenKind::Star);
}
