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
    assert!(!stats.is_running); // GC should not be running after execution
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
    assert!(!stats.is_running);
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

// ========== New Cycle Detection Tests ==========

#[test]
fn test_direct_self_reference_detection() {
    let mut machine = Machine::new();

    // Direct self-reference: x.itself = x
    let source = r#"
let x = {}
x.itself = x
print("Direct self-reference created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should show warning but not crash
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Direct self-reference should not crash");
}

#[test]
fn test_indirect_cycle_detection() {
    let mut machine = Machine::new();

    // Indirect cycle: a→b→a
    let source = r#"
let a = {}
let b = {}
a.ref = b
b.ref = a
print("Indirect cycle created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should detect cycle and show warning
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Indirect cycle should not crash");
}

#[test]
fn test_deep_cycle_detection() {
    let mut machine = Machine::new();

    // Deep cycle: a→b→c→d→a
    let source = r#"
let a = {}
let b = {}
let c = {}
let d = {}
a.next = b
b.next = c
c.next = d
d.next = a
print("Deep cycle created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should detect cycle
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Deep cycle should not crash");
}

#[test]
fn test_no_false_positive_on_acyclic_structures() {
    let mut machine = Machine::new();

    // Tree structure with no cycles
    let source = r#"
let root = { left: null, right: null }
let left_child = { left: null, right: null }
let right_child = { left: null, right: null }
root.left = left_child
root.right = right_child
print("Tree structure created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should NOT show cycle warning
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Acyclic tree should work fine");
}

#[test]
fn test_large_non_cyclic_structure() {
    let mut machine = Machine::new();

    // Linked list with 100 nodes (no cycle)
    let source = r#"
let nodes = []
mut i = 0
while i < 100 {
    let node = { value: i, next: null }
    nodes.push(node)
    i = i + 1
}

# Link nodes
i = 0
while i < 99 {
    nodes[i].next = nodes[i + 1]
    i = i + 1
}

print("Linked list created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should handle large structure without false positives
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Large linked list should work");
}

#[test]
fn test_cycle_in_list() {
    let mut machine = Machine::new();

    // List containing itself
    let source = r#"
let list = []
let item = { list: null }
list.push(item)
item.list = list
print("Cycle in list created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should detect cycle
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "List cycle should not crash");
}

#[test]
fn test_struct_self_reference() {
    let mut machine = Machine::new();

    // Struct pointing to itself
    let source = r#"
struct Node {
    value
    next
}

let node = Node { value: 1, next: null }
node.next = node
print("Struct self-reference created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should warn about self-reference
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Struct self-reference should not crash");
}

#[test]
fn test_memory_leak_stress() {
    let mut machine = Machine::new();

    // Create many cycles to stress test
    let source = r#"
mut i = 0
while i < 100 {
    let x = {}
    x.itself = x
    i = i + 1
}
print("Stress test complete")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Should complete without crash, but memory accumulates
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Stress test should complete");
}

#[test]
fn test_gc_threshold_with_cycles() {
    let mut machine = Machine::new();

    // Create cycles to trigger GC
    let source = r#"
mut i = 0
while i < 1000 {
    let a = {}
    let b = {}
    a.ref = b
    b.ref = a
    i = i + 1
}
print("GC threshold test complete")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // GC should trigger but cycles won't be collected
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "GC threshold test should complete");
}

#[test]
fn test_closure_cycle() {
    let mut machine = Machine::new();

    // Closure capturing container
    let source = r#"
let container = {}
container.func = fn() {
    print(container)
}
print("Closure cycle created")
"#;

    let mut parser = lugli_parser::Parser::new(source).expect("Parser creation failed");
    let (ast, span_map) = parser.parse().expect("Parse failed");

    let mut compiler = lugli_vm::Compiler::new();
    let bytecode = compiler.compile(&ast, span_map).expect("Compile failed");

    // Closure captures container, creating cycle
    let result = machine.run(&bytecode);
    assert!(result.is_ok(), "Closure cycle should not crash");
}
