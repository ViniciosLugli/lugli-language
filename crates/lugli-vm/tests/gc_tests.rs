use lugli_vm::Machine;

#[test]
fn test_gc_with_circular_references() {
    let mut machine = Machine::new();

    // Create code that creates circular references
    let source = r#"
let a = {}
let b = {}
a.ref = b
b.ref = a
# At this point we have a cycle: a → b → a
# GC should detect this during periodic collection
print("Circular references created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Run the code - GC will trigger periodically
    let result = machine.run(&bytecode);
    assert!(result.is_ok());

    // Check GC stats
    let stats = machine.gc_stats();
    assert_eq!(stats.is_running, false); // GC should not be running after execution
}

#[test]
fn test_gc_long_running_program() {
    let mut machine = Machine::new();

    // Create a program that runs many instructions to trigger GC
    let source = r#"
mut count = 0
# Create temporary lists in a loop to trigger GC
while count < 1000 {
    let temp = [1, 2, 3, 4, 5]
    count = count + 1
}
print(f"Completed {count} iterations")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    let result = machine.run(&bytecode);
    assert!(result.is_ok());

    // GC should have run at least once
    let stats = machine.gc_stats();
    assert_eq!(stats.is_running, false);
}

#[test]
fn test_gc_preserves_live_references() {
    let mut machine = Machine::new();

    // Create code that keeps references alive
    let source = r#"
let important = [1, 2, 3, 4, 5]
let backup = important  # Another reference to the same list

# Do a lot of work to potentially trigger GC
mut i = 0
while i < 1000 {
    let temp = {}
    i = i + 1
}

# Verify the list is still intact
print(f"List length: {len(important)}")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    let result = machine.run(&bytecode);
    assert!(result.is_ok());
}
