// Module system integration tests
//
// - Import statements (import path.module)
// - From-import statements (from path.module import func1, func2)
// - Cross-module function calls via bytecode registry
// - Module loading, caching, and circular import detection
// - Method-style calls (module.function())
// - Direct function calls after from-import
// - IP, stack, and call stack properly saved/restored during module load
//
// Implementation: Bytecode registry system tracks which bytecode each function belongs to
// - Each bytecode gets unique ID (0 for main, 1+ for modules)
// - Functions store bytecode_id field pointing to their bytecode
// - Call instruction detects cross-bytecode calls and switches context appropriately

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::compile_and_run;
use std::{fs, path::PathBuf};

fn setup_test_module(test_name: &str) -> PathBuf {
    let test_dir = PathBuf::from(format!("test_modules_{}", test_name));
    fs::create_dir_all(&test_dir).unwrap();

    let math_content = r#"
fn add(a, b) {
    return a + b
}

fn multiply(a, b) {
    return a * b
}

fn square(x) {
    return x * x
}

let PI = 3.14159
"#;

    let math_path = test_dir.join("math.lg");
    fs::write(&math_path, math_content).unwrap();
    test_dir
}

fn cleanup_test_module(test_name: &str) {
    let test_dir = format!("test_modules_{}", test_name);
    let _ = fs::remove_dir_all(&test_dir);
}

fn run_code(code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code).map_err(|e| e.to_string())?;
    let (ast, span_map) = parser.parse().map_err(|e| e.to_string())?;
    compile_and_run(&ast, span_map).map_err(|e| e.to_string())
}

#[test]
fn test_import_basic() {
    let test_dir = setup_test_module("import_basic");

    let code = format!(
        r#"
        import {}.math
        5 + 3
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_test_module("import_basic");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 8.0),
        Ok(other) => panic!("Expected Number(8), got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_simple_import() {
    let test_dir = setup_test_module("simple_import");

    let code = format!(
        r#"
        import {}.math
        math.add(5, 3)
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_test_module("simple_import");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 8.0),
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_from_import() {
    let test_dir = setup_test_module("from_import");

    let code = format!(
        r#"
        from {}.math import add, multiply
        multiply(3, 4)
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_test_module("from_import");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 12.0),
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_caching() {
    let test_dir = setup_test_module("module_caching");

    let code = format!(
        r#"
        import {}.math
        import {}.math
        math.add(1, 1)
    "#,
        test_dir.display(),
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_test_module("module_caching");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0),
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn debug_simple_expression() {
    let code = "5 + 3";
    let mut parser = Parser::new(code).unwrap();
    let (ast, span_map) = parser.parse().unwrap();

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).unwrap();

    let mut machine = lugli_vm::Machine::new();
    let result = machine.run(&bytecode).unwrap();

    assert!(matches!(result, Value::Number(n) if (n - 8.0).abs() < 0.001));
}

#[test]
fn debug_vardecl_plus_expression() {
    let code = "let x = 1\n5 + 3";
    let mut parser = Parser::new(code).unwrap();
    let (ast, span_map) = parser.parse().unwrap();

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).unwrap();

    let mut machine = lugli_vm::Machine::new();
    let result = machine.run(&bytecode).unwrap();

    assert!(matches!(result, Value::Number(n) if (n - 8.0).abs() < 0.001));
}
