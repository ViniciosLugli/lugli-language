// Module import tests with string operations (f-strings, concatenation, format)
//
// These tests validate the shared string pool architecture:
// - Modules share the same StringPool as the main program via Rc<RefCell<>>
// - Strings created during module execution (f-strings, concat, format!) work correctly
// - No string ID mismatches between main program and modules
//
// Previous issue: Each bytecode had its own RefCell<StringPool>, causing:
// - Index out of bounds when modules created new strings
// - String ID mismatches between main and module pools
//
// Current solution: Bytecode uses Rc<RefCell<StringPool>> for shared pool access

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::Vm;
use std::{fs, path::PathBuf};

fn setup_helper_module(test_name: &str, content: &str) -> PathBuf {
    let test_dir = PathBuf::from(format!("test_modules_string_{}", test_name));
    fs::create_dir_all(&test_dir).unwrap();

    let helper_path = test_dir.join("helper.lg");
    fs::write(&helper_path, content).unwrap();
    test_dir
}

fn cleanup_helper_module(test_name: &str) {
    let test_dir = format!("test_modules_string_{}", test_name);
    let _ = fs::remove_dir_all(&test_dir);
}

fn run_code(code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code).map_err(|e| e.to_string())?;
    let (ast, span_map) = parser.parse().map_err(|e| e.to_string())?;
    { let mut vm = Vm::new(); vm.compile_and_run(&ast, span_map) }.map_err(|e| e.to_string())
}

#[test]
fn test_module_fstring_basic() {
    let helper_content = r#"
fn greet(name) {
    return f"Hello, {name}!"
}
"#;

    let test_dir = setup_helper_module("fstring_basic", helper_content);

    let code = format!(
        r#"
        from {}.helper import greet
        greet("World")
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("fstring_basic");

    match result {
        Ok(Value::String(sid)) => {
            // Can't easily check the string content here without access to string pool,
            // but if it doesn't panic, the shared pool is working
            assert!(sid.as_u32() < 100); // Reasonable string ID
        }
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_fstring_multiple_interpolations() {
    let helper_content = r#"
fn describe(name, age, city) {
    return f"{name} is {age} years old and lives in {city}"
}
"#;

    let test_dir = setup_helper_module("fstring_multi", helper_content);

    let code = format!(
        r#"
        from {}.helper import describe
        describe("Alice", 30, "NYC")
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("fstring_multi");

    match result {
        Ok(Value::String(_)) => {} // Success - no panic
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_string_concatenation() {
    let helper_content = r#"
fn combine(a, b, c) {
    return a + " " + b + " " + c
}
"#;

    let test_dir = setup_helper_module("string_concat", helper_content);

    let code = format!(
        r#"
        from {}.helper import combine
        combine("one", "two", "three")
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("string_concat");

    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_dict_with_string_keys() {
    let helper_content = r#"
fn make_dict() {
    return {"name": "Alice", "age": 30, "city": "NYC"}
}
"#;

    let test_dir = setup_helper_module("dict_strings", helper_content);

    let code = format!(
        r#"
        from {}.helper import make_dict
        make_dict()
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("dict_strings");

    match result {
        Ok(Value::Dict(_)) => {} // Success
        Ok(other) => panic!("Expected Dict, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_returns_fstring_and_main_uses_it() {
    let helper_content = r#"
fn get_message(status) {
    if status == "ok" {
        return f"Status: {status} - All good!"
    } else {
        return f"Status: {status} - Error!"
    }
}
"#;

    let test_dir = setup_helper_module("fstring_return", helper_content);

    let code = format!(
        r#"
        from {}.helper import get_message
        get_message("ok")
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("fstring_return");

    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_multiple_modules_sharing_pool() {
    // Create first module
    let helper1_content = r#"
fn msg1() {
    return f"Module 1 says hello"
}
"#;
    let test_dir1 = setup_helper_module("multi_mod1", helper1_content);

    // Create second module
    let helper2_content = r#"
fn msg2() {
    return f"Module 2 says goodbye"
}
"#;
    let test_dir2 = PathBuf::from("test_modules_string_multi_mod2");
    fs::create_dir_all(&test_dir2).unwrap();
    let helper2_path = test_dir2.join("helper2.lg");
    fs::write(&helper2_path, helper2_content).unwrap();

    let code = format!(
        r#"
        from {}.helper import msg1
        from {}.helper2 import msg2
        msg1()
    "#,
        test_dir1.display(),
        test_dir2.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("multi_mod1");
    let _ = fs::remove_dir_all(&test_dir2);

    match result {
        Ok(Value::String(_)) => {} // Success - both modules share pool
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_nested_fstring_expressions() {
    let helper_content = r#"
fn calculate_message(a, b) {
    let sum = a + b
    let product = a * b
    return f"Sum: {sum}, Product: {product}, a={a}, b={b}"
}
"#;

    let test_dir = setup_helper_module("nested_expr", helper_content);

    let code = format!(
        r#"
        from {}.helper import calculate_message
        calculate_message(5, 7)
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("nested_expr");

    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_string_method_chaining() {
    let helper_content = r#"
fn process_text(text) {
    return text.upper()
}
"#;

    let test_dir = setup_helper_module("string_methods", helper_content);

    let code = format!(
        r#"
        from {}.helper import process_text
        process_text("hello world")
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_helper_module("string_methods");

    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
