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

// ============================================================================
// EDGE CASE TESTS - Security, Error Handling, Circular Dependencies
// ============================================================================

#[test]
fn test_missing_module_error() {
    let code = "import nonexistent_module";
    let result = run_code(code);

    assert!(result.is_err(), "Should error on missing module");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("not found") || err_msg.contains("No such file") || err_msg.contains("cannot"),
        "Error should mention module not found, got: {}",
        err_msg
    );
}

#[test]
fn test_circular_import_detection() {
    let test_dir = PathBuf::from("test_modules_circular");
    fs::create_dir_all(&test_dir).unwrap();

    // Create a.lg that imports b
    let a_content = format!("import {}.b\nlet x = 1", test_dir.display());
    fs::write(test_dir.join("a.lg"), &a_content).unwrap();

    // Create b.lg that imports a (circular!)
    let b_content = format!("import {}.a\nlet y = 2", test_dir.display());
    fs::write(test_dir.join("b.lg"), &b_content).unwrap();

    // Try to import a, which will try to import b, which will try to import a again
    let code = format!("import {}.a", test_dir.display());
    let result = run_code(&code);

    fs::remove_dir_all(&test_dir).unwrap();

    assert!(result.is_err(), "Should detect circular import");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("circular") || err_msg.contains("cycle") || err_msg.contains("recursive"),
        "Error should mention circular import, got: {}",
        err_msg
    );
}

#[test]
fn test_relative_parent_directory_rejected() {
    // Security test: .. should be rejected to prevent directory traversal
    let code = "import ../../../etc/passwd";
    let result = run_code(code);

    // Should either error during parsing or runtime
    assert!(result.is_err(), "Should reject parent directory imports");
}

#[test]
fn test_absolute_path_rejected() {
    // Security test: absolute paths should be rejected
    let code = "import /etc/passwd";
    let result = run_code(code);

    // Should either error during parsing or runtime
    assert!(result.is_err(), "Should reject absolute path imports");
}

#[test]
fn test_nested_module_imports() {
    let test_dir = PathBuf::from("test_modules_nested");
    fs::create_dir_all(&test_dir).unwrap();

    // Create c.lg with a value
    fs::write(test_dir.join("c.lg"), "let value = 42").unwrap();

    // Create b.lg that imports c
    let b_content = format!("import {}.c\nfn get_value() {{ return c.value }}", test_dir.display());
    fs::write(test_dir.join("b.lg"), &b_content).unwrap();

    // Create a.lg that imports b
    let a_content = format!("import {}.b", test_dir.display());
    fs::write(test_dir.join("a.lg"), &a_content).unwrap();

    // Import a, which imports b, which imports c
    let code = format!("import {}.a\na.b.get_value()", test_dir.display());
    let result = run_code(&code);

    fs::remove_dir_all(&test_dir).unwrap();

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0, "Nested module import should work"),
        Ok(other) => panic!("Expected Number(42), got {:?}", other),
        Err(e) => panic!("Nested import failed: {}", e),
    }
}

#[test]
fn test_from_import_nonexistent_item() {
    let test_dir = setup_test_module("nonexistent_item");

    let code = format!(r#"from {}.math import nonexistent_function"#, test_dir.display());

    let result = run_code(&code);
    cleanup_test_module("nonexistent_item");

    assert!(result.is_err(), "Should error when importing nonexistent item");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("not found") || err_msg.contains("does not exist") || err_msg.contains("cannot") || err_msg.contains("no export"),
        "Error should mention item not found, got: {}",
        err_msg
    );
}

#[test]
fn test_module_with_syntax_error() {
    let test_dir = PathBuf::from("test_modules_syntax_error");
    fs::create_dir_all(&test_dir).unwrap();

    // Create a module with syntax error
    fs::write(test_dir.join("broken.lg"), "let x = \n this is invalid syntax").unwrap();

    let code = format!("import {}.broken", test_dir.display());
    let result = run_code(&code);

    fs::remove_dir_all(&test_dir).unwrap();

    assert!(result.is_err(), "Should error on module with syntax error");
}

#[test]
fn test_module_with_runtime_error() {
    let test_dir = PathBuf::from("test_modules_runtime_error");
    fs::create_dir_all(&test_dir).unwrap();

    // Create a module that will cause runtime error
    fs::write(test_dir.join("divzero.lg"), "let x = 1 / 0").unwrap();

    let code = format!("import {}.divzero", test_dir.display());
    let result = run_code(&code);

    fs::remove_dir_all(&test_dir).unwrap();

    assert!(result.is_err(), "Should error on module with runtime error");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("division") || err_msg.contains("zero"), "Error should mention division by zero, got: {}", err_msg);
}

#[test]
fn test_import_preserves_module_scope() {
    let test_dir = setup_test_module("scope_test");

    // Module defines private variable
    fs::write(test_dir.join("scoped.lg"), "let private_var = 100\nfn get_private() { return private_var }").unwrap();

    let code = format!(
        r#"
        import {}.scoped
        scoped.get_private()
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_test_module("scope_test");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 100.0),
        Ok(other) => panic!("Expected Number(100), got {:?}", other),
        Err(e) => panic!("Module scope test failed: {}", e),
    }
}

#[test]
fn test_multiple_from_imports_same_module() {
    let test_dir = setup_test_module("multiple_from");

    let code = format!(
        r#"
        from {}.math import add
        from {}.math import multiply
        add(2, 3) + multiply(4, 5)
    "#,
        test_dir.display(),
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_test_module("multiple_from");

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 25.0), // 5 + 20
        Ok(other) => panic!("Expected Number(25), got {:?}", other),
        Err(e) => panic!("Multiple from imports failed: {}", e),
    }
}
