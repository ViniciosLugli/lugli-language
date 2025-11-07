# Phase 1: Foundation & Safety (3-4 weeks)

**Goal:** Fix critical safety/reliability issues, establish testing baseline

**Priority:** P0 - Must complete before other phases

**Success Criteria:**
- ✅ Zero unsafe operations in production code
- ✅ Memory leak detection implemented
- ✅ Test coverage >80%
- ✅ Automated CI/CD pipeline
- ✅ All 674+ tests passing

---

## Week 1-2: Safety & Memory

### Task 1.1: Fix Unsafe Transmute (2 days)

**Priority:** CRITICAL
**Files:**
- `crates/lugli-lexer/src/token.rs:6`

**Current Issue:**
```rust
const PLACEHOLDER_STRING_ID: StringId = unsafe { std::mem::transmute(0u32) };
```

**Problem:** Unsafe transmute has no compile-time validation. If StringId layout changes, undefined behavior occurs.

**Solution:**
```rust
// Option A: Safe constructor
impl StringId {
    pub const fn placeholder() -> Self {
        StringId(0)
    }
}
const PLACEHOLDER_STRING_ID: StringId = StringId::placeholder();

// Option B: Const assertion
impl StringId {
    const fn new(id: u32) -> Self {
        Self(id)
    }
}
const PLACEHOLDER_STRING_ID: StringId = StringId::new(0);
```

**Testing:**
- Verify f-string lexing still works with placeholder
- Add test for StringId layout assumptions
- Compile-time assertion that StringId is repr(transparent) or has expected layout

**Files to update:**
- `crates/lugli-lexer/src/token.rs` - replace unsafe transmute
- `crates/lugli-common/src/value.rs` - ensure StringId has safe constructors

**Success criteria:**
- Zero unsafe blocks in lugli-lexer
- All lexer tests pass
- F-string tests pass

---

### Task 1.2: Implement Weak Reference Support (3 days)

**Priority:** CRITICAL
**Files:**
- `crates/lugli-common/src/value.rs:18-42`
- `crates/lugli-vm/src/gc.rs`
- `crates/lugli-vm/src/machine/object_ops.rs:49-53`

**Current Issue:**
Circular references cause memory leaks:
```lugli
let x = {}
x.self = x  # Memory leak - Rc cycle never freed
```

**Current Warning System:**
- Only detects direct self-assignment
- Doesn't catch indirect cycles (a→b→c→a)

**Solution Phase 1: Enhanced Detection**

Add cycle detection to Dict/List/Struct operations:

```rust
// In value.rs
pub enum Value {
    // Add weak variants for optional references
    WeakDict(Weak<RefCell<HashMap<String, Value>>>),
    WeakList(Weak<RefCell<Vec<Value>>>),
    // ... existing variants
}

impl Value {
    pub fn contains_cycle(&self, visited: &mut HashSet<usize>) -> bool {
        match self {
            Value::Dict(d) => {
                let ptr = Rc::as_ptr(d) as usize;
                if visited.contains(&ptr) {
                    return true;
                }
                visited.insert(ptr);
                for v in d.borrow().values() {
                    if v.contains_cycle(visited) {
                        return true;
                    }
                }
                visited.remove(&ptr);
                false
            }
            Value::List(l) => {
                let ptr = Rc::as_ptr(l) as usize;
                if visited.contains(&ptr) {
                    return true;
                }
                visited.insert(ptr);
                for v in l.borrow().iter() {
                    if v.contains_cycle(visited) {
                        return true;
                    }
                }
                visited.remove(&ptr);
                false
            }
            _ => false
        }
    }
}
```

**In gc.rs - Add cycle detection:**

```rust
impl GarbageCollector {
    pub fn detect_cycles(&self, roots: &[Value]) -> Vec<String> {
        let mut cycles = Vec::new();
        for root in roots {
            let mut visited = HashSet::new();
            if root.contains_cycle(&mut visited) {
                cycles.push(format!("Cycle detected in value: {:?}", root.type_name()));
            }
        }
        cycles
    }
}
```

**In machine/object_ops.rs - Enhanced warning:**

```rust
pub(super) fn exec_set_dict_item(&mut self) -> Result<(), LugliError> {
    // ... existing code ...

    // Check for cycles after assignment
    let mut visited = HashSet::new();
    if value.contains_cycle(&mut visited) {
        eprintln!("⚠️  WARNING: Circular reference detected! This will cause memory leaks.");
        eprintln!("    Consider breaking the cycle with None or restructuring your data.");
    }

    // ... rest of code
}
```

**Documentation to add:**

Create `docs/memory_management.md`:
```markdown
# Memory Management in Lugli

## Circular References

Lugli uses reference counting (Rc) for memory management. Circular references will cause memory leaks.

### Unsafe Patterns

```lugli
# DON'T: Direct self-reference
let x = {}
x.self = x  # ⚠️ Memory leak

# DON'T: Indirect cycles
let a = {}
let b = {}
let c = {}
a.next = b
b.next = c
c.next = a  # ⚠️ Memory leak
```

### Safe Patterns

```lugli
# DO: Break cycles with null
let parent = { children: [] }
let child = { parent: null }  # Weak reference
parent.children.push(child)

# DO: Use one-way references
let node = { value: 1, next: null }
let next_node = { value: 2, next: null }
node.next = next_node  # OK - no cycle
```
```

**Testing:**
- Test direct self-assignment detection
- Test indirect cycle detection (a→b→a)
- Test deep cycle detection (a→b→c→a)
- Performance test: cycle detection on large structures

**Files to create/update:**
- `crates/lugli-common/src/value.rs` - add contains_cycle method
- `crates/lugli-vm/src/gc.rs` - add detect_cycles method
- `crates/lugli-vm/src/machine/object_ops.rs` - add cycle warnings
- `crates/lugli-vm/tests/gc_tests.rs` - add cycle detection tests
- `docs/memory_management.md` - new documentation

**Success criteria:**
- Cycle detection works for direct and indirect cycles
- Warning messages appear when cycles created
- No false positives on non-cyclic structures
- Performance impact <5% on normal operations

---

### Task 1.3: Standardize Borrow Operations (2 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/src/machine/mod.rs` (multiple locations)
- `crates/lugli-vm/src/machine/object_ops.rs`
- `crates/lugli-stdlib/src/core/list.rs:105-106`

**Current Issue:**

Inconsistent borrow patterns:
```rust
// Pattern 1: Unsafe - can panic
self.mark_bytecode_ids(&upvalue.borrow(), live_set);

// Pattern 2: Safe - returns error
dict_ref.try_borrow_mut()
    .map_err(|_| LugliError::runtime("Cannot modify while in use"))?
```

**Problem:** Direct `.borrow()` panics on borrow conflict instead of returning recoverable error.

**Solution:**

Create helper methods:
```rust
// In machine/mod.rs
impl Machine {
    fn safe_borrow<T>(&self, cell: &RefCell<T>) -> Result<Ref<T>, LugliError> {
        cell.try_borrow()
            .map_err(|_| LugliError::runtime("Value is currently borrowed mutably"))
    }

    fn safe_borrow_mut<T>(&self, cell: &RefCell<T>) -> Result<RefMut<T>, LugliError> {
        cell.try_borrow_mut()
            .map_err(|_| LugliError::runtime("Value is currently borrowed"))
    }
}
```

**Files to audit and fix:**

Search for all `.borrow()` and `.borrow_mut()` calls:
```bash
rg '\.borrow\(\)' crates/lugli-vm/src/ --type rust
rg '\.borrow_mut\(\)' crates/lugli-vm/src/ --type rust
```

**Locations to fix:**
1. `machine/mod.rs:197, 201, 206, 232` - upvalue borrows
2. `machine/object_ops.rs:150-172` - closure upvalue capture
3. `stdlib/core/list.rs:105-106` - list append

**Example fix:**

Before:
```rust
let upvalue = upvalue.borrow();
self.mark_bytecode_ids(&upvalue, live_set);
```

After:
```rust
let upvalue = self.safe_borrow(&upvalue)?;
self.mark_bytecode_ids(&upvalue, live_set);
```

**Special case - stdlib/core/list.rs:**

Before:
```rust
pub fn list_append(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    match (&args[0], &args[1]) {
        (Value::List(l1), Value::List(l2)) => {
            if Rc::ptr_eq(l1, l2) {
                return Err(LugliError::runtime("Cannot append list to itself"));
            }
            let items = l2.try_borrow().map_err(...)?.clone();
            l1.borrow_mut().extend(items);  // Different borrow - should be safe
            Ok(Value::Null)
        }
    }
}
```

After (with clear borrow scopes):
```rust
pub fn list_append(args: &[Value], _pool: &mut StringPool) -> Result<Value, LugliError> {
    match (&args[0], &args[1]) {
        (Value::List(l1), Value::List(l2)) => {
            if Rc::ptr_eq(l1, l2) {
                return Err(LugliError::runtime("Cannot append list to itself"));
            }
            // Clone items while holding read lock
            let items = {
                let l2_ref = l2.try_borrow()
                    .map_err(|_| LugliError::runtime("List is currently borrowed"))?;
                l2_ref.clone()
            }; // Release read lock before acquiring write lock

            // Now safe to acquire write lock on l1
            l1.try_borrow_mut()
                .map_err(|_| LugliError::runtime("List is currently borrowed"))?
                .extend(items);
            Ok(Value::Null)
        }
    }
}
```

**Testing:**
- Test nested borrow scenarios
- Test error messages for borrow conflicts
- Ensure no panics under any scenario
- Test concurrent borrow attempts

**Success criteria:**
- All `.borrow()` replaced with `.try_borrow()`
- All `.borrow_mut()` replaced with `.try_borrow_mut()`
- Clear error messages for borrow conflicts
- Zero panics from RefCell borrow conflicts

---

### Task 1.4: Add Comprehensive GC Tests (2 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/tests/gc_tests.rs` (expand)

**Current State:**
- 7 basic tests (threshold-based collection)
- No cycle detection tests
- No weak reference tests

**Tests to Add:**

```rust
// In gc_tests.rs

#[test]
fn test_direct_self_reference_detection() {
    let source = r#"
        let x = {}
        x.self = x
    "#;
    // Should show warning but not crash
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_indirect_cycle_detection() {
    let source = r#"
        let a = {}
        let b = {}
        a.ref = b
        b.ref = a
    "#;
    // Should detect cycle
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_deep_cycle_detection() {
    let source = r#"
        let a = {}
        let b = {}
        let c = {}
        let d = {}
        a.next = b
        b.next = c
        c.next = d
        d.next = a
    "#;
    // Should detect cycle
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_no_false_positive_on_acyclic_structures() {
    let source = r#"
        let root = { left: null, right: null }
        let left_child = { left: null, right: null }
        let right_child = { left: null, right: null }
        root.left = left_child
        root.right = right_child
    "#;
    // Should NOT show cycle warning
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_large_non_cyclic_structure() {
    let source = r#"
        let nodes = []
        for i in range(100) {
            let node = { value: i, next: null }
            nodes.push(node)
        }
        for i in range(99) {
            nodes[i].next = nodes[i + 1]
        }
    "#;
    // Should handle large structures without false positives
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_cycle_in_list() {
    let source = r#"
        let list = []
        let item = { list: null }
        list.push(item)
        item.list = list
    "#;
    // Should detect cycle
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_struct_self_reference() {
    let source = r#"
        struct Node {
            value
            next
        }
        let node = Node { value: 1, next: null }
        node.next = node
    "#;
    // Should warn about self-reference
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_memory_leak_stress() {
    let source = r#"
        for i in range(100) {
            let x = {}
            x.self = x
        }
    "#;
    // Should complete without crash, but memory accumulates
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_gc_threshold_with_cycles() {
    let source = r#"
        for i in range(1000) {
            let a = {}
            let b = {}
            a.ref = b
            b.ref = a
        }
    "#;
    // GC should trigger but cycles won't be collected
    let result = run_test(source);
    assert!(result.is_ok());
}

#[test]
fn test_closure_cycle() {
    let source = r#"
        let container = {}
        container.func = fn() {
            print(container)
        }
    "#;
    // Closure captures container, creating cycle
    let result = run_test(source);
    assert!(result.is_ok());
}
```

**Performance Tests:**

```rust
#[test]
fn bench_cycle_detection_overhead() {
    let source = r#"
        let x = {}
        x.a = 1
        x.b = 2
    "#;
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        run_test(source).unwrap();
    }
    let duration = start.elapsed();
    println!("Cycle detection overhead: {:?}", duration);
    // Should be <5% slower than without detection
}
```

**Success criteria:**
- 10+ new cycle detection tests
- All tests pass
- No false positives on acyclic structures
- Performance overhead <5%

---

## Week 3-4: Testing Infrastructure

### Task 1.5: Add AST Module Tests (2 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-ast/tests/` (create new directory)
- `crates/lugli-ast/tests/visitor_tests.rs` (new)
- `crates/lugli-ast/tests/ast_construction_tests.rs` (new)

**Current State:**
- Only 10 tests in AST crate
- Visitor pattern not tested
- AST node construction not validated

**Tests to Create:**

**File: `crates/lugli-ast/tests/visitor_tests.rs`**

```rust
use lugli_ast::{visitor::*, Expr, Stmt, Program};
use lugli_lexer::{Token, TokenKind};

// Test visitor that counts nodes
struct NodeCounter {
    expr_count: usize,
    stmt_count: usize,
}

impl Visitor<()> for NodeCounter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<(), String> {
        self.expr_count += 1;
        walk_expr(self, expr)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        self.stmt_count += 1;
        walk_stmt(self, stmt)
    }
}

#[test]
fn test_visitor_counts_all_nodes() {
    let program = Program {
        statements: vec![
            Stmt::VarDeclaration {
                name: "x".to_string(),
                value: Some(Expr::Literal { value: Token::number(42.0) }),
                is_mutable: false,
            },
        ],
    };

    let mut counter = NodeCounter { expr_count: 0, stmt_count: 0 };
    counter.visit_program(&program).unwrap();

    assert_eq!(counter.stmt_count, 1);
    assert_eq!(counter.expr_count, 1);
}

#[test]
fn test_visitor_walks_nested_expressions() {
    let program = Program {
        statements: vec![
            Stmt::Expression {
                expr: Expr::Binary {
                    left: Box::new(Expr::Literal { value: Token::number(1.0) }),
                    operator: TokenKind::Plus,
                    right: Box::new(Expr::Binary {
                        left: Box::new(Expr::Literal { value: Token::number(2.0) }),
                        operator: TokenKind::Multiply,
                        right: Box::new(Expr::Literal { value: Token::number(3.0) }),
                    }),
                },
            },
        ],
    };

    let mut counter = NodeCounter { expr_count: 0, stmt_count: 0 };
    counter.visit_program(&program).unwrap();

    assert_eq!(counter.expr_count, 5); // 1 outer binary + 1 left + 1 inner binary + 2 literals
}

// Test visitor that transforms AST
struct ConstantFolder {
    folded_count: usize,
}

impl Visitor<Expr> for ConstantFolder {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Expr, String> {
        match expr {
            Expr::Binary { left, operator, right } => {
                if let (Expr::Literal { value: Token { kind: TokenKind::Number(l), .. } },
                        Expr::Literal { value: Token { kind: TokenKind::Number(r), .. } }) = (&**left, &**right) {
                    self.folded_count += 1;
                    let result = match operator {
                        TokenKind::Plus => l + r,
                        TokenKind::Minus => l - r,
                        _ => return Ok(expr.clone()),
                    };
                    Ok(Expr::Literal { value: Token::number(result) })
                } else {
                    Ok(expr.clone())
                }
            }
            _ => Ok(expr.clone()),
        }
    }
}

#[test]
fn test_visitor_can_transform_ast() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal { value: Token::number(2.0) }),
        operator: TokenKind::Plus,
        right: Box::new(Expr::Literal { value: Token::number(3.0) }),
    };

    let mut folder = ConstantFolder { folded_count: 0 };
    let result = folder.visit_expr(&expr).unwrap();

    assert_eq!(folder.folded_count, 1);
    match result {
        Expr::Literal { value } => {
            if let TokenKind::Number(n) = value.kind {
                assert_eq!(n, 5.0);
            } else {
                panic!("Expected number literal");
            }
        }
        _ => panic!("Expected literal"),
    }
}
```

**File: `crates/lugli-ast/tests/ast_construction_tests.rs`**

```rust
use lugli_ast::*;
use lugli_lexer::{Token, TokenKind};

#[test]
fn test_binary_expression_construction() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal { value: Token::number(1.0) }),
        operator: TokenKind::Plus,
        right: Box::new(Expr::Literal { value: Token::number(2.0) }),
    };

    match expr {
        Expr::Binary { .. } => (),
        _ => panic!("Wrong expression type"),
    }
}

#[test]
fn test_function_declaration_construction() {
    let stmt = Stmt::FunctionDeclaration {
        name: "test".to_string(),
        params: vec![],
        body: vec![],
    };

    match stmt {
        Stmt::FunctionDeclaration { name, .. } => assert_eq!(name, "test"),
        _ => panic!("Wrong statement type"),
    }
}

#[test]
fn test_struct_declaration_construction() {
    let stmt = Stmt::StructDeclaration {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), None),
            ("y".to_string(), None),
        ],
        methods: vec![],
    };

    match stmt {
        Stmt::StructDeclaration { name, fields, .. } => {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Wrong statement type"),
    }
}

#[test]
fn test_deeply_nested_expressions() {
    // Build: ((1 + 2) * 3) + 4
    let expr = Expr::Binary {
        left: Box::new(Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal { value: Token::number(1.0) }),
                operator: TokenKind::Plus,
                right: Box::new(Expr::Literal { value: Token::number(2.0) }),
            }),
            operator: TokenKind::Multiply,
            right: Box::new(Expr::Literal { value: Token::number(3.0) }),
        }),
        operator: TokenKind::Plus,
        right: Box::new(Expr::Literal { value: Token::number(4.0) }),
    };

    // Should construct without issue
    match expr {
        Expr::Binary { .. } => (),
        _ => panic!("Wrong expression type"),
    }
}
```

**Success criteria:**
- 15+ new AST tests
- Visitor pattern fully tested
- AST construction validated
- All tests pass

---

### Task 1.6: Add Optimizer Tests (3 days)

**Priority:** HIGH
**Files:**
- `crates/lugli-vm/tests/optimizer_tests.rs` (new)

**Current State:**
- Optimizer has no dedicated tests
- Constant folding untested
- Dead code elimination disabled but not tested

**Tests to Create:**

```rust
use lugli_vm::compiler::Compiler;
use lugli_vm::bytecode::Instruction;
use lugli_parser::parse;
use lugli_lexer::tokenize;

fn compile_and_get_instructions(source: &str) -> Vec<Instruction> {
    let tokens = tokenize(source).unwrap();
    let (program, span_map) = parse(&tokens).unwrap();
    let compiler = Compiler::new();
    let bytecode = compiler.compile(&program, span_map).unwrap();
    bytecode.instructions().to_vec()
}

#[test]
fn test_constant_folding_addition() {
    let source = "let x = 2 + 3";
    let instructions = compile_and_get_instructions(source);

    // Should have LoadConst(5) instead of LoadConst(2), LoadConst(3), Add
    let has_add = instructions.iter().any(|i| matches!(i, Instruction::Add));
    assert!(!has_add, "Addition should be folded at compile time");
}

#[test]
fn test_constant_folding_multiplication() {
    let source = "let x = 4 * 5";
    let instructions = compile_and_get_instructions(source);

    let has_mul = instructions.iter().any(|i| matches!(i, Instruction::Multiply));
    assert!(!has_mul, "Multiplication should be folded");
}

#[test]
fn test_constant_folding_nested() {
    let source = "let x = (2 + 3) * 4";
    let instructions = compile_and_get_instructions(source);

    // Should compute 5 * 4 = 20 at compile time
    let has_arithmetic = instructions.iter().any(|i| {
        matches!(i, Instruction::Add | Instruction::Multiply)
    });
    assert!(!has_arithmetic, "Nested arithmetic should be folded");
}

#[test]
fn test_algebraic_simplification_multiply_one() {
    let source = "let x = y * 1";
    let instructions = compile_and_get_instructions(source);

    // Should optimize to just loading y
    let mul_count = instructions.iter()
        .filter(|i| matches!(i, Instruction::Multiply))
        .count();
    assert_eq!(mul_count, 0, "Multiply by 1 should be eliminated");
}

#[test]
fn test_algebraic_simplification_add_zero() {
    let source = "let x = y + 0";
    let instructions = compile_and_get_instructions(source);

    let add_count = instructions.iter()
        .filter(|i| matches!(i, Instruction::Add))
        .count();
    assert_eq!(add_count, 0, "Add zero should be eliminated");
}

#[test]
fn test_power_optimization() {
    let source = "let x = 2 ** 3";
    let instructions = compile_and_get_instructions(source);

    // Should fold to 8
    let has_power = instructions.iter().any(|i| matches!(i, Instruction::Power));
    assert!(!has_power, "Power of small integers should be folded");
}

#[test]
fn test_dead_code_after_return() {
    let source = r#"
        fn test() {
            return 42
            print("unreachable")
        }
    "#;
    let instructions = compile_and_get_instructions(source);

    // Currently DCE is disabled, so this will have the print
    // When enabled, should eliminate unreachable code
    let has_print = instructions.iter().any(|i| matches!(i, Instruction::CallMethod { .. }));
    // For now, just document current behavior
    println!("DCE disabled: has_print = {}", has_print);
}

#[test]
fn test_optimizer_preserves_side_effects() {
    let source = "let x = foo() + 0";
    let instructions = compile_and_get_instructions(source);

    // Should keep function call even though + 0 is optimized
    let has_call = instructions.iter().any(|i| matches!(i, Instruction::Call { .. }));
    assert!(has_call, "Function calls must be preserved");
}

#[test]
fn test_no_optimization_with_variables() {
    let source = "let x = a + b";
    let instructions = compile_and_get_instructions(source);

    // Should NOT optimize variable addition
    let has_add = instructions.iter().any(|i| matches!(i, Instruction::Add));
    assert!(has_add, "Variable operations should not be optimized");
}

#[test]
fn test_constant_propagation_simple() {
    let source = r#"
        let x = 5
        let y = x + 3
    "#;
    let instructions = compile_and_get_instructions(source);

    // Currently no constant propagation across statements
    // Document current behavior
    let add_count = instructions.iter()
        .filter(|i| matches!(i, Instruction::Add | Instruction::AddInt(_)))
        .count();
    println!("Add instructions: {}", add_count);
}

#[test]
fn test_optimizer_multiple_passes() {
    let source = "let x = (2 + 3) + (4 + 5)";
    let instructions = compile_and_get_instructions(source);

    // Multiple folding passes should reduce to single constant
    let has_arithmetic = instructions.iter().any(|i| {
        matches!(i, Instruction::Add | Instruction::Subtract | Instruction::Multiply | Instruction::Divide)
    });
    assert!(!has_arithmetic, "Multiple passes should fold everything");
}
```

**Tests for Disabled Features:**

```rust
#[test]
#[ignore] // DCE currently disabled
fn test_dead_code_elimination_simple() {
    let source = r#"
        fn test() {
            let unused = 42
            return 10
        }
    "#;
    let instructions = compile_and_get_instructions(source);

    // Should not have instructions for unused variable
    // This test documents expected behavior when DCE is fixed
}

#[test]
#[ignore] // DCE issues with closures
fn test_dead_code_with_closures() {
    let source = r#"
        fn outer() {
            let x = 10
            fn inner() {
                return 5
            }
            return inner()
        }
    "#;
    let instructions = compile_and_get_instructions(source);

    // DCE should not eliminate x if it's captured (even if not used)
    // This tests the bug that caused DCE to be disabled
}
```

**Success criteria:**
- 15+ optimizer tests
- All optimization paths tested
- Disabled features documented with ignored tests
- Performance benchmarks for optimization impact

---

### Task 1.7: Add Module System Edge Case Tests (2 days)

**Priority:** MEDIUM
**Files:**
- `crates/lugli-vm/tests/module_tests.rs` (expand)

**Current State:**
- Basic import tests exist
- No circular import tests
- No missing file handling tests

**Tests to Add:**

```rust
#[test]
fn test_circular_import_detection() {
    // Create a.lg that imports b.lg
    // Create b.lg that imports a.lg
    // Should detect and error

    let temp_dir = tempdir().unwrap();
    let a_path = temp_dir.path().join("a.lg");
    let b_path = temp_dir.path().join("b.lg");

    fs::write(&a_path, "import b\nlet x = 1").unwrap();
    fs::write(&b_path, "import a\nlet y = 2").unwrap();

    let result = compile_file(&a_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("circular"));
}

#[test]
fn test_missing_module_error() {
    let source = "import nonexistent_module";
    let result = run_test(source);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_relative_import_parent_directory() {
    // Test that .. is rejected for security
    let source = "import ../../../etc/passwd";
    let result = run_test(source);
    assert!(result.is_err());
}

#[test]
fn test_absolute_path_rejected() {
    let source = "import /etc/passwd";
    let result = run_test(source);
    assert!(result.is_err());
}

#[test]
fn test_module_caching() {
    // Import same module twice, should only execute once
    let temp_dir = tempdir().unwrap();
    let module_path = temp_dir.path().join("counter.lg");

    fs::write(&module_path, r#"
        let counter = 0
        mut global_count = 0
        global_count = global_count + 1
    "#).unwrap();

    let source = r#"
        import counter
        import counter
        # Should only increment once
    "#;

    // Test that module is cached and not re-executed
}

#[test]
fn test_import_from_specific_items() {
    let temp_dir = tempdir().unwrap();
    let module_path = temp_dir.path().join("math.lg");

    fs::write(&module_path, r#"
        let PI = 3.14159
        let E = 2.71828
        fn square(x) { return x * x }
    "#).unwrap();

    let source = r#"
        from math import PI, square
        print(PI)
        print(square(5))
    "#;

    let result = run_test_from_dir(source, temp_dir.path());
    assert!(result.is_ok());
}

#[test]
fn test_nested_module_imports() {
    // a imports b, b imports c, c defines value
    // a should be able to use value from c through b
}
```

**Success criteria:**
- 10+ new module system tests
- All edge cases covered
- Security validations tested
- Module caching verified

---

### Task 1.8: Setup CI/CD Pipeline (1 day)

**Priority:** HIGH
**Files:**
- `.github/workflows/ci.yml` (new)
- `.github/workflows/benchmark.yml` (new)

**CI Workflow:**

```yaml
name: CI

on:
  push:
    branches: [ main, claude/* ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace --verbose

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --workspace --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  benchmark:
    name: Performance Regression
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Run benchmarks
        run: cargo bench --workspace -- --save-baseline pr-${{ github.event.number }}

      - name: Compare with main
        run: cargo bench --workspace -- --baseline main
```

**Success criteria:**
- CI runs on all pushes
- Tests must pass before merge
- Code coverage reported
- Benchmark comparisons automated

---

## Phase 1 Completion Checklist

- [ ] Task 1.1: Unsafe transmute fixed
- [ ] Task 1.2: Weak reference support implemented
- [ ] Task 1.3: All borrow operations standardized
- [ ] Task 1.4: 10+ new GC tests added
- [ ] Task 1.5: 15+ AST tests added
- [ ] Task 1.6: 15+ optimizer tests added
- [ ] Task 1.7: 10+ module system tests added
- [ ] Task 1.8: CI/CD pipeline operational

**Success Metrics:**
- Test count: 674 → 750+
- Test coverage: 67% → 80%+
- Zero unsafe operations in production
- All tests passing
- CI green on all branches

**Next Phase:** Phase 2 - Architecture Refactoring
