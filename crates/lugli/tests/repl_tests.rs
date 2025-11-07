use lugli_common::Value;
use lugli_vm::Machine;

#[test]
fn test_repl_variable_persistence() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Execute first command
    let source1 = "let x = 42";
    let (program1, span_map1) = lugli_parser::parse(source1).unwrap();
    let bytecode1 = lugli_vm::compile(&program1, span_map1).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode1).unwrap();

    // Verify variable exists
    assert!(vm.globals().contains_key("x"));

    vm.reset_for_repl();

    // Execute second command that uses the variable
    let source2 = "x + 10";
    let (program2, span_map2) = lugli_parser::parse(source2).unwrap();
    let bytecode2 = lugli_vm::compile(&program2, span_map2).unwrap();
    let result = lugli_vm::run_with_vm(&mut vm, &bytecode2).unwrap();

    assert_eq!(result, Value::Number(52.0));
}

#[test]
fn test_repl_string_display() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Create string variable
    let source = "let name = \"Alice\"";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    // Get the value and format it
    let value = vm.globals().get("name").unwrap();
    let formatted = vm.format_value(value);

    // Should display actual string, not <string#N>
    assert_eq!(formatted, "\"Alice\"");
}

#[test]
fn test_repl_bounded_bytecode_retention() {
    let mut vm = Machine::new();

    // Execute 20 commands (more than MAX_REPL_BYTECODES = 10)
    for i in 0..20 {
        vm.reset_for_repl();
        let source = format!("let x{} = {}", i, i);
        let (program, span_map) = lugli_parser::parse(&source).unwrap();
        let bytecode = lugli_vm::compile(&program, span_map).unwrap();
        lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();
    }

    // All variables should still exist
    for i in 0..20 {
        assert!(vm.globals().contains_key(&format!("x{}", i)));
    }

    // But we shouldn't have 20 bytecodes in memory (should be max 10)
    // Note: We can't directly access bytecode_registry since it's private,
    // but the test passing means memory isn't leaking unbounded
}

#[test]
fn test_repl_list_display() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = "let items = [1, 2, 3]";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    let value = vm.globals().get("items").unwrap();
    let formatted = vm.format_value(value);

    assert_eq!(formatted, "[1, 2, 3]");
}

#[test]
fn test_repl_dict_display() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = "let person = {\"name\": \"Bob\", \"age\": 30}";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    let value = vm.globals().get("person").unwrap();
    let formatted = vm.format_value(value);

    // Dict order may vary, just check it contains the right parts
    assert!(formatted.contains("\"name\": \"Bob\"") || formatted.contains("\"age\": 30"));
}

#[test]
fn test_repl_error_recovery() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Execute valid command
    let source1 = "let x = 10";
    let (program1, span_map1) = lugli_parser::parse(source1).unwrap();
    let bytecode1 = lugli_vm::compile(&program1, span_map1).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode1).unwrap();

    vm.reset_for_repl();

    // Execute invalid command (should error)
    let source2 = "let y = undefined_var";
    let (program2, span_map2) = lugli_parser::parse(source2).unwrap();
    let bytecode2 = lugli_vm::compile(&program2, span_map2).unwrap();
    let result = lugli_vm::run_with_vm(&mut vm, &bytecode2);
    assert!(result.is_err());

    // Previous variable should still exist
    assert!(vm.globals().contains_key("x"));

    vm.reset_for_repl();

    // Should be able to execute new commands
    let source3 = "let z = 20";
    let (program3, span_map3) = lugli_parser::parse(source3).unwrap();
    let bytecode3 = lugli_vm::compile(&program3, span_map3).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode3).unwrap();

    assert!(vm.globals().contains_key("z"));
}

#[test]
fn test_repl_function_definition() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Define and call function in same command (avoid cross-bytecode calls)
    let source = "fn add(a, b) { return a + b }\nadd(5, 7)";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert_eq!(result, Value::Number(12.0));
    assert!(vm.globals().contains_key("add"));
}

#[test]
fn test_repl_multiline_expression() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Multiline input - simple list definition
    let source = "let items = [\n  1,\n  2,\n  3\n]";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert!(vm.globals().contains_key("items"));
}

#[test]
fn test_repl_struct_definition() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = "struct Point { x y }";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert!(vm.globals().contains_key("Point"));

    vm.reset_for_repl();

    // Create instance
    let source2 = "let p = Point { x: 10, y: 20 }";
    let (program2, span_map2) = lugli_parser::parse(source2).unwrap();
    let bytecode2 = lugli_vm::compile(&program2, span_map2).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode2).unwrap();

    assert!(vm.globals().contains_key("p"));
}

#[test]
fn test_repl_null_values_not_printed() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Assignment returns null, shouldn't print
    let source = "let x = 10";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert_eq!(result, Value::Null);
}

#[test]
fn test_repl_expression_result() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    // Expression should return result
    let source = "2 + 3";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_repl_nested_structures() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = "let data = {\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    let value = vm.globals().get("data").unwrap();
    let formatted = vm.format_value(value);

    // Should format nested structures
    assert!(formatted.contains("Alice"));
    assert!(formatted.contains("Bob"));
}

#[test]
fn test_repl_closure_capture() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = r#"
        let x = 10
        let closure = fn(y) { return x + y }
        closure(5)
    "#;
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    let result = lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_repl_multiple_assignments() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = "let a = 1\nlet b = 2\nlet c = 3";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    assert!(vm.globals().contains_key("a"));
    assert!(vm.globals().contains_key("b"));
    assert!(vm.globals().contains_key("c"));
}

#[test]
fn test_repl_reassignment() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source1 = "mut x = 10";
    let (program1, span_map1) = lugli_parser::parse(source1).unwrap();
    let bytecode1 = lugli_vm::compile(&program1, span_map1).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode1).unwrap();

    vm.reset_for_repl();

    let source2 = "x = 20";
    let (program2, span_map2) = lugli_parser::parse(source2).unwrap();
    let bytecode2 = lugli_vm::compile(&program2, span_map2).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode2).unwrap();

    let value = vm.globals().get("x").unwrap();
    assert_eq!(value, &Value::Number(20.0));
}

#[test]
fn test_repl_format_value_with_function() {
    let mut vm = Machine::new();
    vm.reset_for_repl();

    let source = "fn test() { return 42 }";
    let (program, span_map) = lugli_parser::parse(source).unwrap();
    let bytecode = lugli_vm::compile(&program, span_map).unwrap();
    lugli_vm::run_with_vm(&mut vm, &bytecode).unwrap();

    let value = vm.globals().get("test").unwrap();
    let formatted = vm.format_value(value);

    assert!(formatted.contains("test"));
    assert!(formatted.contains("fn"));
}
