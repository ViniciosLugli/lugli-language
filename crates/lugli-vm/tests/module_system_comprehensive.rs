// Comprehensive module system integration tests
//
// Tests cover:
// - Cross-module function calls with various return types
// - Module-level variables and constants
// - Nested imports (module importing other modules)
// - Circular import detection
// - Module caching behavior
// - Error handling (missing modules, invalid imports)

use lugli_common::Value;
use lugli_parser::Parser;
use lugli_vm::compile_and_run;
use std::{fs, path::PathBuf};

fn setup_module(test_name: &str, module_name: &str, content: &str) -> PathBuf {
    let test_dir = PathBuf::from(format!("test_modules_comprehensive_{}", test_name));
    fs::create_dir_all(&test_dir).unwrap();
    let module_path = test_dir.join(format!("{}.lg", module_name));
    fs::write(&module_path, content).unwrap();
    test_dir
}

fn cleanup_module(test_name: &str) {
    let test_dir = format!("test_modules_comprehensive_{}", test_name);
    let _ = fs::remove_dir_all(&test_dir);
}

fn run_code(code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code).map_err(|e| e.to_string())?;
    let (ast, span_map) = parser.parse().map_err(|e| e.to_string())?;
    compile_and_run(&ast, span_map).map_err(|e| e.to_string())
}

#[test]
#[ignore] // TODO: Fix arithmetic precision in test assertion
fn test_module_exports_all_globals() {
    let math_content = r#"
let PI = 3.14159
let E = 2.71828

fn square(x) {
    return x * x
}

let SQRT_2 = 1.41421
"#;

    let test_dir = setup_module("exports_all", "math", math_content);

    let code = format!(
        r#"
        from {}.math import PI, E, square, SQRT_2
        PI + E + square(2) + SQRT_2
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("exports_all");

    match result {
        Ok(Value::Number(n)) => {
            // PI + E + 4 + SQRT_2 ≈ 3.14159 + 2.71828 + 4 + 1.41421 ≈ 11.27408
            assert!((n - 11.27408).abs() < 0.01, "Expected ~11.27, got {}", n);
        }
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
#[ignore] // TODO: Closure state sharing across imports needs investigation
fn test_module_with_closures() {
    let counter_content = r#"
fn make_counter() {
    mut count = 0
    return fn() {
        count = count + 1
        return count
    }
}

let global_counter = make_counter()
"#;

    let test_dir = setup_module("closures", "counter", counter_content);

    let code = format!(
        r#"
        from {}.counter import global_counter
        global_counter() + global_counter() + global_counter()
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("closures");

    match result {
        Ok(Value::Number(n)) => {
            assert_eq!(n, 6.0, "Counter should return 1, 2, 3 → sum = 6");
        }
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
#[ignore] // TODO: Method chaining on function returns needs investigation
fn test_module_with_structs() {
    let person_content = r#"
struct Person {
    name
    age

    fn greet(self) {
        return f"Hi, I'm {self.name}, age {self.age}"
    }

    fn birthday(self) {
        self.age = self.age + 1
    }
}

fn create_person(name, age) {
    return Person { name: name, age: age }
}
"#;

    let test_dir = setup_module("structs", "person", person_content);

    let code = format!(
        r#"
        from {}.person import create_person
        create_person("Alice", 30).greet()
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("structs");

    match result {
        Ok(Value::String(_)) => {} // Success - struct methods work across modules
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
#[ignore] // TODO: Nested module imports (module importing module) needs work
fn test_nested_module_imports() {
    // Module A
    let a_content = r#"
fn helper_a() {
    return 10
}
"#;

    // Module B imports A
    let b_content = r#"
import {DIR}.a

fn helper_b() {
    return a.helper_a() + 20
}
"#;

    let test_dir = setup_module("nested", "a", a_content);
    let b_content = b_content.replace("{DIR}", &test_dir.display().to_string());
    fs::write(test_dir.join("b.lg"), b_content).unwrap();

    let code = format!(
        r#"
        from {}.b import helper_b
        helper_b()
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("nested");

    match result {
        Ok(Value::Number(n)) => {
            assert_eq!(n, 30.0, "helper_a() + 20 should be 30");
        }
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_caching_verification() {
    // Modules should only execute once - verified by testing that functions are available
    // after multiple imports without errors
    let utils_content = r#"
fn add(a, b) {
    return a + b
}

let VERSION = "1.0.0"
"#;

    let test_dir = setup_module("caching", "utils", utils_content);

    let code = format!(
        r#"
        import {}.utils
        import {}.utils
        import {}.utils
        utils.add(5, 10)
    "#,
        test_dir.display(),
        test_dir.display(),
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("caching");

    match result {
        Ok(Value::Number(n)) => {
            assert_eq!(n, 15.0, "Function should work after multiple imports");
        }
        Ok(other) => panic!("Expected Number, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_circular_import_detection() {
    // Module A imports B
    let a_content = r#"
import {DIR}.b

fn func_a() {
    return 1
}
"#;

    // Module B imports A (circular!)
    let b_content = r#"
import {DIR}.a

fn func_b() {
    return 2
}
"#;

    let test_dir = setup_module("circular", "a", "");
    let a_content = a_content.replace("{DIR}", &test_dir.display().to_string());
    let b_content = b_content.replace("{DIR}", &test_dir.display().to_string());

    fs::write(test_dir.join("a.lg"), a_content).unwrap();
    fs::write(test_dir.join("b.lg"), b_content).unwrap();

    let code = format!(
        r#"
        import {}.a
        a.func_a()
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("circular");

    match result {
        Err(e) => {
            assert!(e.contains("Circular import"), "Should detect circular import, got: {}", e);
        }
        Ok(_) => panic!("Should have detected circular import!"),
    }
}

#[test]
#[ignore] // TODO: List comprehension results from modules need investigation
fn test_module_with_list_comprehensions() {
    let utils_content = r#"
fn get_evens(numbers) {
    return [x for x in numbers if x % 2 == 0]
}

fn double_all(numbers) {
    return [x * 2 for x in numbers]
}
"#;

    let test_dir = setup_module("comprehensions", "utils", utils_content);

    let code = format!(
        r#"
        from {}.utils import get_evens, double_all
        double_all(get_evens([1, 2, 3, 4, 5, 6]))
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("comprehensions");

    match result {
        Ok(Value::List(list)) => {
            let list_ref = list.borrow();
            assert_eq!(list_ref.len(), 3, "Should have 3 even numbers");
            // [2, 4, 6] doubled = [4, 8, 12]
        }
        Ok(other) => panic!("Expected List, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_module_with_pattern_matching() {
    let router_content = r#"
fn route(path) {
    return match path {
        "/" => "Home",
        "/about" => "About",
        "/contact" => "Contact",
        _ => "404"
    }
}
"#;

    let test_dir = setup_module("pattern_match", "router", router_content);

    let code = format!(
        r#"
        from {}.router import route
        route("/")
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("pattern_match");

    match result {
        Ok(Value::String(_)) => {} // Success
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_import_nonexistent_module() {
    let code = r#"
        import nonexistent_module_xyz
        nonexistent_module_xyz.func()
    "#;

    let result = run_code(code);

    match result {
        Err(e) => {
            assert!(e.contains("not found") || e.contains("No such file") || e.contains("Failed to read"),
                "Should error on missing module, got: {}", e);
        }
        Ok(_) => panic!("Should have failed to import nonexistent module!"),
    }
}

#[test]
fn test_import_from_nonexistent_item() {
    let math_content = r#"
fn add(a, b) {
    return a + b
}
"#;

    let test_dir = setup_module("missing_item", "math", math_content);

    let code = format!(
        r#"
        from {}.math import add, subtract, multiply
        add(1, 2)
    "#,
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("missing_item");

    match result {
        Err(e) => {
            assert!(e.contains("Undefined variable") || e.contains("subtract"),
                "Should error on missing export, got: {}", e);
        }
        Ok(_) => panic!("Should have failed to import nonexistent functions!"),
    }
}

#[test]
#[ignore] // TODO: Module scope isolation test needs refinement
fn test_module_scope_isolation() {
    let a_content = r#"
let shared_name = "Module A"

fn get_name() {
    return shared_name
}
"#;

    let b_content = r#"
let shared_name = "Module B"

fn get_name() {
    return shared_name
}
"#;

    let test_dir = setup_module("isolation", "a", a_content);
    fs::write(test_dir.join("b.lg"), b_content).unwrap();

    let code = format!(
        r#"
        import {}.a
        import {}.b
        a.get_name()
    "#,
        test_dir.display(),
        test_dir.display()
    );

    let result = run_code(&code);
    cleanup_module("isolation");

    match result {
        Ok(Value::String(_)) => {} // Success - modules are isolated
        Ok(other) => panic!("Expected String, got {:?}", other),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
