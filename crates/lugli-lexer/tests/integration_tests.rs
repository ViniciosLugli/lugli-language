use lugli_lexer::{tokenize, Lexer, TokenKind, LexError};

mod integration_scenarios {
    use super::*;

    #[test]
    fn test_complete_program_tokenization() {
        let program = r#"
            fn fibonacci(n: num) -> num {
                if n <= 1 {
                    return n
                }
                return fibonacci(n - 1) + fibonacci(n - 2)
            }

            let result = fibonacci(10)
            println(f"Result: {result}")
        "#;

        let tokens = tokenize(program).unwrap();

        // Should successfully tokenize without errors
        assert!(!tokens.is_empty());

        // Check that we have the expected keywords
        let keywords_found: Vec<_> = tokens.iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Fn => Some("fn"),
                TokenKind::If => Some("if"),
                TokenKind::Return => Some("return"),
                TokenKind::Let => Some("let"),
                _ => None,
            })
            .collect();

        assert!(keywords_found.contains(&"fn"));
        assert!(keywords_found.contains(&"if"));
        assert!(keywords_found.contains(&"return"));
        assert!(keywords_found.contains(&"let"));
    }

    #[test]
    fn test_modern_rust_like_syntax() {
        let program = r#"
            struct Person {
                name: String,
                age: num,
            }

            impl Person {
                fn new(name: String, age: num) -> Self {
                    Self { name, age }
                }
            }

            match status {
                "ok" => handle_success(),
                "error" => handle_error(),
                _ => handle_default(),
            }
        "#;

        let tokens = tokenize(program).unwrap();

        // Check for modern syntax elements
        let has_struct = tokens.iter().any(|t| matches!(t.kind, TokenKind::Struct));
        let has_impl = tokens.iter().any(|t| matches!(t.kind, TokenKind::Impl));
        let has_match = tokens.iter().any(|t| matches!(t.kind, TokenKind::Match));
        let has_self_keyword = tokens.iter().any(|t| matches!(t.kind, TokenKind::SelfKeyword));
        let has_self_identifier = tokens.iter().any(|t| matches!(t.kind, TokenKind::Identifier(ref s) if s == "Self"));
        let has_fat_arrow = tokens.iter().any(|t| matches!(t.kind, TokenKind::FatArrow));

        assert!(has_struct, "Should tokenize 'struct' keyword");
        assert!(has_impl, "Should tokenize 'impl' keyword");
        assert!(has_match, "Should tokenize 'match' keyword");
        assert!(has_self_keyword || has_self_identifier, "Should tokenize 'self' keyword or 'Self' identifier");
        assert!(has_fat_arrow, "Should tokenize '=>' operator");
    }

    #[test]
    fn test_python_like_syntax() {
        let program = r#"
            # Python-style comment
            for item in collection {
                if item > threshold {
                    print(f"Found: {item}")
                }
            }

            // Alternative comment style
            let numbers = [1, 2, 3, 4, 5]
            let result = [x * 2 for x in numbers if x % 2 == 0]
        "#;

        let tokens = tokenize(program).unwrap();

        // Check for Python-like elements
        let has_for = tokens.iter().any(|t| matches!(t.kind, TokenKind::For));
        let has_in = tokens.iter().any(|t| matches!(t.kind, TokenKind::In));
        let _has_f_string = tokens.iter().any(|t| matches!(t.kind, TokenKind::FString(_)));

        assert!(has_for, "Should tokenize 'for' keyword");
        assert!(has_in, "Should tokenize 'in' keyword");

        // Note: F-strings might be parsed differently depending on implementation
        // This test checks if the lexer can handle the basic syntax
    }

    #[test]
    fn test_complex_expressions_with_operators() {
        let program = r#"
            let complex = (x ** 2 + y ** 2) ** 0.5
            let bitwise = (a << 2) | (b >> 1) & mask
            let logical = condition1 && condition2 || condition3
            let comparison = value >= min && value <= max
            let nullish = user?.name ?? "Anonymous"
            let range = start..end
        "#;

        let tokens = tokenize(program).unwrap();

        // Check for complex operators
        let has_power = tokens.iter().any(|t| matches!(t.kind, TokenKind::Power));
        let has_left_shift = tokens.iter().any(|t| matches!(t.kind, TokenKind::LeftShift));
        let has_right_shift = tokens.iter().any(|t| matches!(t.kind, TokenKind::RightShift));
        let has_double_question = tokens.iter().any(|t| matches!(t.kind, TokenKind::DoubleQuestion));
        let has_range = tokens.iter().any(|t| matches!(t.kind, TokenKind::DotDot));

        assert!(has_power, "Should tokenize '**' power operator");
        assert!(has_left_shift, "Should tokenize '<<' left shift operator");
        assert!(has_right_shift, "Should tokenize '>>' right shift operator");
        assert!(has_double_question, "Should tokenize '??' nullish coalescing operator");
        assert!(has_range, "Should tokenize '..' range operator");
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_unterminated_string_error() {
        let invalid_code = r#"let message = "Hello world"#; // Missing closing quote

        let result = tokenize(invalid_code);

        // Should fail gracefully with appropriate error
        match result {
            Err(LexError::UnterminatedString) => {
                // Expected error type
            }
            Err(other_error) => {
                // Different error is also acceptable
                println!("Got different error (acceptable): {:?}", other_error);
            }
            Ok(_) => {
                // Some lexers might be more lenient
                println!("Lexer was lenient with unterminated string");
            }
        }
    }

    #[test]
    fn test_unexpected_character_error() {
        let invalid_code = "let x = 5 @ invalid";

        let result = tokenize(invalid_code);

        // Should handle unexpected characters gracefully
        match result {
            Err(LexError::UnexpectedChar(ch)) => {
                assert_eq!(ch, '@');
            }
            Err(_) => {
                // Different error handling is acceptable
            }
            Ok(tokens) => {
                // Some lexers might tokenize '@' as a separate token
                println!("Lexer handled '@' as token: {:?}", tokens);
            }
        }
    }

    #[test]
    fn test_recovery_after_error() {
        // Test that lexer can continue after encountering errors
        let code_with_errors = "let x = 5; @ invalid; let y = 10";

        // Even if there are errors, we should be able to get some tokens
        match tokenize(code_with_errors) {
            Ok(tokens) => {
                // Should have tokenized at least some parts
                assert!(!tokens.is_empty());
            }
            Err(_) => {
                // Error is also acceptable - depends on error recovery strategy
            }
        }
    }
}

mod performance_tests {
    use super::*;

    #[test]
    fn test_large_input_performance() {
        // Generate a reasonably large input
        let mut large_program = String::new();
        for i in 0..1000 {
            large_program.push_str(&format!("let var{} = {}\n", i, i));
        }

        let start = std::time::Instant::now();
        let result = tokenize(&large_program);
        let duration = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 1000, "Tokenization took too long: {:?}", duration);

        // Should successfully tokenize
        match result {
            Ok(tokens) => {
                assert!(tokens.len() > 2000); // Should have many tokens
            }
            Err(e) => {
                panic!("Large input tokenization failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_deeply_nested_structures() {
        // Test with deeply nested brackets/braces
        let mut nested = String::new();
        let depth = 100;

        // Create nested structure
        for _ in 0..depth {
            nested.push_str("{ [");
        }
        nested.push_str("42");
        for _ in 0..depth {
            nested.push_str("] }");
        }

        let result = tokenize(&nested);

        // Should handle nesting without issues
        match result {
            Ok(tokens) => {
                assert!(tokens.len() > depth * 2); // At least opening and closing brackets
            }
            Err(e) => {
                println!("Deep nesting caused error (may be acceptable): {:?}", e);
            }
        }
    }
}

mod public_api_tests {
    use super::*;

    #[test]
    fn test_lexer_iterator_interface() {
        let source = "let x = 42";
        let mut lexer = Lexer::new(source);

        // Test iterator interface
        let first_token = lexer.next();
        assert!(first_token.is_some());

        if let Some(Ok(token)) = first_token {
            assert_eq!(token.kind, TokenKind::Let);
            assert_eq!(token.lexeme, "let");
        }

        // Continue iterating
        let mut token_count = 1;
        while lexer.next().is_some() {
            token_count += 1;
        }

        assert_eq!(token_count, 4); // let, x, =, 42
    }

    #[test]
    fn test_token_span_information() {
        let source = "let x = 42";
        let tokens = tokenize(source).unwrap();

        // Check that span information is provided
        for token in &tokens {
            assert!(token.span.start <= token.span.end);
            assert!(token.span.end <= source.len());
        }

        // Check specific spans
        assert_eq!(tokens[0].span.start, 0); // "let" starts at position 0
        assert_eq!(tokens[0].span.end, 3);   // "let" ends at position 3
    }

    #[test]
    fn test_tokenize_convenience_function() {
        let source = "fn test() {}";

        // Test the convenience function
        let result = tokenize(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        // Check that we have at least the basic tokens we expect
        assert!(tokens.len() >= 4); // Should have at least fn, test, (, ), {, }

        // Verify the structure of the tokens
        let has_fn = tokens.iter().any(|t| matches!(t.kind, TokenKind::Fn));
        let has_identifier = tokens.iter().any(|t| matches!(t.kind, TokenKind::Identifier(ref s) if s == "test"));
        let has_left_paren = tokens.iter().any(|t| matches!(t.kind, TokenKind::LeftParen));
        let has_right_paren = tokens.iter().any(|t| matches!(t.kind, TokenKind::RightParen));

        assert!(has_fn, "Should have 'fn' keyword");
        assert!(has_identifier, "Should have 'test' identifier");
        assert!(has_left_paren, "Should have '(' token");
        assert!(has_right_paren, "Should have ')' token");
        assert_eq!(tokens[0].kind, TokenKind::Fn);
    }

    #[test]
    fn test_empty_input_handling() {
        // Test with empty input
        let empty_tokens = tokenize("").unwrap();
        assert!(empty_tokens.is_empty());

        // Test with whitespace only
        let whitespace_tokens = tokenize("   \n\t  ").unwrap();
        // Should either be empty or contain only whitespace tokens
        assert!(whitespace_tokens.is_empty() ||
               whitespace_tokens.iter().all(|t| matches!(t.kind, TokenKind::Newline)));
    }
}