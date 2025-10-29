use logos::Logos;
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
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(source),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?;
        let span = self.inner.span();
        let lexeme = self.inner.slice().to_string();

        match kind {
            Ok(token_kind) => Some(Ok(Token::new(token_kind, lexeme, span.into()))),
            Err(_) => Some(Err(LexError::UnexpectedChar(lexeme.chars().next().unwrap_or('\0')))),
        }
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> { Lexer::new(source).collect() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let source = "let x = 5";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens.len(), 4); // let, x, =, 5
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
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

        for (keyword, expected_kind) in keywords {
            let tokens = tokenize(keyword).unwrap();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].kind, expected_kind);
            assert_eq!(tokens[0].lexeme, keyword);
        }
    }

    #[test]
    fn test_f_string_tokenization() {
        let source = r#"f"Hello {name}!""#;
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens.len(), 1);
        if let TokenKind::FString(content) = &tokens[0].kind {
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
            let tokens = tokenize(input).unwrap();
            assert_eq!(tokens.len(), 1);
            if let TokenKind::String(content) = &tokens[0].kind {
                assert!(content.contains(expected_content));
            } else {
                panic!("Expected String token, got {:?}", tokens[0].kind);
            }
        }
    }

    #[test]
    fn test_complex_expression() {
        let source = "fn calculate(x: f64, y: f64) -> f64 { x + y }";
        let tokens = tokenize(source).unwrap();

        let expected_kinds = vec![
            TokenKind::Fn,
            TokenKind::Identifier("calculate".to_string()),
            TokenKind::LeftParen,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Colon,
            TokenKind::Identifier("f64".to_string()),
            TokenKind::Comma,
            TokenKind::Identifier("y".to_string()),
            TokenKind::Colon,
            TokenKind::Identifier("f64".to_string()),
            TokenKind::RightParen,
            TokenKind::Arrow,
            TokenKind::Identifier("f64".to_string()),
            TokenKind::LeftBrace,
            TokenKind::Identifier("x".to_string()),
            TokenKind::Plus,
            TokenKind::Identifier("y".to_string()),
            TokenKind::RightBrace,
        ];

        assert_eq!(tokens.len(), expected_kinds.len());
        for (token, expected) in tokens.iter().zip(expected_kinds.iter()) {
            assert_eq!(&token.kind, expected);
        }
    }
}
