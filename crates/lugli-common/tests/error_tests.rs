use lugli_common::{LugliError, Span};

mod basic_error_tests {
    use super::*;

    #[test]
    fn test_runtime_error() {
        let error = LugliError::runtime("Division by zero");

        match error {
            LugliError::Runtime(data) => {
                assert_eq!(data.message, "Division by zero");
            }
            _ => panic!("Expected Runtime error"),
        }
    }

    #[test]
    fn test_parse_error() {
        let span = Span {
            start: 10,
            end: 15,
        };
        let error = LugliError::parse("Unexpected token", span);

        match error {
            LugliError::Parse {
                message,
                span: error_span,
            } => {
                assert_eq!(message, "Unexpected token");
                assert_eq!(error_span.start, 10);
                assert_eq!(error_span.end, 15);
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_type_error() {
        let error = LugliError::type_error("number", "string");

        match error {
            LugliError::Type(data) => {
                assert_eq!(data.expected, "number");
                assert_eq!(data.found, "string");
            }
            _ => panic!("Expected Type error"),
        }
    }

    #[test]
    fn test_undefined_variable_error() {
        let error = LugliError::undefined_variable("x");

        match error {
            LugliError::UndefinedVariable(data) => {
                assert_eq!(data.name, "x");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_error_display() {
        let runtime_error = LugliError::runtime("Test runtime error");
        let display_string = format!("{}", runtime_error);
        assert!(display_string.contains("Test runtime error"));

        let span = Span {
            start: 5,
            end: 10,
        };
        let parse_error = LugliError::parse("Test parse error", span);
        let parse_display = format!("{}", parse_error);
        assert!(parse_display.contains("Test parse error"));
    }

    #[test]
    fn test_error_debug() {
        let error = LugliError::runtime("Debug test");
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("Runtime"));
        assert!(debug_string.contains("Debug test"));
    }
}

mod error_conversion_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = LugliError::runtime("String error message");

        match error {
            LugliError::Runtime(data) => {
                assert_eq!(data.message, "String error message");
            }
            _ => panic!("Expected Runtime error"),
        }
    }

    #[test]
    fn test_error_chain() {
        let original_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let lugli_error = LugliError::runtime(format!("IO Error: {}", original_error));

        match lugli_error {
            LugliError::Runtime(data) => {
                assert!(data.message.contains("IO Error"));
                assert!(data.message.contains("File not found"));
            }
            _ => panic!("Expected Runtime error"),
        }
    }
}

mod error_context_tests {
    use super::*;

    #[test]
    fn test_error_with_context() {
        let span = Span {
            start: 0,
            end: 5,
        };
        let error = LugliError::parse("Syntax error", span);

        // Test that we can extract context information
        if let LugliError::Parse {
            span: error_span, ..
        } = error
        {
            assert_eq!(error_span.start, 0);
            assert_eq!(error_span.end, 5);
        }
    }

    #[test]
    fn test_multiple_error_types() {
        let errors = vec![
            LugliError::runtime("Runtime issue"),
            LugliError::parse(
                "Parse issue",
                Span {
                    start: 0,
                    end: 1,
                },
            ),
            LugliError::type_error("expected", "actual"),
            LugliError::undefined_variable("var_name"),
        ];

        assert_eq!(errors.len(), 4);

        for error in errors {
            // All errors should be displayable
            let _display = format!("{}", error);
            let _debug = format!("{:?}", error);
        }
    }
}
