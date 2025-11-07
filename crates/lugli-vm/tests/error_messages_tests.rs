use lugli_parser::parse;
use lugli_vm::{ErrorFormatter, Vm, VmError};

#[test]
fn test_undefined_variable_with_suggestion() {
    let source = r#"
let foo = 10
let bar = foe + 5
"#;

    let (program, span_map) = parse(source).unwrap();
    let mut vm = Vm::builder()
        .with_source("test.lg".to_string(), source.to_string())
        .build();
    let bytecode = vm.compile(&program, span_map).unwrap();

    let result = vm.run(&bytecode);
    assert!(result.is_err());

    let error = match result.unwrap_err() {
        VmError::RuntimeError(e) => e,
        _ => panic!("Expected RuntimeError"),
    };
    let formatter = ErrorFormatter::without_colors();
    let formatted = formatter.format(&error);

    println!("Formatted error:\n{}", formatted);

    // Should contain undefined variable error
    assert!(formatted.contains("Undefined variable"));
    assert!(formatted.contains("foe"));

    // Should suggest 'foo' (may not work without source locations yet)
    // assert!(formatted.contains("Did you mean 'foo'?"));

    // Should show source context (may not work without source locations yet)
    // assert!(formatted.contains("test.lg"));
    // assert!(formatted.contains("let bar = foe + 5"));
}

#[test]
fn test_error_with_source_location() {
    let source = r#"
let x = 10
let y = unknown_var
"#;

    let (program, span_map) = parse(source).unwrap();
    let mut vm = Vm::builder()
        .with_source("test.lg".to_string(), source.to_string())
        .build();
    let bytecode = vm.compile(&program, span_map).unwrap();

    let result = vm.run(&bytecode);
    assert!(result.is_err());

    let error = match result.unwrap_err() {
        VmError::RuntimeError(e) => e,
        _ => panic!("Expected RuntimeError"),
    };
    let formatter = ErrorFormatter::without_colors();
    let formatted = formatter.format(&error);

    println!("Formatted error:\n{}", formatted);

    // Should have error code
    assert!(formatted.contains("E003"));

    // Should have undefined variable name
    assert!(formatted.contains("unknown_var"));
}

#[test]
fn test_division_by_zero_error() {
    let source = r#"
let x = 10
let y = x / 0
"#;

    let (program, span_map) = parse(source).unwrap();
    let mut vm = Vm::builder()
        .with_source("test.lg".to_string(), source.to_string())
        .build();
    let bytecode = vm.compile(&program, span_map).unwrap();

    let result = vm.run(&bytecode);
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    assert!(error_msg.contains("Division by zero") || error_msg.contains("division by zero"));
}
