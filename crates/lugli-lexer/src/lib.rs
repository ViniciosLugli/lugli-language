//! Lexical analysis for Lugli source code.
//!
//! Tokenizes source code into a stream of tokens using the `logos` crate for performance.
//! Supports string interning via `StringPool` to reduce memory usage.

use logos::Logos;
use lugli_common::StringPool;
use thiserror::Error;

pub mod scanner;
pub mod token;

pub use scanner::Scanner;
pub use token::{Token, TokenKind};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    #[error("Unexpected character: {0}")]
    UnexpectedChar(char),
    #[error("Unterminated string literal")]
    UnterminatedString,
    #[error("Invalid number format")]
    InvalidNumber,
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
    pool: StringPool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(source),
            pool: StringPool::new(),
        }
    }

    pub fn with_pool(source: &'a str, pool: StringPool) -> Self {
        Self {
            inner: TokenKind::lexer(source),
            pool,
        }
    }

    pub fn into_pool(self) -> StringPool { self.pool }

    pub fn pool(&self) -> &StringPool { &self.pool }

    pub fn pool_mut(&mut self) -> &mut StringPool { &mut self.pool }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?;
        let span = self.inner.span();

        match kind {
            Ok(mut token_kind) => {
                // Only allocate string for tokens that need interning
                match &mut token_kind {
                    TokenKind::String(_) | TokenKind::FString(_) => {
                        let lexeme = self.inner.slice();
                        let id = self.pool.intern(lexeme);
                        match &mut token_kind {
                            TokenKind::String(sid) => *sid = id,
                            TokenKind::FString(sid) => *sid = id,
                            _ => unreachable!(),
                        }
                    }
                    TokenKind::Identifier(_) => {
                        let lexeme = self.inner.slice();
                        let id = self.pool.intern(lexeme);
                        if let TokenKind::Identifier(sid) = &mut token_kind {
                            *sid = id;
                        }
                    }
                    _ => {}
                }

                Some(Ok(Token::new(token_kind, span.into())))
            }
            Err(_) => {
                let lexeme = self.inner.slice();
                Some(Err(LexError::UnexpectedChar(lexeme.chars().next().unwrap_or('\0'))))
            }
        }
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> { Lexer::new(source).collect() }

pub fn tokenize_with_pool(source: &str) -> Result<(Vec<Token>, StringPool), LexError> {
    let mut lexer = Lexer::new(source);
    let tokens: Result<Vec<Token>, LexError> = lexer.by_ref().collect();
    tokens.map(|t| (t, lexer.into_pool()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let source = "let x = 5";
        let (tokens, pool) = tokenize_with_pool(source).unwrap();

        assert_eq!(tokens.len(), 4); // let, x, =, 5
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(_)));
        assert_eq!(
            pool.resolve(match tokens[1].kind {
                TokenKind::Identifier(id) => id,
                _ => panic!(),
            }),
            "x"
        );
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number(5.0));
    }

    #[test]
    fn test_modern_keywords() {
        let keywords = vec![
            ("let", TokenKind::Let),
            ("mut", TokenKind::Mut),
            ("const", TokenKind::Const),
            ("self", TokenKind::SelfKeyword),
            ("match", TokenKind::Match),
            ("try", TokenKind::Try),
            ("catch", TokenKind::Catch),
            ("async", TokenKind::Async),
            ("await", TokenKind::Await),
        ];

        for (_keyword, expected_kind) in keywords {
            let tokens = tokenize(_keyword).unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind, expected_kind);
        }
    }

    #[test]
    fn test_f_string_tokenization() {
        let source = r#"f"Hello {name}!""#;
        let (tokens, pool) = tokenize_with_pool(source).unwrap();

        assert_eq!(tokens.len(), 1);
        if let TokenKind::FString(id) = &tokens[0].kind {
            let content = pool.resolve(*id);
            assert!(content.starts_with("f\""));
            assert!(content.ends_with("\""));
        } else {
            panic!("Expected FString token, got {:?}", tokens[0].kind);
        }
    }

    #[test]
    fn test_operators() {
        let operators = vec![
            ("->", TokenKind::Arrow),
            ("::", TokenKind::DoubleColon),
            ("=>", TokenKind::FatArrow),
            ("//", TokenKind::IntegerDivision),
            ("**", TokenKind::Power),
            ("<<", TokenKind::LeftShift),
            (">>", TokenKind::RightShift),
            ("??", TokenKind::DoubleQuestion),
        ];

        for (op, expected_kind) in operators {
            let tokens = tokenize(op).unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind, expected_kind);
        }
    }

    #[test]
    fn test_comments_are_skipped() {
        let source = "let x = 5 # This is a comment\nlet y = 10";
        let tokens = tokenize(source).unwrap();

        // Should have: let, x, =, 5, newline, let, y, =, 10
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[5].kind, TokenKind::Let);
    }

    #[test]
    fn test_numbers() {
        let numbers = vec![("42", 42.0), ("3.14", 3.14), ("-5", -5.0), ("0.0", 0.0)];

        for (input, expected) in numbers {
            let tokens = tokenize(input).unwrap();
            assert_eq!(tokens.len(), 1);
            if let TokenKind::Number(value) = tokens[0].kind {
                assert_eq!(value, expected);
            } else {
                panic!("Expected Number token, got {:?}", tokens[0].kind);
            }
        }
    }

    #[test]
    fn test_strings() {
        let strings = vec![(r#""hello""#, "hello"), (r#""hello world""#, "hello world"), (r#""with\nnewline""#, "with\\nnewline")];

        for (input, expected_content) in strings {
            let (tokens, pool) = tokenize_with_pool(input).unwrap();
            assert_eq!(tokens.len(), 1);
            if let TokenKind::String(id) = &tokens[0].kind {
                let content = pool.resolve(*id);
                assert!(content.contains(expected_content));
            } else {
                panic!("Expected String token, got {:?}", tokens[0].kind);
            }
        }
    }

    #[test]
    fn test_complex_expression() {
        let source = "fn calculate(x: f64, y: f64) -> f64 { x + y }";
        let (tokens, pool) = tokenize_with_pool(source).unwrap();

        // Verify token count and basic structure
        assert_eq!(tokens.len(), 18);

        // Verify first token is fn keyword
        assert_eq!(tokens[0].kind, TokenKind::Fn);

        // Verify identifier tokens
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(_)));
        assert_eq!(
            pool.resolve(match tokens[1].kind {
                TokenKind::Identifier(id) => id,
                _ => panic!(),
            }),
            "calculate"
        );

        // Verify some key tokens
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[11].kind, TokenKind::Arrow);
        assert_eq!(tokens[13].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[15].kind, TokenKind::Plus);
        assert_eq!(tokens[17].kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_token_size_reduction() {
        use std::mem::size_of;

        let token_size = size_of::<Token>();

        // Token should be significantly smaller without lexeme field
        // Original: ~48 bytes (TokenKind discriminant + String 24 bytes + Span 16 bytes + padding)
        // Actual: 32 bytes (TokenKind discriminant 16 bytes + Span 16 bytes)
        // This is a 33% reduction in size!
        assert!(token_size <= 32, "Token size should be <= 32 bytes, got {} bytes", token_size);

        let reduction_percent = ((48.0 - token_size as f64) / 48.0 * 100.0) as u32;
        println!("✅ Token size: {} bytes (reduced {}% from 48 bytes)", token_size, reduction_percent);
    }
}
