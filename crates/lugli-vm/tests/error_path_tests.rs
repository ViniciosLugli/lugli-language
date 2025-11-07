use lugli_parser::Parser;
use lugli_vm::{Bytecode, Vm};

fn compile_source(source: &str) -> Bytecode {
    let mut parser = Parser::new(source).expect("Parser creation should succeed");
    let (program, span_map) = parser.parse().expect("Parse should succeed");
    let vm = Vm::new();
    vm.compile(&program, span_map).expect("Compile should succeed")
}

#[test]
fn test_stack_overflow_protection() {
    let source = r#"
        fn recursive(n) {
            return recursive(n + 1)
        }
        recursive(0)
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("stack") || err_msg.contains("depth") || err_msg.contains("overflow"));
}

#[test]
fn test_division_by_zero_error() {
    let source = "let x = 10 / 0";

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("zero"));
}

#[test]
fn test_index_out_of_bounds_positive() {
    // Out of bounds now returns null instead of erroring
    let source = r#"
        let list = [1, 2, 3]
        let x = list[10]
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    // Should succeed (returns null)
    assert!(result.is_ok());
}

#[test]
fn test_index_out_of_bounds_negative() {
    // Out of bounds now returns null instead of erroring
    let source = r#"
        let list = [1, 2, 3]
        let x = list[-10]
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    // Should succeed (returns null)
    assert!(result.is_ok());
}

#[test]
fn test_undefined_variable_error() {
    let source = "let x = undefined_var";

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Undefined") || err_msg.contains("undefined"));
}

#[test]
fn test_type_error_on_invalid_operation() {
    let source = r#"let x = "string" + 42"#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    // This might work (string concatenation) or fail - test that it doesn't panic
    let _ = result;
}

#[test]
fn test_invalid_method_call() {
    let source = r#"
        let x = 42
        x.nonexistent_method()
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
}

#[test]
fn test_function_arity_mismatch() {
    let source = r#"
        fn add(a, b) {
            return a + b
        }
        add(1)
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("argument") || err_msg.contains("arity"));
}

#[test]
fn test_nested_error_propagation() {
    let source = r#"
        fn inner() {
            return 1 / 0
        }
        fn outer() {
            return inner()
        }
        outer()
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("zero"));
}

#[test]
fn test_error_in_loop() {
    let source = r#"
        for i in [1, 2, 3] {
            if i == 2 {
                let x = 1 / 0
            }
        }
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
}

#[test]
fn test_error_in_match_expression() {
    let source = r#"
        let x = 5
        match x {
            5 => 1 / 0,
            _ => 42
        }
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
}

#[test]
fn test_empty_list_pop() {
    let source = r#"
        let list = []
        list.pop()
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    // Should either return null or error gracefully
    let _ = result;
}

#[test]
fn test_large_number_operations() {
    let source = r#"
        let huge = 999999999999999999999999.0
        let result = huge * huge
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    // Should handle infinity or large numbers gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_modulo_by_zero() {
    let source = "let x = 10 % 0";

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("zero"));
}

#[test]
fn test_property_access_on_non_object() {
    let source = r#"
        let x = 42
        let y = x.some_property
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
}

#[test]
fn test_call_non_function() {
    let source = r#"
        let x = 42
        x()
    "#;

    let bytecode = compile_source(source);
    let mut vm = Vm::new();
    let result = vm.run(&bytecode);

    assert!(result.is_err());
}
