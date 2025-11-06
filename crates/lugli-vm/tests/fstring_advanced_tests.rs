// Advanced f-string tests for string context tracking and nested f-strings
//
// These tests validate:
// - String literals with quotes inside f-string expressions
// - Nested f-strings (f-strings containing f-strings)
// - Mixed quote styles (single and double quotes)
// - Complex expressions with strings inside interpolations
//
// Previous issue: Brace counting didn't track string context, causing:
// - Lexer errors on escaped quotes in expressions: f"Value: {d[\"key\"]}"
// - Inability to nest f-strings: f"outer {f'inner {x}'}"
//
// Current solution: extract_interpolation_expression() tracks string context

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::compile_and_run;

fn run_code(code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code).map_err(|e| e.to_string())?;
    let (ast, span_map) = parser.parse().map_err(|e| e.to_string())?;
    compile_and_run(&ast, span_map).map_err(|e| e.to_string())
}

#[test]
fn test_fstring_with_string_literal() {
    let code = r#"
        let msg = f"Value: {'hello'}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_fstring_with_dict_string_key_single_quotes() {
    let code = r#"
        let d = {"name": "Alice", "age": 30}
        let msg = f"Name: {d['name']}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_fstring_with_dict_string_key_double_quotes() {
    let code = r#"
        let d = {"city": "NYC", "country": "USA"}
        let msg = f'City: {d["city"]}'
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_nested_fstring_simple() {
    let code = r#"
        let x = 42
        let msg = f"outer {f'inner {x}'}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_nested_fstring_with_expressions() {
    let code = r#"
        let a = 5
        let b = 7
        let msg = f"Sum: {f'Result is {a + b}'}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
#[ignore] // TODO: Triple-nested f-strings require more sophisticated parsing
fn test_triple_nested_fstring() {
    let code = r#"
        let x = 10
        let msg = f"L1 {f'L2 {f"L3 {x}"}'}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success - triple nesting works!
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_fstring_with_string_method_calls() {
    let code = r#"
        let text = "hello"
        let msg = f"Upper: {text.upper()}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_fstring_mixed_quotes_complex() {
    let code = r#"
        let name = "Alice"
        let data = {"key": "value"}
        let msg = f'Name: {name}, Data: {data["key"]}'
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_fstring_with_list_access_and_strings() {
    let code = r#"
        let items = ["apple", "banana", "cherry"]
        let msg = f"First: {items[0]}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
#[ignore] // TODO: Block expressions with braces inside f-strings need special handling
fn test_fstring_with_conditional_and_strings() {
    let code = r#"
        let status = "ok"
        let msg = f"Status: {if status == "ok" { "All good" } else { "Error" }}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_fstring_single_quotes_only() {
    let code = r#"
        let x = 100
        let msg = f'Value is {x}'
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
#[ignore] // TODO: Triple-nested f-strings with alternating quotes need special handling
fn test_nested_fstring_alternating_quotes() {
    let code = r#"
        let name = "Bob"
        let age = 25
        let msg = f"Person {f'Name: {name}, Age: {f"{age}"}'}"
        msg
    "#;

    let result = run_code(code);
    match result {
        Ok(Value::String(_)) => {} // Success - alternating quotes work!
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
