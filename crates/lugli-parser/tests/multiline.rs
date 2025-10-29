use lugli_parser::parse;

mod common;
use common::*;

#[test]
fn test_multiline_dict() {
    let source = r#"{
        "name": "Alice",
        "age": 30,
        "active": true
    }"#;
    assert_parse_success(source);

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse multiline dict: {:?}", result.err());

    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_multiline_dict_with_trailing_comma() {
    let source = r#"{
        "name": "Bob",
        "age": 25,
        "active": false,
    }"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_list() {
    let source = r#"[
        1,
        2,
        3,
        4,
        5
    ]"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_list_with_trailing_comma() {
    let source = r#"[
        "apple",
        "banana",
        "cherry",
    ]"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_function_call() {
    let source = r#"calculate(
        x + y,
        z * 2,
        {"config": true}
    )"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_function_call_with_complex_args() {
    let source = r#"process(
        [1, 2, 3],
        {
            "mode": "fast",
            "debug": false
        },
        "output.txt"
    )"#;
    assert_parse_success(source);
}

#[test]
fn test_nested_multiline_structures() {
    let source = r#"{
        "users": [
            {
                "name": "Alice",
                "settings": {
                    "theme": "dark",
                    "notifications": true
                }
            },
            {
                "name": "Bob",
                "settings": {
                    "theme": "light",
                    "notifications": false
                }
            }
        ],
        "config": {
            "version": "1.0",
            "features": [
                "auth",
                "notifications",
                "themes"
            ]
        }
    }"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_function_definition() {
    let source = r#"fn process_data(
        input_data,
        config
    ) {
        let result = transform(
            input_data,
            {
                "format": "json",
                "validate": true
            }
        )
        return result
    }"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_if_statement() {
    let source = r#"if (
        x > 0 &&
        y < 100 &&
        is_valid(data)
    ) {
        process_data([
            x,
            y,
            data
        ])
    }"#;
    assert_parse_success(source);
}

#[test]
fn test_multiline_variable_assignment() {
    let source = r#"let config = {
        "database": {
            "host": "localhost",
            "port": 5432,
            "name": "mydb"
        },
        "cache": {
            "enabled": true,
            "ttl": 3600
        }
    }"#;
    assert_parse_success(source);
}

#[test]
fn test_empty_multiline_structures() {
    let sources = vec![
        r#"{
        }"#,
        r#"[
        ]"#,
        r#"func(
        )"#,
    ];

    for source in sources {
        assert_parse_success(source);
    }
}

#[test]
fn test_mixed_line_styles() {
    let source = r#"let data = {"compact": true, "items": [
        1,
        2,
        3
    ], "config": {"mode": "fast"}}"#;
    assert_parse_success(source);
}
